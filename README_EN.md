<h1 align="center">Yahalang</h1>

<p align="center"><img src="image.png" alt="Usagi from Chiikawa" width="280"></p>
<p align="center">English · <a href="README.md">한국어</a> · <a href="README_JP.md">日本語</a></p>

Yahalang is a stack language inspired by Usagi's sounds and rap refrain in *Chiikawa*.
It supports Korean, English and Japanese spellings, checked signed 64-bit integers, and mutable integer arrays.

**Breaking grammar change:** old literals such as `ura yaha`, old commands such as `ui`, and line-number jumps are no longer supported. Convert existing programs to the new grammar; there is no automatic legacy mode.
The complete specification is in [DESIGN.md](DESIGN.md) (Korean).

## Running

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang ko
cargo run -- run examples/countdown.yaha --lang ko
cargo run -- repl --lang en
```

Repository examples use Korean. The first two commands print `Hello, world!` followed by a newline, and `321`, respectively.
The REPL runs each input line as an independent program, including its stack and arrays. Close blocks on that same input line. Exit with `:q` or `:quit`.

## Numbers and commands

`hayaha` opens a number, each `ppuru` adds one, and `yaha` closes it.
`hayaha yaha` is 0; `hayaha ppuruppuru yaha` is 2. Whitespace inside a number does not change its value.

| English | Korean | Japanese | Meaning |
| --- | --- | --- | --- |
| `hayaha … yaha` | `하야하 … 야하` | `ハヤハ … ヤハ` | Number, counting `ppuru` tokens |
| `yaha` | `야하` | `ヤハ` | Add |
| `ura yaha` | `우라 야하` | `ウラ ヤハ` | Subtract |
| `hat` | `핫` | `ハッ` | Multiply |
| `ura hat` | `우라 핫` | `ウラ ハッ` | Divide, truncating toward zero |
| `ura hathat` | `우라 핫핫` | `ウラ ハッハッ` | Remainder |
| `ura yat` | `우라 얏` | `ウラ ヤッ` | Negate |
| `ppuru` / `ura ppuru` | `뿌루` / `우라 뿌루` | `プル` / `ウラ プル` | Duplicate / discard |
| `iiiiyaha` | `이이이야하` | `イイイヤハ` | Read an integer |
| `iiiiyaha ppururu` | `이이이야하 뿌루루` | `イイイヤハ プルル` | Print an integer |
| `iiiiyaha urara` | `이이이야하 우라라` | `イイイヤハ ウララ` | Print a Unicode character |
| `yat` | `얏` | `ヤッ` | Halt |

Binary operations consume two integers; the lower value is the left operand. Output consumes its operand without adding a newline. Input reads one integer per complete line.
These English and Japanese spellings are language design choices, not official lyric transcriptions.

## Phrases and blocks

Write `iya` and `ura` without dashes; dashes are errors. Tokens can be joined, as in `ppuruppuru`, and the longest sound wins: `ppururu` is distinct from `ppuru`.
Spaces and newlines do not break phrases. For example, `iiiiyaha` followed by `ppururu` on the next line is still integer output.
Use `;` to break a phrase explicitly. `ura yahayaha` closes a conditional; `ura yaha ; yaha` subtracts, then adds. Standalone `yahayaha` adds twice.
Semicolons cannot split number literals or block markers. Strings, character literals and comments are unsupported.

| Open | Close | Meaning |
| --- | --- | --- |
| `iya yadayada` | `ura yadayada` | While the top integer is nonzero |
| `iya yahayaha` | `ura yahayaha` | Pop an integer; execute the body if nonzero |

A while loop does not pop its condition. It checks before entry and after each iteration; its body must leave an integer for the next check. Blocks may nest up to 128 levels. Blank lines and indentation have no control-flow meaning.

```text
hayaha ppuruppuruppuru yaha
iya yadayada
    ppuru
    iiiiyaha ppururu
    hayaha ppuru yaha
    ura yaha
ura yadayada
yat
```

Prints `321`. To read and add two integers:

```text
iiiiyaha
iiiiyaha
yaha
iiiiyaha ppururu
yat
```

## Arrays

```text
[hayaha yaha, hayaha ppuru yaha, hayaha ppuruppuru yaha]
hayaha ppuru yaha @
iiiiyaha ppururu
```

Prints `1`. Arrays are zero-indexed; `[]` is empty and nested arrays are unsupported.
`#` consumes an array and pushes its length. `@` consumes an array and index and pushes the selected integer. `=` consumes an array, index and integer value and updates that element.
`ppuru` duplicates a shared reference, so modifications are visible through both references.

## Settings and errors

`run` and `repl` accept `--lang en|ko|ja` and `--error-style usagi|friendly`.
An explicit language overrides the saved setting; otherwise the default is English.
Save a default with `cargo run -- set lang ko`.
The TOML setting is `lang = "ko"` in `yahalang/config.toml` under `$XDG_CONFIG_HOME` (or `~/.config`) on Linux, `~/Library/Application Support` on macOS, or `%APPDATA%` on Windows.

The default error style prints playful sounds; `friendly` shows details and physical source locations.
Stack underflow, incorrect types, arithmetic overflow, zero divisors, invalid input, invalid array indices and invalid Unicode scalars cause runtime errors. Malformed literals, unknown sounds and mismatched blocks cause compile errors.

## Examples and verification

All files in [examples](examples) use `--lang ko`: Hello World, addition, integer echo, array mutation, countdown and a truth machine. The truth machine accepts 0 or 1; stop repeated output with Ctrl-C.

```sh
cargo fmt --check
cargo test --release
```

GitHub Actions checks formatting, runs release tests and builds Linux x86-64, Windows x86-64 and macOS Apple Silicon binaries. To release, update Cargo.toml and Cargo.lock and push a matching `v` version tag. Releases include archives and `SHA256SUMS`; prerelease tags create prereleases. Manual runs only produce build artifacts.
