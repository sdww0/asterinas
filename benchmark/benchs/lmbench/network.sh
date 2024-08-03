#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

echo "Running Network-related benchs"

# bw_tcp -s
# bw_tcp localhost

# lat_tcp -s
# lat_tcp localhost

# lat_udp -s
# lat_udp localhost

# lat_connect -s
# lat_connect localhost

# dd if=/dev/zero of=test_file bs=1M count=128
# lat_http localhost < test_file

# lat_select tcp