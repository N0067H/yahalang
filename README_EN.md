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
The REPL preserves the stack and arrays between submissions. Unfinished blocks, numbers, and phrases continue at the `...>` prompt and execute once complete.
Use `:stack` to inspect values from bottom to top, `:reset` to clear values and pending source, and `:q` or `:quit` to exit.
Syntax errors discard pending source; runtime errors preserve changes made before the error. `yat` ends only the current submission. EOF reports unfinished source.

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
| `iya ppuru` | `이야 뿌루` | `イヤ プル` | Swap the top two values |
| `iiiiyaha` | `이이이야하` | `イイイヤハ` | Read an integer |
| `urara iiiiyaha` | `우라라 이이이야하` | `ウララ イイイヤハ` | Read a Unicode character; -1 at EOF |
| `iiiiyaha ppururu` | `이이이야하 뿌루루` | `イイイヤハ プルル` | Print an integer |
| `iiiiyaha urara` | `이이이야하 우라라` | `イイイヤハ ウララ` | Print a Unicode character |
| `ura iya` | `우라 이야` | `ウラ イヤ` | Break out of the nearest loop |
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

## Character input

`urara iiiiyaha` reads one UTF-8 Unicode scalar and pushes its code point, including whitespace and NUL. EOF pushes -1; invalid or truncated UTF-8 is a runtime error. `iiiiyaha urara` prints a character, so its word order is reversed.

Run `yahalang run examples/character_echo.yaha --lang ko` to echo input until EOF. The example adds 1 to each code point when checking the loop condition, allowing NUL while stopping at -1.

Character input shares the integer input and REPL stream. It consumes no following characters or line endings. Terminals may require Enter to deliver input; that newline remains available for the next read.

## Growing and shrinking arrays

| Phrase | Stack, bottom to top |
| --- | --- |
| `yada hayaha` | push a new empty array |
| `yada ppuru` | array, integer → array; append the integer |
| `yada ura` | array → array, integer; remove the last element |
| `yada urara` | array → original array, independent copy |
| `yada ppururu` | array → length |
| `yada iya` | array, index → element |
| `yada hat` | array, index, value → consume operands and update element |

`yada hayaha` creates a fresh empty array on each execution and needs no closing `yaha`. Existing array literals remain supported.

Append and pop keep the array reference on the stack and modify the shared array. Empty-array pops, wrong types, and stack underflow report runtime errors without changing the stack or array. Phrases may cross line breaks but cannot contain `;`.

```text
yada hayaha
hayaha ppuru yaha
yada ppuru
hayaha ppuruppuru yaha
yada ppuru
yada ura
iiiiyaha ppururu
yada ura
iiiiyaha ppururu
```

This appends 1 and 2, then removes and prints them in reverse order: `21`.
The array remains on the stack after each operation.

```sh
printf 'abc' | yahalang run examples/reverse_input.yaha --lang ko
```

Output: `cba`. The example reads until EOF and reverses Unicode scalar values, including any input newline; combining sequences are not kept together.

`yada ppururu`, `yada iya`, and `yada hat` are equivalent to `#`, `@`, and `=`. These operations consume the array reference; duplicate it first to reuse it. Existing symbols remain supported.

## Breaking out of a loop

`ura iya` exits the nearest enclosing loop, including from inside a conditional. Nested loops exit only the innermost loop. The stack is unchanged, and execution continues after the loop. Unlike `yat`, it does not end the program. Use outside a loop is a parse error, even in unreachable code. The phrase may cross a newline but cannot contain `;`.

```text
hayaha ppuruppuruppuru yaha
iya yadayada
    ura iya
ura yadayada
iiiiyaha ppururu
```

Output: `3`. The loop exits even though the condition remains nonzero.

```sh
printf '3\n2\n0\n' | yahalang run examples/break_on_zero.yaha --lang ko
```

Output: `32`. This example reads integers until 0, then breaks from inside a conditional. EOF before 0 is an integer input error.

## Independent array copies

`ppuru` duplicates a reference to the same array. `yada urara` copies the contents into a new array, leaving the original below the copy on the stack. Changes to either array do not affect the other; existing references to the original still share it. Empty arrays can be copied. Stack underflow and wrong types report runtime errors without modifying the stack.

```text
yada hayaha
hayaha ppuru yaha
yada ppuru
yada urara
hayaha ppuruppuru yaha
yada ppuru
```

In the REPL, `:stack` now shows `[[1], [1, 2]]`. Using `ppuru` instead of `yada urara` would show `[[1, 2], [1, 2]]`.

```sh
yahalang run examples/array_copy.yaha --lang ko
```

Output: `21`, the length of the modified copy followed by the original length. The phrase may cross a newline but cannot contain `;`.

## Printing character arrays

`yada iiiiyaha` (`야다 이이이야하` / `ヤダ イイイヤハ`) prints an integer array as Unicode characters in array order, without an automatic newline. On success it consumes one array reference without modifying the array. Use `ppuru` first to keep the array. An empty array prints nothing.

```text
yada hayaha
urara iiiiyaha
yada ppuru
urara iiiiyaha
yada ppuru
yada iiiiyaha
```

Input `Hi` produces `Hi`: the characters are collected in `[72, 105]` and then printed together.

```sh
printf 'Hello, 안녕!\n' | yahalang run examples/print_character_array.yaha --lang ko
```

The example collects characters until EOF before printing the entire array. Input whitespace, newlines, and NUL are preserved. EOF's -1 must not be stored as a character.

Every element is validated before output begins. Invalid Unicode values, wrong types, or stack underflow produce no output and leave the stack unchanged. An output device failure also leaves the array reference on the stack, but bytes already written cannot be rolled back. The phrase may cross a newline but cannot contain `;`.

## String input

`urara yada` (`우라라 야다` / `ウララ ヤダ`) reads one UTF-8 line into a new, independent array of Unicode code points. It removes the terminating LF or CRLF while preserving spaces, tabs, NUL, and a standalone CR. An empty line produces an empty array. A final nonempty line without a newline is accepted; EOF with no data is a runtime error, unlike character input's -1.

```text
urara yada
yada iiiiyaha
```

Input `Hello` followed by Enter produces `Hello`, without an automatic output newline.

```sh
printf '가나다\n' | yahalang run examples/string_echo.yaha --lang ko
printf '가나🦀\n' | yahalang run examples/string_length.yaha --lang ko
```

The examples output `가나다` and `3`, respectively. Length counts Unicode scalar values, not grapheme clusters.

Invalid UTF-8, I/O errors, and lines larger than 1 MiB excluding LF/CRLF report runtime errors without pushing a value. Consumed input is not rolled back. The REPL shares the same input stream; this command consumes only the next line. Phrases may cross newlines but cannot contain `;`.
