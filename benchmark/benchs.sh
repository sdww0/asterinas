#!/bin/bash
source ./util.sh

rm -r result/log
mkdir result/log

# Get benchs
benchs=$(iterDir "benchs")
echo "Running benchs:"
for file in ${benchs[@]}; do
    if [[ $(is_excluded $file) == 1 ]]; then
        continue
    else
        echo "$file"
    fi
done

# Go to root directory
cd ..

# Run benchs
for file in ${benchs[@]}; do
    if [[ $(is_excluded $file) == 1 ]]; then
        continue
    fi
    $(make_initramfs)

    if [[ $OS_NAME == "aster" ]]; then
        make run RELEASE=1 AUTO_RUN_SCRIPTS=benchmark_entrypoint.sh
    elif [[ $OS_NAME == "linux" ]]; then
        rm ./benchmark/result/ext2.img
        dd if=/dev/zero of=./benchmark/result/ext2.img bs=2G count=1
        mke2fs ./benchmark/result/ext2.img

        rm ./benchmark/result/vda.img
        dd if=/dev/zero of=./benchmark/result/vda.img bs=1G count=1

        qemu_cmd="qemu-system-x86_64 \
            --no-reboot \
            -smp 1 \
            -m 8G \
            -machine q35,kernel-irqchip=split \
            -cpu host \
            --enable-kvm \
            -kernel ./benchmark/result/${LINUX_KERNEL} \
            -initrd ./test/build/initramfs.cpio.gz \
            -append 'console=ttyS0 rdinit=benchmark_entrypoint.sh kpti=0' \
            -nographic \
            -drive \
            if=none,format=raw,id=x0,file=./benchmark/result/vda.img \
            -drive \
            if=none,format=raw,id=x1,file=./benchmark/result/ext2.img \
            -device \
            virtio-blk-pci,bus=pcie.0,addr=0x6,drive=x0,serial=vda,disable-legacy=on,disable-modern=off \
            -device \
            virtio-blk-pci,bus=pcie.0,addr=0x8,drive=x1,serial=vext2,disable-legacy=on,disable-modern=off \
            2>&1 | tee qemu.log"

        eval "$qemu_cmd"
    else
        exit 1
    fi
    # remove `benchs/`
    logFile=${file#*/}
    # replace / with -
    logFile=${logFile////-}
    mv qemu.log benchmark/result/log/$OS_NAME-qemu-$logFile.log
done
