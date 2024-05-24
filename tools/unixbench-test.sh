#!/bin/sh
# The shell in Asterinas do not support list, so we need to sepearte the pgms :(
# pgms=(pipe register spawn context1 fstime syscall dhry2 dhry2reg execl arithoh hanoi short int long float double)
cd pgms
# for pgm in ${pgms[@]}
 # do 
 # echo "Running "$pgm"(10 seconds):" 
 # ./$pgm 10
 # done

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

# Spawn will cause out of memory, it need 4GB memory to complete this test script
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

