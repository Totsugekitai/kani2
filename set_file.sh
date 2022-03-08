rm /boot/efi/EFI/BOOT/kani2_loader.efi /boot/efi/kani2_kernel.elf
cp target/x86_64-unknown-uefi/release/kani2_loader.efi /boot/efi/EFI/BOOT
cp target/x86_64-kani2-kernel/release/kani2_kernel.elf /boot/efi
efibootmgr -n $(efibootmgr | grep kani2 | cut -c 5-8)