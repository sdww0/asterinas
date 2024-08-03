#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

echo "Running IPC-related benchs"

bw_pipe
lat_pipe
bw_unix
lat_unix

# lat_unix_connect -s
# lat_unix_connect
# lat_fifo