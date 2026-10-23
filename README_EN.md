<h1 align="center">Yahalang</h1>

<p align="center"><img src="image.png" alt="Usagi from Chiikawa" width="280"></p>
<p align="center">English / <a href="README.md">한국어</a> / <a href="README_JP.md">日本語</a></p>

Yahalang is a small stack-based esolang that uses Usagi’s sounds from *Chiikawa*, with a grammar based on the refrain from Usagi’s rap.\
You can write it in Korean, English or Japanese, using a stack to work with signed 64-bit integers and integer arrays.

If you’re coming from an older version, you’ll need to update code that uses `ura yaha` numbers, `ui` input or line-number jumps to match the new grammar.\
See the examples below and the full grammar in [DESIGN.md](DESIGN.md) (in Korean).

## Running

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang ko
cargo run -- run examples/countdown.yaha --lang ko
cargo run -- repl --lang en
```

Run the Korean example files with `--lang ko` to print `Hello, world!` followed by a newline in the first example, and `321` in the second.\
The REPL runs one line at a time with a fresh stack and fresh arrays, so blocks must close on the same line.\
Type `:q` or `:quit` to exit.

## Numbers and commands

Start a number with `hayaha`, count it out with `ppuru`, then finish with `yaha`.\
No `ppuru` means 0 and two of them make 2, regardless of any spaces or line breaks inside the number.

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

Math commands take the top two integers off the stack with the lower one on the left, so pushing 7, then 2, then subtracting gives 5.\
Output takes a value off the stack without adding a newline, while input reads one integer per line.\
The English and Japanese spellings here are for writing code; they aren’t official transcriptions of the lyrics.

## Phrases and blocks

Write sounds without dashes, like `iya` and `ura`, or join them as in `ppuruppuru`.\
When sounds overlap, the longer one is read first, so `ppururu` is a different sound from `ppuru`.\
A phrase can carry on across spaces and newlines, so `iiiiyaha` and `ppururu` still work together as integer output even on separate lines.\
Use `;` to keep commands apart: `ura yahayaha` closes a conditional, but `ura yaha ; yaha` subtracts and then adds, while `yahayaha` on its own adds twice.\
Keep semicolons out of numbers and block markers; there’s no syntax for strings, character literals or comments yet.

| Open | Close | Meaning |
| --- | --- | --- |
| `iya yadayada` | `ura yadayada` | While the top integer is nonzero |
| `iya yahayaha` | `ura yahayaha` | Pop an integer; execute the body if nonzero |

A loop checks the top integer before it starts and after each pass without taking the value off the stack, so make sure the body leaves an integer for the next check.\
Blocks can nest up to 128 levels deep, with blank lines and indentation left to you.

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

The loop prints `321`; the following program reads two integers and adds them:

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

This prints `1`, the second item in `[0, 1, 2]`, since array indices start at 0.\
Use `[]` for an empty array; nested arrays aren’t supported.\
Use `#` to get an array’s length, `@` to read an item, or `=` to change one, putting the array on the stack first, followed by the index for `@` or the index and new value for `=`.\
These commands take their arguments off the stack.\
To reuse an array, copy its reference with `ppuru`; both references share the same array and its changes.

## Settings and errors

Choose your spelling with `--lang en`, `--lang ko` or `--lang ja` on `run` and `repl`.\
The command-line option takes priority over your saved setting, with English used if you haven’t chosen either.\
To save a default, run `cargo run -- set lang ko`.\
The TOML setting is `lang = "ko"` in `yahalang/config.toml` under `$XDG_CONFIG_HOME` (or `~/.config`) on Linux, `~/Library/Application Support` on macOS, or `%APPDATA%` on Windows.

Errors use Usagi sounds by default; `--error-style friendly` shows the reason and source location.\
If a program stops, check for missing stack values, wrong types, numbers outside the 64-bit range, division by zero, bad input, out-of-bounds array indices or invalid Unicode values.\
Unclosed numbers or blocks and unknown sounds are caught before the program runs.

## Examples and tests

Run the programs in [examples](examples) with `--lang ko` to try Hello World, addition, integer echo, arrays, a countdown and a truth machine.\
Give the truth machine 0 or 1, pressing Ctrl-C to stop the repeated output if you enter 1.

```sh
cargo fmt --check
cargo test --release
```

GitHub Actions runs the format check and release tests, then builds for Linux x86-64, Windows x86-64 and macOS Apple Silicon.\
To publish a release with the archives and `SHA256SUMS` attached, update the version in Cargo.toml and Cargo.lock, then push a matching tag such as `v1.0.0`.\
Prerelease tags make prereleases, while manual runs just build the files.
