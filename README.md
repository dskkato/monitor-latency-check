# モニター遅延計測用ソフト

現在、Windowsのみサポート（serial通信でCOMPortを直接使っているため）

注意：全画面で明滅するので、長い間画面を見ないほうがいいと思われる。

## 使用方法

```
USAGE:
    cargo run --release -- <SERIAL_PORT> <BAUD> <BG_COLOR> <STIM_COLOR> <MONITOR>
```

* SERIAL_PORT : シリアル通信用のポート。COM3, COM4など
* BAUD : ボーレート。USB-serialの場合、無視される。
* BG_COLOR : 背景色, [0, 1]の値を指定する
* STIM_COLOR : 刺激用の色, [0, 1]の値を指定する
* MONITOR : モニターインデックス。接続されているモニターが１つの場合は0を指定する。


## 使用例
以下の例では、シリアル通信はCOM5でUSB-serialを使うものとする。

### 黒背景に白色刺激
黒背景に白色を呈示した場合のモニター0のレスポンスを計測する場合。

```
cargo run --release -- COM5 115200 0.0 1.0 0
```
### 白背景に黒色刺激
```
cargo run --release -- COM5 115200 1.0 0.0 0
```