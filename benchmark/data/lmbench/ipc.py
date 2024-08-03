import util

pattern_dict = {
    "bw_pipe": ("Pipe bandwidth:", 1),
    "lat_pipe": ("Pipe latency:", 1),
    "bw_unix": ("AF_UNIX sock stream bandwidth:", 1),
    "lat_unix": ("AF_UNIX sock stream latency:", 1),
}

util.data_process(
    "qemu-lmbench-ipc.sh.log",
    "lmbench-ipc",
    "Running IPC-related benchs",
    pattern_dict,
)
