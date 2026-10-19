# Yahalang

Yahalang is a small stack-based language inspired by Usagi's sounds in
*Chiikawa*.

A program is made from short utterances. The words carry the commands, while
spaces and line breaks make them easier to read.

## How to read a program

Yahalang keeps its values on a stack. Think of the stack as a pile: most
commands take values from the top, do something, and put the result back.

A number starts with `ura`, then uses repeated `yaha` sounds to count up from
zero:

```text
ura              means 0
ura yaha         means 1
ura yahayaha     means 2
```

The repeated `yaha` syllables that make one number stay together. A space
ends that number and starts another command. For example:

```text
ura yaha yaha
```

This pushes 1, then adds the top two values. It does not mean the number 2.

Commands can be separated with spaces or written together when their boundaries
are clear. We prefer spaces in examples so each command is easy to spot. A new
line starts the next instruction line; blank lines still count when a program
jumps to a line number.

## The core vocabulary

The same commands can be written in Korean, English, or Japanese.
English uses romanized spellings, while Japanese uses katakana.

These spellings represent Usagi's sounds. The English forms are practical
romanizations, not an official English transcription.

| Role | Korean | English | Japanese |
| --- | --- | --- | --- |
| Start a number | `우라` | `ura` | `ウラ` |
| Count / add | `야하` | `yaha` | `ヤハ` |
| Subtract | `야하!` | `yaha!` | `ヤハ!` |
| Multiply | `후응` | `huung` | `フゥン` |
| Divide | `후으응` | `hueung` | `フウン` |
| Remainder | `쥬르리` | `jureuri` | `ジュルリ` |
| Negate | `후웅` | `fuun` | `フン`* |
| Duplicate / discard | `푸루` / `푸루!` | `puru` / `puru!` | `プル` / `プル!` |
| Conditional | `하아?` | `haa?` | `ハァ?` |
| Jump / stop | `푸루야` / `푸루야!` | `puruya` / `puruya!` | `プルャ` / `プルャ!` |
| Read an integer | `우이` | `ui` | `ウイ` |
| Print an integer | `이` | `i` | `イ` |
| Print a character | `우아` | `ua` | `ウア` |

*The Japanese negation spelling is chosen to keep it distinct from
multiplication. These command spellings are design choices, not strict
phonetic transcriptions.

Use one language's spellings consistently in a program. Punctuation stays
ASCII (`!` and `?`) in every spelling set.

## What the commands do

`yaha` adds the top two numbers. `yaha!` subtracts the top number from the
value below it.

`huung`, `hueung`, and `jureuri` multiply, divide, and take the remainder.
`fuun` changes the sign of the top number.

`puru` copies the top value, and `puru!` removes it.
`haa?` checks the top value and removes it.
If the value was zero, `haa?` runs the next command on the same line.

`puruya` takes a line number from the stack and jumps there.
`puruya!` ends the program.

`ui` reads an integer. `i` prints an integer, and `ua` prints a character.
Commands that need values fail if the stack does not have enough.
Invalid input, division by zero, arithmetic overflow, and invalid jump
targets also stop the program with an error.

## Running programs

`yahalang run <file>` runs a source file. `yahalang repl` starts the
interactive one-line REPL. Both commands accept `--lang en`, `--lang ko`, or
`--lang ja`.

Set a default language with `yahalang set lang ko`. The setting is saved in
`config.toml` under the user's configuration directory, inside a `yahalang`
folder. On Linux this is `$XDG_CONFIG_HOME/yahalang/config.toml`, or
`~/.config/yahalang/config.toml` when `XDG_CONFIG_HOME` is unset. An explicit
`--lang` option overrides the saved default; without either, English is used.

## Example: add two inputs

This program reads two integers, adds them, and prints the sum:

```text
ui ui yaha i
```

For input `12` and `30`, it prints `42`.

## Example: build and add small numbers

```text
ura yahayaha
ura yaha yaha
```

The first line pushes 2.
The second pushes 1, then adds it to 2, leaving 3 on the stack.

The space before the last `yaha` makes it an addition command rather than
part of the number.

## Future expressive forms

Longer cries may become expressive commands later. Candidates include
`ウララララララララ！`, `ヤァァァァッ！`, `ハァァァァッ！`,
`ヤハヤハヤハ！`, `ヤハッ！ ウハッ！`, `プルルルルルル`, `ヤプ`,
`ミヤァァ`, `フフーン`, `ハァッハッ！`, and `ユフフフフフフフフフ`.

These forms are not part of the core language yet.
