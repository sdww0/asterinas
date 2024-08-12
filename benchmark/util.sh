#!/bin/bash

LINUX_KERNEL="vmlinuz-6.6.42-060642-generic"
EXCLUDED_BENCHS=(util cpu memory ipc other network disk fio)

function iterDir() {
    files=()
    for file in $(ls $1); do
        if [ -d $1"/"$file ]; then
            files+=($(iterDir $1"/"$file))
        else
            files+=($1"/"$file)
        fi
    done
    for file in ${files[@]}; do
        echo $file
    done
}

function copy_vmlinuz() {
    # Copy vmlinuz
    sudo cp "/boot/${LINUX_KERNEL}" ./result
    sudo chmod 777 ./result/${LINUX_KERNEL}
}

function generate_entrypoint_script() {
    local benchmark="$1"
    if [[ $OS_NAME == "aster" ]]; then
        local init_script=$(
            cat <<EOF
#!/bin/sh

chmod +x ${benchmark}
${benchmark}

poweroff -f
EOF
        )
    elif [[ $OS_NAME == "linux" ]]; then
        local init_script=$(
            cat <<EOF
#!/bin/sh
mount -t devtmpfs devtmpfs /dev
mount -t ext2 /dev/vdb /ext2

chmod +x ${benchmark}
${benchmark}

poweroff -f
EOF
        )
    else
        exit 1
    fi

    echo "$init_script"
}

function is_excluded() {
    local file=$1
    for excluded_file in ${EXCLUDED_BENCHS[@]}; do
        if [[ $file =~ $excluded_file ]]; then
            echo "1"
        fi
    done
}

function make_initramfs() {
    # Clean initramfs
    cd test && make clean && make
    cd ..

    # Copy benchs to the initramfs
    cp -r benchmark/benchs ./test/build/initramfs

    # Entrypoint script for initramfs
    initramfs_entrypoint_script="./test/build/initramfs/benchmark_entrypoint.sh"
    generate_entrypoint_script "${file}" >"${initramfs_entrypoint_script}"
    chmod +x "${initramfs_entrypoint_script}"

    # Copy benchmark to bin so that we can easily call these benchmarks.
    cp ./test/build/initramfs/benchmark/bin/lmbench/* test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/unixbench/* test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/iperf3 test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/iozone test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/fio test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/sysbench test/build/initramfs/bin
    cp ./test/build/initramfs/benchmark/bin/membench test/build/initramfs/bin

    make initramfs
}

# qemu-system-x86_64 --no-reboot -smp 1 -m 8G -machine q35,kernel-irqchip=split -cpu host --enable-kvm -kernel ./benchmark/result/vmlinuz-6.5.0-44-generic -initrd ./test/build/initramfs.cpio.gz -append 'console=ttyS0 rdinit=/bin/sh' -nographic -drive if=none,format=raw,id=x0,file=./benchmark/result/ext2.img -device virtio-blk-pci,bus=pcie.0,addr=0x6,drive=x0,serial=vext2,disable-legacy=on,disable-modern=off
