#!/bin/sh
mount -t devtmpfs devtmpfs /dev
mount -t proc /proc proc/
ip link set lo up
depmod
modprobe failover
modprobe net_failover
modprobe virtio_net
modprobe virtio_blk
ip link set eth0 up
mkfs.ext2 -F /dev/vda
mount -t ext2 /dev/vda /ext2

echo "Running lmbench-ctx"
chmod +x /benchmark/lmbench-ctx/run.sh
/benchmark/lmbench-ctx/run.sh

poweroff -f
