import util

pattern_dict = {
    "bw_mem 256m frd": ("268.44", 2),
    "bw_mem 128m fcp": ("134.22", 2),
    "bw_mem 256m fwr": ("268.44", 2),
    "lat_dram_page": ("lat_dram_page:", 1),
    "par_mem 67.1": ("67.108864", 2),
    "stream copy latency": ("STREAM copy latency:", 1),
    "stream copy bandwidth": ("STREAM copy bandwidth:", 1),
    "stream scale latency": ("STREAM scale latency:", 1),
    "stream scale bandwidth": ("STREAM scale bandwidth:", 1),
    "stream add latency": ("STREAM add latency:", 1),
    "stream add bandwidth": ("STREAM add bandwidth:", 1),
    "stream triad latency": ("STREAM triad latency:", 1),
    "stream triad bandwidth": ("STREAM triad bandwidth:", 1),
    "lat_mem_rd 256": ("256.00000", 2),
}

util.data_process(
    "qemu-lmbench-memory.sh.log",
    "lmbench-memory",
    "Running Memory-related benchs",
    pattern_dict,
)
