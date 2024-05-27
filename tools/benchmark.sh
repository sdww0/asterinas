#!/bin/sh
# ========================================================
# IoZone
echo "Running iozone"
iozone -s 4KB
iozone -s 8KB

cd ext2
iozone -s 4KB
iozone -s 8KB

cd ../exfat
iozone -s 4KB
iozone -s 8KB

cd ..

# =========================================================
# UnixBench
echo "Running Unixbench"
cd pgms
# echo "Running execl:" 
# UB_BINDIR=./ ./execl 10

echo "Running fstime in ramfs:" 
./fstime -c -t 10

echo "Running fstime in exfat:" 
cd ..
mkdir exfat
cd exfat
../pgms/fstime -w -t 10

echo "Running fstime in ext2:" 
cd ..
mkdir ext2
cd ext2
../pgms/fstime -w -t 10

cd ../pgms

echo "Running pipe:" 
./pipe 10

echo "Running register:" 
./register 10

echo "Running spawn:" 
./spawn 10

echo "Running context1:" 
./context1 10

echo "Running syscall:" 
./syscall 10

echo "Running dhry2:" 
./dhry2 10

echo "Running dhry2reg:" 
./dhry2reg 10

echo "Running arithoh:" 
./arithoh 10

echo "Running hanoi:" 
./hanoi 10

echo "Running short:" 
./short 10

echo "Running int:" 
./int 10

echo "Running long:" 
./long 10

echo "Running float:" 
./float 10

echo "Running double:" 
./double 10

echo "Running whetstone-double"
./whetstone-double

cd ..
# =========================================================
# lmbench
cd lmbench

echo ""
echo "Running lat_syscall -P 1 null"
./lat_syscall -P 1 null

echo ""
echo "Running lat_rand -P 1"
./lat_rand

echo ""
echo "Running lat_fs -P 1"
./lat_fs

echo ""
echo "Running lat_proc -P 1 fork"
./lat_proc -P 1 fork

echo ""
echo "Running lat_proc -P 1 exec"
./lat_proc -P 1 exec

echo ""
echo "Running lat_pipe -P 1"
./lat_pipe -P 1

echo ""
echo "Running lat_unix -P 1"
./lat_unix -P 1

echo ""
echo "Running lat_sig -P 1 install"
./lat_sig -P 1 install

echo ""
echo "Running lat_sig -P 1 catch"
./lat_sig -P 1 catch

echo ""
echo "Running lat_ctx -P 1 1024"
./lat_ctx -P 1 1024

