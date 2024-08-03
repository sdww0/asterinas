#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

echo "Running Memory-related benchs"

bw_mem 256m frd
bw_mem 128m fcp
bw_mem 256m fwr
echo -n "lat_dram_page: " && lat_dram_page
par_mem
stream
lat_mem_rd 256

