make build

# UnixBench
mkdir benchmark
cd benchmark

rm -rf byte-unixbench
git clone https://github.com/kdlucas/byte-unixbench
cd byte-unixbench/UnixBench

# Disable sse2
sed "s/-O3 -ffast-math/-O3 -ffast-math -mno-sse2/g" Makefile > Makefile.tmp
mv Makefile.tmp Makefile
make 
cp -r pgms/ ../../../regression/build/initramfs
cd ../../

# ========================================================
# iozone
wget https://www.iozone.org/src/current/iozone3_506.tar
tar -x -f iozone3_506.tar
cd iozone3_506/src/current
CFLAGS=-static make linux-AMD64
cp iozone ../../../../regression/build/initramfs/bin
cd ../../../

# Go to asterinas
cd ..
# Copy test srcipts
cp tools/benchmark.sh regression/build/initramfs/

make run AUTO_TEST=benchmark RELEASE=1
