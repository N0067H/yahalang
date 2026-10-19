# Yahalang

[English](README.md) | [한국어](README_KO.md) | [日本語](README_JP.md)

Yahalangは『ちいかわ』のうさぎの発話をもとにした、スタック型の難解プログラミング言語です。
英語・韓国語・日本語の命令表記、符号付き64ビット整数、変更可能な整数配列をサポートします。

## ビルドと実行

RustとCargoをインストールし、リポジトリのルートで実行します。

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang en
```

出力:

```text
Hello, world!
```

REPLを起動するには、次のコマンドを実行します。

```sh
cargo run -- repl --lang en
```

```text
yaha> ura yahayaha i
2
yaha> :q
```

REPLでは各入力を独立したプログラムとして実行します。
スタックや配列は次の入力に引き継がれません。
終了するには `:q` または `:quit` を入力します。

ソースファイルは全体が1つのプログラムとして実行され、行をまたいで状態が保持されます。

以下の例では英語表記を使用します。
明示的な `--lang en` の指定は、保存された既定の言語より優先されます。

## 数値とスタック

`ura` は0から数値を作ります。
続けて書く `yaha` ごとに値が1増えます。

| コード | 値 |
| --- | --- |
| `ura` | 0 |
| `ura yaha` | 1 |
| `ura yahayaha` | 2 |
| `ura yahayahayaha` | 3 |

1つの数値を表す `yaha` は空白を入れずに続けて書きます。
空白で数値のカウントが終了します。
例えば、`ura yaha yaha` は2ではなく、1をスタックに積んでから加算を実行するコードです。

次のコードは1と2を積んで加算し、`3` を出力します。

```text
ura yaha ura yahayaha yaha i
```

二項演算はスタックの先頭から2つの値を取り出し、結果を積みます。
減算・除算・剰余では、下にある値が左オペランドです。

オーバーフロー、ゼロ除算、スタックの値の不足は実行時エラーになります。

## 命令

| 英語 | 韓国語 | 日本語 | 動作 |
| --- | --- | --- | --- |
| `ura` | `우라` | `ウラ` | 整数リテラルの開始 |
| `yaha` / `yaha!` | `야하` / `야하!` | `ヤハ` / `ヤハ!` | 加算 / 減算 |
| `huung` | `후응` | `フゥン` | 乗算 |
| `hueung` | `후으응` | `フウン` | 整数除算 |
| `jureuri` | `쥬르리` | `ジュルリ` | 剰余 |
| `fuun` | `후웅` | `フン` | 符号反転 |
| `puru` / `puru!` | `푸루` / `푸루!` | `プル` / `プル!` | 先頭の値を複製 / 破棄 |
| `haa?` | `하아?` | `ハァ?` | 取り出した値が0なら次の命令を実行 |
| `puruya` / `puruya!` | `푸루야` / `푸루야!` | `プルャ` / `プルャ!` | 指定行へジャンプ / 終了 |
| `ui` | `우이` | `ウイ` | 整数入力 |
| `i` | `이` | `イ` | 整数出力 |
| `ua` | `우아` | `ウア` | コードポイントをUnicode文字として出力 |

`haa?` は条件値を取り出し、同じ行の次の命令1つを条件付きで実行します。
条件のネストは最大128段階です。

ジャンプ先の行番号は1から始まり、空行も数えます。

`i` と `ua` はオペランドを取り出して出力し、末尾に改行を追加しません。
REPLでは出力と次のプロンプトを区別するため、必要に応じて改行を追加します。

文字列リテラル、文字リテラル、コメント構文はサポートしていません。

## 整数入力

`ui` は1行全体を読み取ります。
各行には整数を1つだけ入力します。

```sh
cargo run -- run examples/add_two_numbers.yaha --lang en
```

`10` を入力してEnterを押し、続いて `2` を入力してEnterを押すと、`12` が出力されます。

`10 2` を1行で入力するとエラーになります。

パイプで入力を渡すこともできます。

```sh
printf '10\n2\n' | cargo run -- run examples/add_two_numbers.yaha --lang en
```

## 配列

角括弧の中に整数リテラルをカンマで区切って書きます。

```text
[ura, ura yaha, ura yahayaha] ura yaha @ i
```

配列 `[0, 1, 2]` を作り、インデックス1の要素を読み取って `1` を出力します。
インデックスは0から始まります。

空の配列は `[]` で、配列のネストはサポートしていません。

| 命令 | オペランドの順序: スタックの下から上へ | 結果 |
| --- | --- | --- |
| `#` | 配列 | 長さを積む |
| `@` | 配列、インデックス | 指定した整数を積む |
| `=` | 配列、インデックス、値 | 指定した要素を更新 |

