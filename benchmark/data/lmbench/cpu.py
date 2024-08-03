import util

pattern_dict = {
    "integer bit": ("integer bit: ", 1),
    "integer add": ("integer add: ", 1),
    "integer mul": ("integer mul: ", 1),
    "integer div": ("integer div: ", 1),
    "integer mod": ("integer mod: ", 1),
    "int64 bit": ("int64 bit: ", 1),
    "int64 add": ("int64 add: ", 1),
    "int64 mul": ("int64 mul: ", 1),
    "int64 div": ("int64 div: ", 1),
    "int64 mod": ("int64 mod: ", 1),
    "float add": ("float add: ", 1),
    "float mul": ("float mul: ", 1),
    "float div": ("float div: ", 1),
    "double add": ("double add: ", 1),
    "double mul": ("double mul: ", 1),
    "double div": ("double div: ", 1),
    "float bogomflops": ("float bogomflops: ", 1),
    "double bogomflops": ("double bogomflops: ", 1),
    "integer bit parallelism": ("integer bit parallelism: ", 1),
    "integer add parallelism": ("integer add parallelism: ", 1),
    "integer mul parallelism": ("integer mul parallelism: ", 1),
    "integer div parallelism": ("integer div parallelism: ", 1),
    "integer mod parallelism": ("integer mod parallelism: ", 1),
    "int64 bit parallelism": ("int64 bit parallelism: ", 1),
    "int64 add parallelism": ("int64 add parallelism: ", 1),
    "int64 mul parallelism": ("int64 mul parallelism: ", 1),
    "int64 div parallelism": ("int64 div parallelism: ", 1),
    "int64 mod parallelism": ("int64 mod parallelism: ", 1),
    "float add parallelism": ("float add parallelism: ", 1),
    "float mul parallelism": ("float mul parallelism: ", 1),
    "float div parallelism": ("float div parallelism: ", 1),
    "double add parallelism": ("double add parallelism: ", 1),
    "double mul parallelism": ("double mul parallelism: ", 1),
    "double div parallelism": ("double div parallelism: ", 1),
}

util.data_process(
    "qemu-lmbench-cpu.sh.log", "lmbench-cpu", "Running CPU-related benchs", pattern_dict
)
