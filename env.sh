#! /bin/bash

sudo apt-get install -y --no-install-recommends \ 
    build-essential \ 
    ca-certificates \ 
    curl \ 
    git-core \ 
    gnupg \ 
    libssl-dev \ 
    jq \
    python3-pip \ 
    python-is-python3 \ 
    wget


sudo apt install -y automake \ 
    libtool \ 
    pkg-config \
    libntirpc-dev 


mkdir tmp
cd tmp

wget https://github.com/akopytov/sysbench/archive/1.0.20.tar.gz
tar -zxvf 1.0.20.tar.gz
rm 1.0.20.tar.gz

git clone https://github.com/nicktehrany/membench.git
git clone https://github.com/esnet/iperf.git
git clone https://github.com/kdlucas/byte-unixbench.git
git clone https://github.com/asterinas/lmbench.git
wget https://www.iozone.org/src/current/iozone3_506.tar
tar -x -f iozone3_506.tar
git clone -b fio-3.37 https://github.com/axboe/fio.git

sudo mkdir /usr/local/benchmark

# Build sysbench
cd sysbench-1.0.20
./autogen.sh 
./configure --without-mysql --prefix=/usr/local/benchmark/sysbench 
make -j 
sudo make install

# Build membench
cd ../membench
make -j 
mkdir /usr/local/benchmark/membench 
sudo cp membench /usr/local/benchmark/membench/


# Build iperf
cd ../iperf
./configure --prefix=/usr/local/benchmark/iperf 
make -j 
sudo make install

# Build lmbench
cd ../lmbench
make -j 
sudo mv bin/x86_64-linux-gnu /usr/local/benchmark/lmbench 
sudo rm /usr/local/benchmark/lmbench/*.o 
sudo rm /usr/local/benchmark/lmbench/*.a 

# Build unixbench
cd ../byte-unixbench/UnixBench
make UB_GCC_OPTIONS=-mno-sse2 -j && sudo mv pgms /usr/local/benchmark/unixbench 

# Build iozone
cd ../iozone3_506/src/current
CFLAGS=-static make linux-AMD64 
sudo cp iozone /usr/local/benchmark/

# Build fio
cd ../fio
./configure --disable-shm --prefix=/usr/local/benchmark/fio 

    # Remove this when we support syscall timerfd_create and fadvise
sed -i -e '/#define CONFIG_HAVE_TIMERFD_CREATE/d' -e '/#define CONFIG_POSIX_FADVISE/d' config-host.h 
make -j 
sudomake install

cd ..

sudo apt-get install -y --no-install-recommends libgcrypt-dev libglib2.0-dev libpixman-1-dev libusb-dev meson ninja-build

wget -O qemu.tar.xz https://download.qemu.org/qemu-8.2.1.tar.xz 
mkdir qemu 
tar xf qemu.tar.xz --strip-components=1 -C ./qemu 
rm qemu.tar.xz

cd qemu

./configure --target-list=x86_64-softmmu --prefix=/usr/local/qemu --enable-slirp 
make -j 
sudo make install

cd ..

# grub
sudo apt-get install -y --no-install-recommends autoconf automake autopoint bison flex gawk gettext libfreetype6-dev pkg-config

wget -O grub.tar.xz https://ftp.gnu.org/gnu/grub/grub-2.12.tar.xz 
mkdir grub 
tar xf grub.tar.xz --strip-components=1 -C ./grub 
rm grub.tar.xz
# Fetch and install the Unicode font data for grub.
wget -O unifont.pcf.gz https://unifoundry.com/pub/unifont/unifont-15.1.04/font-builds/unifont-15.1.04.pcf.gz 
sudo mkdir -pv /usr/share/fonts/unifont 
sudo gunzip -c unifont.pcf.gz > /usr/share/fonts/unifont/unifont.pcf 
rm unifont.pcf.gz

cd ./grub

echo depends bli part_gpt > grub-core/extra_deps.lst 
./configure --target=x86_64 --disable-efiemu --with-platform=efi --enable-grub-mkfont --prefix=/usr/local/grub --disable-werror 
make -j 
sudo make install

cd ..

wget -O busybox.tar.bz2 https://busybox.net/downloads/busybox-1.35.0.tar.bz2 
mkdir /root/busybox 
tar xf busybox.tar.bz2 --strip-components=1 -C /root/busybox 
rm busybox.tar.bz2

cd busybox

make defconfig 
sed -i "s/# CONFIG_STATIC is not set/CONFIG_STATIC=y/g" .config 
sed -i "s/# CONFIG_FEATURE_SH_STANDALONE is not set/CONFIG_FEATURE_SH_STANDALONE=y/g" .config 
sudo make -j

cd ..


sudo apt-get install -y --no-install-recommends    clang-format      
sudo apt-get install -y --no-install-recommends    cpio 
sudo apt-get install -y --no-install-recommends    cpuid 
sudo apt-get install -y --no-install-recommends    exfatprogs 
sudo apt-get install -y --no-install-recommends    file 
sudo apt-get install -y --no-install-recommends    gdb 
sudo apt-get install -y --no-install-recommends    grub-efi-amd64 
sudo apt-get install -y --no-install-recommends    grub-efi-amd64-bin 
sudo apt-get install -y --no-install-recommends    grub-efi-amd64-dbg 
sudo apt-get install -y --no-install-recommends    ovmf                
sudo apt-get install -y --no-install-recommends    libpixman-1-dev    
sudo apt-get install -y --no-install-recommends    mtools           
sudo apt-get install -y --no-install-recommends    net-tools 
sudo apt-get install -y --no-install-recommends    openssh-server 
sudo apt-get install -y --no-install-recommends    pkg-config 
sudo apt-get install -y --no-install-recommends    strace 
sudo apt-get install -y --no-install-recommends    sudo 
sudo apt-get install -y --no-install-recommends    unzip 
sudo apt-get install -y --no-install-recommends    vim 
sudo apt-get install -y --no-install-recommends    xorriso 
sudo apt-get install -y --no-install-recommends    zip