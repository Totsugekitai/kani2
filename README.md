## memo

### ビルドのやり方

各ディレクトリで `cargo build` もしくは `make build`

以下の環境変数でスイッチ(組み合わせ可能)

* `RELEASE=1` - リリースビルド
* `QEMU=1` - QEMU用にビルド

## tips

### USBメモリの作り方

1. `win+R` で `diskpart` を起動
2. 次のコマンドを入力

```
DISKPART> list disk
(ディスク一覧が出る)
DISKPART> select disk 2 # 個々の番号は上記コマンドで出たディスクに合わせる
DISKPART> clean
DISKPART> convert gpt
DISKPART> create partition primary
DISKPART> select partition 1
DISKPART> format fs=fat32
DISKPART> assign
DISKPART> exit
```

### おまじない `command`

```sh
efibootmgr --create --disk /dev/sdX --part 1  --loader \\EFI\\BOOT\\kani2_loader.efi --label kani2 #エントリー追加
efibootmgr -o 0003,0004,0000,0005,0001,0002 #OS のエントリーが優先されるように変更
efibootmgr -n $(efibootmgr | grep kani2 | cut -c 5-8) #BitVisor を起動したい時のみ実行
```