import numpy as np
import pandas as pd
import time
import os


def is_float(s):
    try:
        float(s)
        return True
    except:
        return False


def data_process(log_file, bench_name, start_string, pattern_dict):
    os.chdir("result/log")
    result = {}
    with open(os.getenv("OS_NAME") + "-" + log_file, "r") as file:
        result = parse_log(file, start_string, pattern_dict)

    # Convert to csv
    os.chdir("../data")
    df = out_csv(
        pd.DataFrame(
            result, index=[time.strftime("%Y-%m-%d %H:%M:%S", time.localtime())]
        ).T,
        os.getenv("OS_NAME") + "-" + bench_name + ".csv",
    )

    os.chdir("../fig")
    fig = df.T.cumsum().plot()
    fig.figure.savefig(bench_name + ".png")


"""
    pattern_dict: {'result_name': ('result_line_start_string', number index start with 1)}
    Example:
    
    In 0.111 seconds, foo result: 0.1232
    then:
    results_dict: {'result1': ('In ', 2)}

"""


def parse_log(file, start_string, pattern_dict):
    line = file.readline()
    results = {}
    while line:
        if line.startswith(start_string):
            break
        line = file.readline()
    for results_name, pattern in pattern_dict.items():
        while line:
            if line.startswith(pattern[0]):
                break
            line = file.readline()

        strs = line.strip().split(" ")
        for index in range(len(strs)):
            string = strs.pop(0)
            strs.extend(string.split("\t"))

        index = pattern[1]
        for string in strs:
            if is_float(string):
                index -= 1
                if index == 0:
                    results[results_name] = float(string)
                    break
    return results


def out_csv(data_frame, csv_name):
    if not os.path.exists(csv_name):
        data_frame.to_csv(csv_name)
        return data_frame
    else:
        old_df = pd.read_csv(csv_name, index_col=0)
        out_df = pd.concat([old_df, data_frame], axis=1)
        out_df.to_csv(csv_name)
        return out_df
