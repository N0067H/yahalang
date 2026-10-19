# Yahalang

[English](README.md) | [한국어](README_KO.md) | [日本語](README_JP.md)

Yahalang is a stack-based esoteric programming language inspired by Usagi's utterances in *Chiikawa*.
It supports English, Korean, and Japanese command spellings, signed 64-bit integers, and mutable integer arrays.

## Build and run

Install Rust and Cargo, then run these commands from the repository root:

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang en
```

Output:

```text
Hello, world!
```

Start the REPL:

```sh
cargo run -- repl --lang en
```

```text
yaha> ura yahayaha i
2
yaha> :q
```

Each REPL entry runs as a separate program.
Stack values and arrays do not persist between entries.
Use `:q` or `:quit` to exit.

A source file runs as one program, with state shared across its lines.

The examples below use English spellings.
An explicit `--lang en` overrides any saved default language.

## Numbers and the stack

`ura` starts a number at zero.
Each consecutive `yaha` increases it by one:

| Source | Value |
| --- | --- |
| `ura` | 0 |
| `ura yaha` | 1 |
| `ura yahayaha` | 2 |
| `ura yahayahayaha` | 3 |

The `yaha` syllables within one number must be adjacent.
A space ends the count.
For example, `ura yaha yaha` pushes 1 and then performs addition; it does not push 2.

This program pushes 1 and 2, adds them, and prints `3`:

```text
ura yaha ura yahayaha yaha i
```

Binary operations consume the top two values and push the result.
Subtraction, division, and remainder use the lower value as the left operand.

Overflow, division by zero, and insufficient stack values are runtime errors.

## Commands

| English | Korean | Japanese | Operation |
| --- | --- | --- | --- |
| `ura` | `우라` | `ウラ` | Start an integer literal |
| `yaha` / `yaha!` | `야하` / `야하!` | `ヤハ` / `ヤハ!` | Add / subtract |
| `huung` | `후응` | `フゥン` | Multiply |
| `hueung` | `후으응` | `フウン` | Integer division |
| `jureuri` | `쥬르리` | `ジュルリ` | Remainder |
| `fuun` | `후웅` | `フン` | Negate |
| `puru` / `puru!` | `푸루` / `푸루!` | `プル` / `プル!` | Duplicate / discard the top value |
| `haa?` | `하아?` | `ハァ?` | Execute the next instruction if the popped value is zero |
| `puruya` / `puruya!` | `푸루야` / `푸루야!` | `プルャ` / `プルャ!` | Jump to a line / halt |
| `ui` | `우이` | `ウイ` | Read an integer |
| `i` | `이` | `イ` | Print an integer |
| `ua` | `우아` | `ウア` | Print a Unicode character from its code point |

`haa?` consumes its condition and controls one instruction on the same line.
Conditions may be nested up to 128 levels.

Jump targets start at line 1.
Blank lines count toward line numbers.

`i` and `ua` consume their operands and do not append a newline.
The REPL adds a newline when needed to separate output from the next prompt.

There are no string literals, character literals, or comments.

## Integer input

Each `ui` reads one complete line containing a single integer.

```sh
cargo run -- run examples/add_two_numbers.yaha --lang en
```

Enter `10`, press Enter, then enter `2` and press Enter again.
The result is `12`.

Entering `10 2` on one line is invalid.

Alternatively:

```sh
printf '10\n2\n' | cargo run -- run examples/add_two_numbers.yaha --lang en
```

## Arrays

Array literals use square brackets and comma-separated integer literals:

```text
[ura, ura yaha, ura yahayaha] ura yaha @ i
```

This creates `[0, 1, 2]`, reads the element at index 1, and prints `1`.
Indices start at zero.

`[]` creates an empty array.
Nested arrays are not supported.

| Command | Operand order, bottom to top | Result |
| --- | --- | --- |
| `#` | array | Push the length |
| `@` | array, index | Push the selected integer |
| `=` | array, index, value | Update the selected element |

All three commands consume their operands.
Use `puru` to retain an array reference for later use.
Duplicated references share the same array, so mutations are visible through every reference.

Invalid types and indices produce runtime errors.

## Language settings

Use `--lang en`, `--lang ko`, or `--lang ja` with `run` or `repl`.
Use one spelling set consistently within a program.

Save a default language:

```sh
cargo run -- set lang ko
```

The setting is stored as TOML:

```toml
lang = 'ko'
```

| Platform | Configuration file |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/yahalang/config.toml`, or `~/.config/yahalang/config.toml` if unset |
| macOS | `~/Library/Application Support/yahalang/config.toml` |
| Windows | `%APPDATA%/yahalang/config.toml` |

An explicit `--lang` takes precedence over the saved setting.
Without either, English is used.

## Errors

The default `usagi` error style uses utterances such as `haa?`, `haa? haa?`, and `ui`.

Select `friendly` to see the cause:

```sh
cargo run -- repl --lang en --error-style friendly
```

Entering `yaha` on an empty stack reports:

```text
runtime error at line 1, instruction 1: stack underflow
```

The `--error-style usagi|friendly` option is available for both `run` and `repl`.

## Examples

| File | Behavior |
| --- | --- |
| [hello_world.yaha](examples/hello_world.yaha) | Print `Hello, world!` |
| [add_two_numbers.yaha](examples/add_two_numbers.yaha) | Read two integers on separate lines and print their sum |
| [integer_echo.yaha](examples/integer_echo.yaha) | Read and print one integer |
| [array_access.yaha](examples/array_access.yaha) | Update and read an array element |
| [truth_machine.yaha](examples/truth_machine.yaha) | Print `0` once and halt, or print `1` indefinitely |

For `truth_machine.yaha`, enter `0` or `1`.
Stop the repeating output with Ctrl-C.

See [DESIGN.md](DESIGN.md) for the language design.

Run the test suite with `cargo test`.

> Listen to [this song](https://www.youtube.com/watch?v=wFOjWCLPDDM) while coding for x10000 efficiency.
