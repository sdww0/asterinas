#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Count broad unsafe-related source regions, including complete items and bodies.

Includes safe members inside unsafe impl/trait/extern items and entire items
carrying unsafe attributes. Counts source as written, not macro expansions or
architecture-selected code. This is not a count of unsafe operations.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import tempfile
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, TextIO


RAW_STRING = re.compile(r"(?:b|c)?r(#*)\"")
IDENT_START = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
IDENT_CHARS = IDENT_START | set("0123456789")
COUNTING_POLICY = "full unsafe bodies and unsafe-attributed items; unique source lines"
REPOSITORY_ROOT = Path(__file__).resolve().parents[1]


def default_paths() -> list[Path]:
    return [REPOSITORY_ROOT / "kernel", REPOSITORY_ROOT / "ostd"]


@dataclass(frozen=True)
class UnsafeSpan:
    kind: str
    start_line: int
    end_line: int


@dataclass
class FileResult:
    path: Path
    total_lines: int
    spans: list[UnsafeSpan]
    occurrences: int
    unsafe_lines: set[int]
    code_lines: set[int]

    @property
    def unsafe_loc(self) -> int:
        return len(self.unsafe_lines & self.code_lines)

    @property
    def unsafe_physical_lines(self) -> int:
        return len(self.unsafe_lines)

    def construct_counts(self) -> Counter[str]:
        return Counter(span.kind for span in self.spans)


def skip_comment(text: str, index: int) -> int:
    if text.startswith("//", index):
        end = text.find("\n", index)
        return len(text) if end < 0 else end
    if text.startswith("/*", index):
        depth = 0
        while index < len(text):
            if text.startswith("/*", index):
                depth += 1
                index += 2
            elif text.startswith("*/", index):
                depth -= 1
                index += 2
                if depth == 0:
                    break
            else:
                index += 1
        return index
    return index


def skip_char_literal(text: str, index: int) -> int:
    if index + 1 >= len(text) or text[index + 1] != "\\":
        if index + 2 < len(text) and text[index + 2] == "'":
            return index + 3
        return index
    index += 2
    while index < len(text) and text[index] not in ("'", "\n"):
        if text[index] == "\\":
            index += 1
        index += 1
    if index < len(text) and text[index] == "'":
        return index + 1
    return index


def skip_non_code(text: str, index: int, code_lines: set[int] | None = None) -> int:
    while index < len(text):
        char = text[index]
        if char.isspace():
            index += 1
            continue

        comment_end = skip_comment(text, index)
        if comment_end != index:
            index = comment_end
            continue

        raw_match = RAW_STRING.match(text, index)
        if raw_match:
            start = index
            hashes = len(raw_match.group(1))
            terminator = '"' + "#" * hashes
            end = text.find(terminator, raw_match.end())
            index = len(text) if end < 0 else end + len(terminator)
            mark_literal_lines(text, start, index, code_lines)
            continue

        if char == '"':
            start = index
            index += 1
            while index < len(text):
                if text[index] == "\\":
                    index += 2
                    continue
                if text[index] == '"':
                    index += 1
                    break
                index += 1
            mark_literal_lines(text, start, index, code_lines)
            continue

        if char in ("b", "c") and index + 1 < len(text) and text[index + 1] in ('"', "'"):
            index += 1
            continue

        if char == "'":
            char_end = skip_char_literal(text, index)
            if char_end != index:
                if code_lines is not None:
                    code_lines.add(line_number(text, index))
                index = char_end
                continue

        return index
    return index


def mark_literal_lines(text: str, start: int, end: int, code_lines: set[int] | None) -> None:
    if code_lines is None:
        return
    first = line_number(text, start)
    for offset, content in enumerate(text[start:end].splitlines()):
        if content.strip():
            code_lines.add(first + offset)


def previous_significant(text: str, index: int) -> str:
    index -= 1
    while index >= 0:
        char = text[index]
        if char.isspace():
            index -= 1
            continue
        if text[index - 1 : index + 1] in ("*/", '"'):
            index -= 1
            continue
        return char
    return ""


def next_significant(text: str, index: int) -> int:
    return skip_non_code(text, index)


def read_identifier(text: str, index: int) -> tuple[str, int]:
    end = index + 1
    while end < len(text) and text[end] in IDENT_CHARS:
        end += 1
    return text[index:end], end


def scan_until(text: str, index: int, stops: Iterable[str]) -> tuple[str, int]:
    stop_set = set(stops)
    angle_depth = 0
    while index < len(text):
        index = skip_non_code(text, index)
        if index >= len(text):
            return "", index
        char = text[index]
        if char in "([":
            index = find_matching_delimiter(text, index, char, ")" if char == "(" else "]") + 1
            continue
        if char == "<":
            angle_depth += 1
        elif char == ">" and angle_depth:
            angle_depth -= 1
        if char == "{" and angle_depth:
            index = find_matching_brace(text, index) + 1
            continue
        if char in stop_set and not angle_depth:
            return char, index
        if char in IDENT_START:
            identifier, end = read_identifier(text, index)
            if identifier in stop_set and not angle_depth:
                return identifier, index
            index = end
        else:
            index += 1
    return "", index


