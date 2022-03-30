# モニター遅延計測用ソフトの補助ツール

XiaoにUSBシリアル通信して、GPIOを制御する。

受け付けるコマンドは1バイトのコマンドで、0x00がlow, 0x01がhighのみ。

趣味でRustを用いて書いているが、別に何を使って実装してもよい。

## ビルド環境の準備

Rustがインストール済みであるとする。また、以下の動作確認はWindows 11で実施した。

詳細は書籍「組込みRust」を参考にしてほしい。

### クロスビルドツールチェインのインストール

Wio terminal用

```
$ rustup target add thumbv6m-none-eabi
```

### 開発補助ツールのインストール

```
$ cargo install hf2-cli cargo-hf2
```

## ビルド方法

前提：XiaoでUSB接続されており、ブートローダー書き込みモードであること。

```
$ cargo run --release
```