これらの命令はすべてオペランドをスタックから取り出します。
配列を再利用するには、`puru` で参照を複製します。
複製した参照は同じ配列を共有するため、更新内容も共有されます。

不正な型やインデックスは実行時エラーになります。

## 言語設定

`run` と `repl` では `--lang en`、`--lang ko`、`--lang ja` を指定できます。
1つのプログラムでは同じ言語の表記を使用します。

既定の言語を保存するには、次のコマンドを実行します。

```sh
cargo run -- set lang ko
```

設定はTOMLで保存されます。

```toml
lang = 'ko'
```

| OS | 設定ファイル |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/yahalang/config.toml`、未設定の場合は `~/.config/yahalang/config.toml` |
| macOS | `~/Library/Application Support/yahalang/config.toml` |
| Windows | `%APPDATA%/yahalang/config.toml` |

実行時の `--lang` は保存された設定より優先されます。
どちらも指定されていない場合は英語を使用します。

## エラーメッセージ

既定のエラー形式 `usagi` では、`haa?`、`haa? haa?`、`ui` などの発話を表示します。

原因を確認するには `friendly` を指定します。

```sh
cargo run -- repl --lang en --error-style friendly
```

空のスタックで `yaha` を入力すると、次のエラーが表示されます。

```text
runtime error at line 1, instruction 1: stack underflow
```

`--error-style usagi|friendly` は `run` と `repl` の両方で使用できます。

## CIとリリース

GitHub Actionsでフォーマットの検査とreleaseプロファイルでのテストを実行し、
Linux x86-64、Windows x86-64、macOS Apple Silicon（aarch64）向けのバイナリをビルドします。
PR、`master`へのプッシュ、バージョンタグのプッシュ、手動実行でワークフローを開始できます。
ビルドしたアーカイブはワークフローのArtifactsからダウンロードできます。

リリースするには、`Cargo.toml`と`Cargo.lock`のバージョンを更新し、変更をコミットしてプッシュします。
続いて、バージョンに一致するタグをプッシュします。

```sh
git tag v0.1.0
git push origin v0.1.0
```

`0.1.0`は公開するパッケージのバージョンに置き換えてください。
タグは`v`の後に`Cargo.toml`のバージョンを付けた値と一致する必要があります。
すべてのビルドが成功すると、各プラットフォームのアーカイブと`SHA256SUMS`を添付したGitHub Releaseを作成します。
`v0.2.0-beta.1`などのプレリリースバージョンのタグは、プレリリースを作成します。
手動実行ではリリースを公開せず、ビルドのアーティファクトのみを作成します。

## サンプル

| ファイル | 動作 |
| --- | --- |
| [hello_world.yaha](examples/hello_world.yaha) | `Hello, world!` を出力 |
| [add_two_numbers.yaha](examples/add_two_numbers.yaha) | 別々の行で2つの整数を読み取り、合計を出力 |
| [integer_echo.yaha](examples/integer_echo.yaha) | 整数を1つ読み取り、そのまま出力 |
| [array_access.yaha](examples/array_access.yaha) | 配列の要素を更新して読み取る |
| [truth_machine.yaha](examples/truth_machine.yaha) | `0` を1回出力して終了、または `1` を繰り返し出力 |

`truth_machine.yaha` には `0` または `1` を入力します。
繰り返し出力を停止するにはCtrl-Cを押します。

言語設計の詳細は [DESIGN.md](DESIGN.md) を参照してください。

テストは `cargo test` で実行できます。

> [この曲](https://www.youtube.com/watch?v=wFOjWCLPDDM)を聴きながらコードを書くと効率 x10000
