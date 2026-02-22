# cymg

`cymg` は、画像にサイバーパンク風エフェクトを適用する CLI ツールです。

## Usage

```bash
cymg [options] <image-path>
```

- 入力: 画像ファイルを 1 つ指定
- 出力: 入力と同じディレクトリに
  - `<元ファイル名>.cyber.<元拡張子>`

例:

- 入力: `photo.jpg`
- 出力: `photo.cyber.jpg`

ヘルプ表示:

```bash
cymg --help
```

## サンプル出力

参照画像:

- https://avatars.githubusercontent.com/u/2846039?v=4

生成ファイル:

- 元画像: `docs/images/sample-source.png`
- 加工後: `docs/images/sample-cyber.png`

使用コマンド:

```bash
cymg --green --scanline 0.72 --scanline-step 2 --glitch-rate 0.24 --bloom 0.36 --noise 0.14 --contrast 1.22 docs/images/sample-source.png
```

| 元画像 | 加工後 |
| --- | --- |
| ![Source sample](docs/images/sample-source.png) | ![Cyber sample](docs/images/sample-cyber.png) |

## パラメータ詳細

### 色モード（同時指定不可）

`--red`, `--green`, `--blue`

- 色味の主方向を指定します。
- 同時に指定できるのは 1 つだけです。
- `--green` / `--blue` は強めの演出で、ハイライト補色処理も強く効きます。

### 全体強度

`--intensity <0.0-2.0>`（デフォルト: `1.0`）

- 処理後画像を元画像へどれだけ強く反映するか。
- `0.0`: 元画像に近い
- `1.0`: 標準
- `>1.0`: 誇張（白飛び・黒つぶれしやすくなる）

### グリッチ系

`--glitch-shift <0-128>`（デフォルト: `9`）

- RGB チャンネルのずらし最大ピクセル量。
- 値を上げるほど色ずれ・分離が強くなります。

`--glitch-rate <0.0-1.0>`（デフォルト: `0.18`）

- 強いグリッチ行の発生頻度。
- 値を上げるほど行単位の乱れが増えます。

### 走査線

`--scanline <0.0-1.0>`（デフォルト: `0.62`）

- 走査線の強さ。
- 値を上げると暗線/明線コントラストが強くなります。

`--scanline-step <1-32>`（デフォルト: `2`）

- 走査線パターンの縦方向間隔。
- 小さいほど高密度、大きいほど粗くなります。

### トーン・明暗

`--vignette <0.0-1.0>`（デフォルト: `0.45`）

- 周辺減光の強さ。
- 値を上げると中心へ視線を集めやすくなります。

`--brightness <-1.0-1.0>`（デフォルト: `0.0`）

- 全体の明るさシフト。
- 正で明るく、負で暗くなります。

`--contrast <0.0-3.0>`（デフォルト: `1.08`）

- 中間調を基準にしたコントラスト倍率。
- `1.0` がほぼニュートラル。
- `>1.0` で明暗差が強くなります。

`--saturation <0.0-3.0>`（デフォルト: `1.15`）

- 彩度倍率。
- `0.0` でほぼモノクロ。
- `>1.0` で色の鮮やかさが増します。

### 質感・光彩

`--noise <0.0-1.0>`（デフォルト: `0.08`）

- 粒状ノイズ量。
- 値を上げるほど荒さ・ざらつきが増えます。

`--bloom <0.0-1.0>`（デフォルト: `0.20`）

- 明部のにじみ（ネオン感）の強さ。
- 値を上げるほど発光の広がりが増えます。

### 再現性

`--seed <u64>`（デフォルト: `2077`）

- グリッチ/ノイズの疑似乱数シード。
- 同じ入力・同じオプション・同じ seed で再現可能。
- seed を変えるとバリエーションが変わります。

### ヘルプ

`-h`, `--help`

- 使い方と実行例を表示します。

## 実行例

基本:

```bash
cymg input.jpg
```

赤寄りグリッチ強調:

```bash
cymg --red --intensity 1.35 --glitch-shift 14 --glitch-rate 0.28 input.jpg
```

青寄り + 走査線強調:

```bash
cymg --blue --scanline 0.70 --scanline-step 2 --vignette 0.60 input.png
```

緑寄りシネマ調:

```bash
cymg --green --contrast 1.30 --saturation 1.35 --noise 0.20 --bloom 0.35 --seed 42 input.webp
```