def find_matching_delimiter(text: str, index: int, opening: str, closing: str) -> int:
    if index >= len(text) or text[index] != opening:
        raise ValueError(f"expected {opening}")
    depth = 0
    while index < len(text):
        index = skip_non_code(text, index)
        if index >= len(text):
            return len(text)
        if text[index] == opening:
            depth += 1
        elif text[index] == closing:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return len(text)


def find_matching_brace(text: str, index: int) -> int:
    return find_matching_delimiter(text, index, "{", "}")


def find_matching_paren(text: str, index: int) -> int:
    return find_matching_delimiter(text, index, "(", ")")


def line_number(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def classify_unsafe(text: str, start: int) -> tuple[str, UnsafeSpan | None, int]:
    after_keyword = start + len("unsafe")
    next_index = next_significant(text, after_keyword)
    next_char = text[next_index] if next_index < len(text) else ""

    if previous_significant(text, start) == "[" and next_char == "(":
        end = find_matching_paren(text, next_index)
        attribute_close = next_significant(text, end + 1)
        if attribute_close < len(text) and text[attribute_close] == "]":
            token, token_index = scan_until(text, attribute_close + 1, ("{", ";"))
            if token == "{":
                end = find_matching_brace(text, token_index)
            elif token == ";":
                end = token_index
            else:
                end = len(text)
        span = UnsafeSpan("attribute", line_number(text, start), line_number(text, end))
        return "attribute", span, after_keyword

    if next_char == "{":
        end = find_matching_brace(text, next_index)
        span = UnsafeSpan("block", line_number(text, start), line_number(text, end))
        return "block", span, after_keyword

    if next_char in IDENT_START:
        keyword, keyword_end = read_identifier(text, next_index)
        if keyword == "fn":
            token, token_index = scan_until(text, keyword_end, ("{", ";"))
            if token == ";":
                end = token_index
                span = UnsafeSpan("fn", line_number(text, start), line_number(text, end))
                return "fn", span, after_keyword
            if token == "{":
                end = find_matching_brace(text, token_index)
                span = UnsafeSpan("fn", line_number(text, start), line_number(text, end))
                return "fn", span, after_keyword
            span = UnsafeSpan("fn", line_number(text, start), line_number(text, len(text)))
            return "fn", span, after_keyword

        if keyword in ("impl", "trait"):
            token, token_index = scan_until(text, keyword_end, ("{", ";"))
            if token == "{":
                end = find_matching_brace(text, token_index)
            elif token == ";":
                end = token_index
            else:
                end = len(text)
            span = UnsafeSpan(keyword, line_number(text, start), line_number(text, end))
            return keyword, span, after_keyword

        if keyword == "extern":
            token, token_index = scan_until(text, keyword_end, ("fn", "{", ";"))
            if token == "fn":
                body_token, body_index = scan_until(text, token_index + 2, ("{", ";"))
                if body_token == ";":
                    end = body_index
                    span = UnsafeSpan("fn", line_number(text, start), line_number(text, end))
                    return "fn", span, after_keyword
                if body_token == "{":
                    end = find_matching_brace(text, body_index)
                    span = UnsafeSpan("fn", line_number(text, start), line_number(text, end))
                    return "fn", span, after_keyword
                span = UnsafeSpan("fn", line_number(text, start), line_number(text, len(text)))
                return "fn", span, after_keyword
            if token == "{":
                end = find_matching_brace(text, token_index)
                span = UnsafeSpan("extern", line_number(text, start), line_number(text, end))
                return "extern", span, after_keyword
            if token == ";":
                span = UnsafeSpan("extern", line_number(text, start), line_number(text, token_index))
                return "extern", span, after_keyword

    span = UnsafeSpan("other", line_number(text, start), line_number(text, start))
    return "other", span, after_keyword


def collect_code_lines(text: str) -> set[int]:
    code_lines: set[int] = set()
    index = 0
    while index < len(text):
        index = skip_non_code(text, index, code_lines)
        if index >= len(text):
            break
        code_lines.add(line_number(text, index))
        if text[index] in IDENT_START:
            _, index = read_identifier(text, index)
        else:
            index += 1
    return code_lines


def analyze_text(text: str, path: Path) -> FileResult:
    total_lines = len(text.splitlines())
    spans: list[UnsafeSpan] = []
    unsafe_lines: set[int] = set()
    code_lines = collect_code_lines(text)
    index = 0
    occurrences = 0

    while index < len(text):
        index = skip_non_code(text, index, code_lines)
        if index >= len(text):
            break

        if text[index] not in IDENT_START:
            index += 1
            continue

        identifier, identifier_end = read_identifier(text, index)
        if identifier != "unsafe":
            index = identifier_end
            continue

        occurrences += 1
        kind, span, next_index = classify_unsafe(text, index)
        index = next_index
        if span is not None:
            span = UnsafeSpan(span.kind, span.start_line, min(span.end_line, total_lines))
            spans.append(span)
            unsafe_lines.update(range(span.start_line, span.end_line + 1))

    return FileResult(
        path=path,
        total_lines=total_lines,
        spans=spans,
        occurrences=occurrences,
        unsafe_lines=unsafe_lines,
        code_lines=code_lines,
    )


def analyze_path(path: Path) -> list[FileResult]:
    if path.is_file():
        if path.suffix != ".rs":
            raise ValueError(f"not a Rust source file: {path}")
        files = [path]
    elif path.is_dir():
        files = sorted(path.rglob("*.rs"))
    else:
        raise ValueError(f"path does not exist: {path}")

    if not files:
        raise ValueError(f"no Rust files found under: {path}")
    return [analyze_text(file.read_text(encoding="utf-8-sig"), file) for file in files]


def aggregate(results: list[FileResult]) -> dict[str, int]:
    totals: Counter[str] = Counter()
    for result in results:
        totals["unsafe_loc"] += result.unsafe_loc
        totals["unsafe_physical_lines"] += result.unsafe_physical_lines
        totals["unsafe_occurrences"] += result.occurrences
        totals.update(result.construct_counts())
        totals["total_lines"] += result.total_lines
    return dict(totals)


def merge_lines(results: list[FileResult]) -> int:
    merged: set[tuple[Path, int]] = set()
    for result in results:
        merged.update((result.path, line) for line in result.unsafe_lines)
    return len(merged)


def merge_code_lines(results: list[FileResult]) -> int:
    merged: set[tuple[Path, int]] = set()
    for result in results:
        merged.update((result.path, line) for line in result.unsafe_lines & result.code_lines)
    return len(merged)


def cloc_statistics(results: list[FileResult]) -> dict[str, object]:
    if not shutil.which("cloc"):
        raise ValueError("cloc is required; install it with: sudo apt install cloc")
    with tempfile.TemporaryDirectory(prefix="asterinas-unsafe-cloc-") as directory:
        file_list = Path(directory) / "rust-files.txt"
        paths = sorted({str(result.path.resolve()) for result in results})
        if any("\n" in path or "\r" in path for path in paths):
            raise ValueError("cloc file lists do not support newlines in paths")
        file_list.write_text("".join(path + "\n" for path in paths))
        completed = subprocess.run(
            ["cloc", "--json", "--quiet", "--skip-uniqueness", "--include-lang=Rust",
             f"--list-file={file_list}"],
            capture_output=True, text=True, check=True,
        )
    report = json.loads(completed.stdout)
    rust = report.get("Rust", {})
    code_lines = int(rust.get("code", 0))
    unsafe_lines = merge_code_lines(results)
    return {
        "scope": "Rust source files only; identical numerator and denominator file lists",
        "version": report.get("header", {}).get("cloc_version"),
        "rust_code_lines": code_lines,
        "rust_comment_lines": int(rust.get("comment", 0)),
        "rust_blank_lines": int(rust.get("blank", 0)),
        "rust_files": int(rust.get("nFiles", 0)),
        "unsafe_related_code_lines": unsafe_lines,
        "unsafe_percentage": 100.0 * unsafe_lines / code_lines if code_lines else None,
    }


def print_cloc_summary(statistics: dict[str, object], output: TextIO) -> None:
    print(f"CLOC Rust code LOC={statistics['rust_code_lines']}, "
          f"comments={statistics['rust_comment_lines']}, "
          f"blank={statistics['rust_blank_lines']}, "
          f"files={statistics['rust_files']}", file=output)
    percentage = statistics["unsafe_percentage"]
    ratio = f"{percentage:.4f}%" if percentage is not None else "N/A (zero Rust code lines)"
    print(f"UNSAFE RATIO={statistics['unsafe_related_code_lines']} / "
          f"{statistics['rust_code_lines']} = {ratio}", file=output)
    print("SCOPE: Rust only; excludes assembly, C/C++, documents and configuration. "
          "Ratio measures unsafe-related regions, not unsafe operations.", file=output)


def print_detail(result: FileResult, output: TextIO) -> None:
    print(f"{result.path}:", file=output)
    for span in sorted(result.spans, key=lambda item: (item.start_line, item.end_line, item.kind)):
        print(f"  {span.kind:9} {span.start_line}-{span.end_line}", file=output)


def print_table(results: list[FileResult], top: int | None, output: TextIO) -> None:
    ordered = sorted(
        results,
        key=lambda result: (-result.unsafe_loc, -result.unsafe_physical_lines, str(result.path)),
    )
    if top is not None:
        ordered = ordered[:top]

    headers = ("LOC", "REGION", "OCC", "BLOCK", "FN", "IMPL", "TRAIT", "EXTERN", "ATTR", "FILE")
    rows: list[tuple[str, ...]] = []
    for result in ordered:
        counts = result.construct_counts()
        rows.append(
            (
                str(result.unsafe_loc),
                str(result.unsafe_physical_lines),
                str(result.occurrences),
                str(counts["block"]),
                str(counts["fn"]),
                str(counts["impl"]),
                str(counts["trait"]),
                str(counts["extern"]),
                str(counts["attribute"]),
                str(result.path),
            )
        )

    widths = [len(header) for header in headers]
    for row in rows:
        for index, value in enumerate(row):
            widths[index] = max(widths[index], len(value))
    line_format = "  ".join(f"{{:{width}}}" for width in widths)
    print(line_format.format(*headers), file=output)
    for row in rows:
        print(line_format.format(*row), file=output)

    totals = aggregate(results)
    summary = (
        f"TOTAL code LOC={merge_code_lines(results)}, "
        f"physical region={merge_lines(results)}, "
        f"occurrences={totals['unsafe_occurrences']}, "
        f"files={len(results)}"
    )
    print(summary, file=output)
    print(f"POLICY: {COUNTING_POLICY}; safe members included; no macro expansion or cfg filtering", file=output)


def result_json(result: FileResult) -> dict[str, object]:
    return {
        "path": str(result.path),
        "total_lines": result.total_lines,
        "unsafe_loc": result.unsafe_loc,
        "unsafe_physical_lines": result.unsafe_physical_lines,
        "unsafe_occurrences": result.occurrences,
        "constructs": dict(result.construct_counts()),
        "spans": [span.__dict__ for span in sorted(result.spans, key=lambda item: item.start_line)],
    }


def self_test() -> None:
    source = '''let text = "unsafe { not code }";
// unsafe { not code }
/* unsafe fn not code */
let lifetime: &'static str = r#"
unsafe { not code }
"#;
unsafe {
    unsafe { () }
}
unsafe fn defined() -> i32 { 1 }
unsafe trait Example { fn safe_method(); unsafe fn declared(); }
unsafe impl Example for Unit { }
#[unsafe(no_mangle)]
unsafe extern "C" { }
unsafe extern "C" fn entry() {}
'''
    result = analyze_text(source, Path("self-test.rs"))
    assert result.occurrences == 9
    assert result.unsafe_loc == 9
    assert result.unsafe_physical_lines == 9
    counts = result.construct_counts()
    assert counts == {
        "block": 2,
        "fn": 3,
        "trait": 1,
        "impl": 1,
        "attribute": 1,
        "extern": 1,
    }, counts
    print("self-test: PASS")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog="LOC excludes blank/comment-only lines; REGION includes them. "
               "Nested regions are merged. All source architectures and tests are included.",
    )
    parser.add_argument("paths", nargs="*", help="Rust files or directories (default: repository kernel/ and ostd/)")
    parser.add_argument("--detail", action="store_true", help="print unsafe line ranges")
    parser.add_argument("--json", action="store_true", help="print machine-readable results")
    parser.add_argument("--top", type=int, default=20, metavar="N", help="show only the first N files (default: 20; totals cover all files)")
    parser.add_argument("--self-test", action="store_true", help="run built-in lexical checks")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        return 0

    if args.top < 0:
        raise SystemExit("--top must be nonnegative")
    paths = [Path(path) for path in args.paths] if args.paths else default_paths()
    results: list[FileResult] = []
    seen: set[Path] = set()
    for path in paths:
        for result in analyze_path(path):
            canonical = result.path.resolve()
            if canonical not in seen:
                seen.add(canonical)
                results.append(result)

    try:
        statistics = cloc_statistics(results)
    except (ValueError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        raise SystemExit(f"cloc statistics failed: {error}") from error

    if args.json:
        payload = {
            "counting_policy": COUNTING_POLICY,
            "source_paths": [str(path.resolve()) for path in paths],
            "cloc": statistics,
            "totals": aggregate(results),
            "unsafe_code_lines": merge_code_lines(results),
            "unsafe_physical_lines": merge_lines(results),
            "files": [result_json(result) for result in results],
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0

    if args.detail:
        for result in sorted(results, key=lambda item: str(item.path)):
            print_detail(result, __import__("sys").stdout)
        print_cloc_summary(statistics, __import__("sys").stdout)
        return 0

    print_table(results, args.top, __import__("sys").stdout)
    print_cloc_summary(statistics, __import__("sys").stdout)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
