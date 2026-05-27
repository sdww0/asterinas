// SPDX-License-Identifier: MPL-2.0

//! System call dispatch in the RISC-V architecture.

#[path = "./generic.rs"]
mod generic;

use ostd::{arch::flush_local_instruction_cache, mm::VmIo};

use super::SyscallReturn;
use crate::prelude::*;

const SYS_RISCV_FLUSH_ICACHE_LOCAL: usize = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod)]
struct RiscvHwprobePair {
    key: i64,
    value: u64,
}

fn sys_riscv_hwprobe(
    pairs_addr: Vaddr,
    pair_count: usize,
    _cpu_count: usize,
    _cpus_addr: Vaddr,
    flags: usize,
    ctx: &Context,
) -> Result<SyscallReturn> {
    if flags != 0 {
        return_errno_with_message!(Errno::EINVAL, "riscv_hwprobe flags are invalid");
    }

    let user_space = ctx.user_space();
    for pair_idx in 0..pair_count {
        let pair_addr = pairs_addr + pair_idx * size_of::<RiscvHwprobePair>();
        let mut pair = user_space.read_val::<RiscvHwprobePair>(pair_addr)?;

        // Asterinas does not expose RISC-V hwprobe keys yet. Linux reports an
        // unknown key by setting the key to -1 and clearing the value.
        pair.key = -1;
        pair.value = 0;
        user_space.write_val(pair_addr, &pair)?;
    }

    Ok(SyscallReturn::Return(0))
}

fn sys_riscv_flush_icache(
    _start: Vaddr,
    _end: Vaddr,
    flags: usize,
    _ctx: &Context,
) -> Result<SyscallReturn> {
    if flags & !SYS_RISCV_FLUSH_ICACHE_LOCAL != 0 {
        return_errno_with_message!(Errno::EINVAL, "riscv_flush_icache flags are invalid");
    }

    // The Linux ABI may request a process-wide flush. Zion currently runs this
    // Asterinas path with a single guest hart, so a local fence is sufficient.
    flush_local_instruction_cache();

    Ok(SyscallReturn::Return(0))
}

generic::define_syscalls_with_generic_syscall_table! {
    SYS_RISCV_HWPROBE = 258      => sys_riscv_hwprobe(args[..5]);
    SYS_RISCV_FLUSH_ICACHE = 259 => sys_riscv_flush_icache(args[..3]);
}
