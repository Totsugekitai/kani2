#!/bin/bash

function setfile() {
    if [ $# != 2 ]; then
        echo "error invalid argument"
        exit 1
    fi
    drvletter=$1
    mntpoint=$2
    echo "drive letter: $drvletter"
    echo "mount point: $mntpoint"
    mount -t drvfs $drvletter $mntpoint
    cp target/x86_64-kani2-kernel/release/kani2_kernel.elf $mntpoint
    cp target/x86_64-unknown-uefi/release/kani2_loader.efi $mntpoint/EFI/BOOT/BOOTX64.EFI
    sync
    umount $mntpoint
}

setfile $1 $2
