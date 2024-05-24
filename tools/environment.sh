rm -rf tmp
mkdir tmp
cd tmp

# Copy from Dockerfile.ubuntu22.04, but modify some command

sudo apt update && sudo apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git-core \
    gnupg \
    libssl-dev \
    python3-pip \
    python-is-python3 \
    wget

sudo apt install -y automake \
    libtool \
    pkg-config

wget https://github.com/akopytov/sysbench/archive/1.0.20.tar.gz \
    && tar -zxvf 1.0.20.tar.gz \
    && rm 1.0.20.tar.gz
git clone https://github.com/nicktehrany/membench.git
git clone https://github.com/esnet/iperf.git

cd sysbench-1.0.20

./autogen.sh \
    && ./configure --without-mysql --prefix=/usr/local/benchmark/sysbench \
    && make -j \
    && sudo make install

cd ../membench
make -j
sudo mkdir /usr/local/benchmark/membench 
sudo cp membench /usr/local/benchmark/membench/

cd ../iperf
./configure --prefix=/usr/local/benchmark/iperf \
    && make -j \
    && sudo make install

cd ..
rm -rf sysbench-1.0.20 \
    membench \
    iperf

# I am not sure whether it is really bazel
sudo apt install bazel-bootstrap
# lack of syscall_test


# QEMU

sudo apt update && sudo apt-get install -y --no-install-recommends \
    libgcrypt-dev   `# optional build dependency` \
    libglib2.0-dev  `# build dependency` \
    libpixman-1-dev `# build dependency` \
    libusb-dev      `# optional build dependency` \
    meson \
    ninja-build

wget -O qemu.tar.xz https://download.qemu.org/qemu-8.2.1.tar.xz \
    && mkdir qemu \
    && tar xf qemu.tar.xz --strip-components=1 -C ./qemu \
    && rm qemu.tar.xz

cd qemu
./configure --target-list=x86_64-softmmu --enable-slirp \
    && make -j \
    && sudo make install

cd ..

# Grub
sudo apt update && sudo apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    autopoint \
    bison \
    flex \
    gawk \
    gettext \
    libfreetype6-dev \
    pkg-config

wget -O grub.tar.xz https://ftp.gnu.org/gnu/grub/grub-2.12.tar.xz \
    && mkdir grub \
    && tar xf grub.tar.xz --strip-components=1 -C ./grub \
    && rm grub.tar.xz

wget -O unifont.pcf.gz https://unifoundry.com/pub/unifont/unifont-15.1.04/font-builds/unifont-15.1.04.pcf.gz \
    && sudo mkdir -pv /usr/share/fonts/unifont \
    && gunzip -c unifont.pcf.gz > temp \
    && mv temp /usr/share/fonts/unifont/unifont.pcf \
    && rm unifont.pcf.gz \
    && rm temp

cd grub

echo depends bli part_gpt > grub-core/extra_deps.lst \
    && ./configure \
        --target=x86_64 \
        --disable-efiemu \
        --with-platform=efi \
        --enable-grub-mkfont \
        --prefix=/usr/local/grub \
        --disable-werror \
    && make -j \
    && sudo make install

# Busybox
cd ..

wget -O busybox.tar.bz2 https://busybox.net/downloads/busybox-1.35.0.tar.bz2 
mkdir busybox 
tar xf busybox.tar.bz2 --strip-components=1 -C ./busybox 
rm busybox.tar.bz2
cd busybox
make defconfig \
    && sed -i "s/# CONFIG_STATIC is not set/CONFIG_STATIC=y/g" .config \
    && sed -i "s/# CONFIG_FEATURE_SH_STANDALONE is not set/CONFIG_FEATURE_SH_STANDALONE=y/g" .config \
    && make -j

# We assume Rust is installed

# Asterinas dependence
sudo apt update && sudo apt-get install -y --no-install-recommends \
    clang-format       `# formatting regression tests` \
    cpio \
    cpuid \
    exfatprogs \
    file \
    gdb \
    grub-efi-amd64 \
    grub-efi-amd64-bin \
    grub-efi-amd64-dbg \
    libpixman-1-dev     `# running dependency for QEMU` \
    mtools              `# used by grub-mkrescue` \
    net-tools \
    openssh-server \
    ovmf                `# provide an alternative stable firmware`\
    pkg-config \
    strace \
    sudo \
    unzip \
    vim \
    xorriso \
    zip

cargo install mdbook

sudo rm -rf tmp
