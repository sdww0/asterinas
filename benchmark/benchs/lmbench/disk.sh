#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

echo "Running Disk-related benchs"

mkdir ext2

dd if=/dev/zero of=/ext2/test_file bs=1M count=512
bw_file_rd 512m io_only /ext2/test_file

# lat_fcntl

lat_fs /ext2
lmdd if=internal of=/ext2/test_file bs=1M count=512
cd /ext2 && lat_select file
