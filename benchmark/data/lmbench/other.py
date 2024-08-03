import util

pattern_dict = {
    "lat_ctx 18": ("18 ", 2),
    "lat_proc procedure": ("Procedure call:", 1),
    "lat_proc fork": ("Process fork+exit:", 1),
    "lat_proc exec": ("Process fork+execve:", 1),
    "lat_proc shell": ("Process fork+/bin/sh -c:", 1),
    "lat_sig install": ("Signal handler installation:", 1),
    "lat_sig catch": ("Signal handler overhead:", 1),
    "lat_syscall null": ("Simple syscall:", 1),
    "lat_syscall read": ("Simple read:", 1),
    "lat_syscall write": ("Simple write:", 1),
    "lat_syscall stat": ("Simple stat:", 1),
    "lat_syscall fstat": ("Simple fstat:", 1),
    "lat_syscall open": ("Simple open/close:", 1),
    "lat_rand drand48": ("drand48 latency:", 1),
    "lat_rand lrand48": ("lrand48 latency:", 1),
}

util.data_process(
    "qemu-lmbench-other.sh.log",
    "lmbench-other",
    "Running others benchs",
    pattern_dict,
)
