
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
cd ../../../

# Copy test srcipts
cp tools/unixbench-test.sh regression/build/initramfs/

make run AUTO_TEST=benchmark RELEASE=1
