import util

pattern_dict = {
    "bw_file_rd 512m": ("536.87 ", 2),
    "lat_fs 0k-cps": ("0k", 2),
    "lat_fs 0k-rps": ("0k", 3),
    "lat_fs 1k-cps": ("1k", 2),
    "lat_fs 1k-rps": ("1k", 3),
    "lat_fs 4k-cps": ("4k", 2),
    "lat_fs 4k-rps": ("4k", 3),
    "lat_fs 10k-cps": ("10k", 2),
    "lat_fs 10k-rps": ("10k", 3),
    "lmdd 512": ("512.0000 MB in", 3),
    "lat_select": ("Select on ", 2),
}

util.data_process(
    "qemu-lmbench-disk.sh.log",
    "lmbench-disk",
    "Running Disk-related benchs",
    pattern_dict,
)
