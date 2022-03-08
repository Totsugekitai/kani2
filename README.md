# おまじない `command`

```sh
efibootmgr --create --disk /dev/sdX --part 1  --loader \\EFI\\BOOT\\kani2_loader.efi --label kani2 #エントリー追加
efibootmgr -o 0003,0004,0000,0005,0001,0002 #OS のエントリーが優先されるように変更
efibootmgr -n $(efibootmgr | grep kani2 | cut -c 5-8) #BitVisor を起動したい時のみ実行
```