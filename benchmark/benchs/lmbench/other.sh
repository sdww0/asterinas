#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

echo "Running others benchs"

mkdir tmp
cp /benchmark/bin/lmbench/hello /tmp/hello

lat_ctx 18
lat_proc procedure
lat_proc fork
lat_proc exec
lat_proc shell

# lat_sem

lat_sig install
lat_sig catch
# lat_sig prot enough

# dd if=/dev/zero of=test_file bs=1M count=256
# bw_mmap_rd 256m mmap_only test_file
# /lat_mmap 256m test_file
# lat_pagefault test_file

touch test_file

lat_syscall null
lat_syscall read
lat_syscall write
lat_syscall stat test_file
lat_syscall fstat test_file
lat_syscall open test_file

lat_rand