sudo  \
        /usr/local/bin/qemu-system-x86_64  \
        -cpu  \
        host  \
        -smp  \
        1  \
        -m  \
        4G  \
        --no-reboot  \
        -serial  \
        chardev:mux  \
        -monitor  \
        chardev:mux  \
        -machine  \
        q35,kernel-irqchip=split  \
        --enable-kvm  \
        -chardev  \
        stdio,id=mux,mux=on,signal=off,logfile=qemu.log  \
        -netdev  \
        user,id=net01,hostfwd=tcp::24256-:22,hostfwd=tcp::40083-:8080  \
        -object  \
        filter-dump,id=filter0,netdev=net01,file=virtio-net.pcap  \
        -device  \
        isa-debug-exit,iobase=0xf4,iosize=0x04  \
        -device  \
        virtio-blk-pci,bus=pcie.0,addr=0x6,drive=x0,serial=vext2,disable-legacy=on,disable-modern=off  \
        -device  \
        virtio-blk-pci,bus=pcie.0,addr=0x7,drive=x1,serial=vexfat,disable-legacy=on,disable-modern=off  \
        -device  \
        virtio-keyboard-pci,disable-legacy=on,disable-modern=off  \
        -device  \
        virtio-net-pci,netdev=net01,disable-legacy=on,disable-modern=off  \
        -device  \
        virtio-serial-pci,disable-legacy=on,disable-modern=off  \
        -device  \
        virtconsole,chardev=mux  \
        -drive  \
        if=none,format=raw,id=x0,file=../regression/build/ext2.img  \
        -drive  \
        if=none,format=raw,id=x1,file=../regression/build/exfat.img  \
        -drive  \
        if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd  \
        -drive  \
        if=pflash,format=raw,unit=1,file=/usr/share/OVMF/OVMF_VARS.fd  \
        -cdrom  \
        ./as/ubuntu.iso