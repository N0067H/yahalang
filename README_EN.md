<h1 align="center">Yahalang</h1>

<p align="center"><img src="image.png" alt="Usagi from Chiikawa" width="280"></p>
<p align="center">English / <a href="README.md">한국어</a> / <a href="README_JP.md">日本語</a></p>

Yahalang is an esoteric programming language whose grammar comes from the refrain of Usagi’s rap in *Chiikawa*.\
Commands are phrases made from Usagi’s sounds. Programs use a stack to work with signed 64-bit integers and integer arrays, with Korean, English, and Japanese spellings available.

## Getting started

Build and run with Rust and Cargo installed:

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang ko
```

This prints `Hello, world!` and a newline. All example files in this repository use Korean spellings, so run them with `--lang ko`.

To enter code interactively, start the REPL:

```sh
cargo run -- repl --lang en
```

```text
yaha> hayaha ppuruppuru yaha
yaha> ppuru
yaha> yaha
yaha> :stack
[4]
yaha> iiiiyaha ppururu
4
yaha> :q
```

This pushes 2, duplicates it, and adds the two values. The REPL keeps values between submissions. See [REPL](#repl) for multiline input and reset commands.

## Stack and numbers

Commands work with values at the top of the stack. In this guide, the bottom is on the left and the top is on the right: in `[7, 2]`, 2 is on top.

A number starts with `hayaha` and ends with `yaha`. Its value is the number of `ppuru` sounds between them.

| Code | Value |
| --- | --- |
| `hayaha yaha` | 0 |
| `hayaha ppuru yaha` | 1 |
| `hayaha ppuruppuru yaha` | 2 |

To make a negative number, create a number and apply `ura yat` to change its sign. Fractional values are not supported.

```text
hayaha ppuruppuru yaha
hayaha ppuruppuruppuru yaha
yaha
iiiiyaha ppururu
```

This pushes 2 and 3, adds them, and prints `5`. Printing removes the value from the stack.

In the tables below, `a` and `b` are values and `A` is an array reference. Each table shows only the part of the stack changed by a successful command. Values below it remain in place.

| Phrase | Stack change | Action |
| --- | --- | --- |
| `ppuru` | `a → a, a` | Duplicate the top value |
| `ura ppuru` | `a → nothing` | Discard the top value |
| `iya ppuru` | `a, b → b, a` | Swap the top two values |

For arrays, `ppuru` creates another reference to the same array. To copy the contents into a separate array, use [array copying](#references-and-copies).

## Reading phrases

Use the exact spellings without dashes. `iya` and `ura` are valid; `i-ya` and `u-ra` are not. There is no syntax for comments, character literals, or string literals.

Spaces and newlines do not end a phrase. Writing `iiiiyaha` on one line and `ppururu` on the next still forms one integer output command. Indentation is optional.

Sounds can be joined together. When spellings overlap, the longest match wins: `ppuruppuru` is two duplicates, but `ppururu` is a separate sound.

Use `;` to prevent adjacent commands from combining into a phrase.

| Code | Meaning |
| --- | --- |
| `iiiiyaha ppururu` | Print an integer |
| `iiiiyaha ; ppuru` | Read an integer, then duplicate it |
| `ura yahayaha` | Close a conditional block |
| `ura yaha ; yaha` | Subtract, then add |
| `yahayaha` | Add twice |

A semicolon cannot appear inside a phrase or number. For example, `haa ; hat`, `yada ; ppuru`, and `hayaha ; yaha` are errors.

## Arithmetic and comparisons

Binary operations use the lower value as the left operand and the top value as the right operand. Subtraction turns `[7, 2]` into `[5]`.

| Phrase | Stack change |
| --- | --- |
| `yaha` | `a, b → a + b` |
| `ura yaha` | `a, b → a - b` |
| `hat` | `a, b → a × b` |
| `ura hat` | `a, b →` integer quotient of `a ÷ b` |
| `ura hathat` | `a, b → a % b` |
| `ura yat` | `a → -a` |
| `haa iya` | `a, b →` 1 if `a < b`, otherwise 0 |
| `haa hat` | `a, b →` 1 if `a == b`, otherwise 0 |
| `haa yaha` | `a, b →` 1 if `a > b`, otherwise 0 |

Division truncates toward zero: -7 divided by 2 is -3, with a remainder of -1. Division by zero and arithmetic outside the signed 64-bit range are errors.

Comparisons accept integers only. They consume both operands and leave one result, so duplicate any values you need later. To test inequality, compare the equality result with 0. To test less-than-or-equal or greater-than-or-equal, compare the greater-than or less-than result with 0, respectively.

## Conditions and loops

| Open | Close | Condition handling |
| --- | --- | --- |
| `iya yahayaha` | `ura yahayaha` | Pop the top integer; run the body if nonzero |
| `iya yadayada` | `ura yadayada` | Leave the top integer in place; repeat while nonzero |

Conditions must be integers. Blocks can nest up to 128 levels deep, and opening and closing phrases must match.

### Conditionals

```text
iiiiyaha
iiiiyaha
haa iya
iya yahayaha
    hayaha ppuru yaha
    iiiiyaha ppururu
ura yahayaha
```

This prints `1` if the first input is less than the second, and nothing otherwise. The conditional consumes the comparison result left by `haa iya`.

### Loops

```text
hayaha ppuruppuruppuru yaha
iya yadayada
    ppuru
    iiiiyaha ppururu
    hayaha ppuru yaha
    ura yaha
ura yadayada
```

This prints `321` and leaves 0 on the stack. A loop checks the top integer before running the body and again before the next iteration. The body must leave an integer for that check. If the initial value is 0, the body is skipped and the 0 remains on the stack.

### Breaking and halting

`ura iya` exits the nearest enclosing loop. It can appear inside a conditional; in nested loops, it exits only the innermost loop. It does not change the stack.

```text
hayaha ppuruppuruppuru yaha
iya yadayada
    ura iya
ura yadayada
iiiiyaha ppururu
```

The loop exits even though the condition is 3. Execution continues after it and prints `3`. A break does not require an integer for the next loop check. A break outside a loop is a parse error, and code after a break is still checked for syntax errors.

`yat` ends the current program rather than just the loop. It is optional: execution also ends at the end of the file. In the REPL, it ends only the current submission and returns to the prompt.

## Input and output

Characters are represented by integer Unicode code points. For example, `A` is 65. A string is an array of these integers in order.

| Phrase | Input or output | Stack change |
| --- | --- | --- |
| `iiiiyaha` | Read one integer from a line | Push an integer |
| `urara iiiiyaha` | Read one UTF-8 character | Push a code point |
| `urara yada` | Read one UTF-8 line | Push a new code point array |
| `iiiiyaha ppururu` | Print an integer | Consume an integer |
| `iiiiyaha urara` | Print a character | Consume an integer |
| `yada iiiiyaha` | Print an array as characters | Consume one array reference |

Output commands do not add a newline. To print one, output code point 10 as a character or include it in a character array.

### Integer input

```sh
printf '12\n30\n' | cargo run -- run examples/add_two_numbers.yaha --lang ko
```

This reads two lines and prints their sum, `42`. Enter one integer per line. Surrounding whitespace is allowed, but `12 30` on a single line is an error.

### Character input

```text
urara iiiiyaha
iiiiyaha urara
```

This reads and echoes one character. Input `A` produces `A`. Replace the last command with `iiiiyaha ppururu` to print its code point, `65`.

Whitespace and newlines are characters too. The command consumes no following character or line ending. A terminal may wait for Enter before delivering input; that newline remains available for the next read.

### String input and output

```text
urara yada
yada iiiiyaha
```

This reads and echoes one line. Entering `ABC` followed by Enter creates `[65, 66, 67]` and then prints `ABC`.

String input strips only the terminating LF or CRLF. Spaces, tabs, NUL, and a standalone CR are preserved. An empty line produces an empty array, and a final line without a newline is accepted. Each read creates an independent array.

To get the length instead, use `yada ppururu`:

```text
urara yada
yada ppururu
iiiiyaha ppururu
```

Input `ABC` produces `3`. Length counts Unicode code points, not displayed characters. A combining sequence or an emoji made of several code points occupies several array elements.

Character array output reads from the beginning of the array without modifying it. An empty array prints nothing. All character values are validated before writing, so an invalid value reports an error without producing output or changing the stack. Duplicate the reference with `ppuru` before printing if you need to keep the array.

### EOF and input limits

EOF handling depends on the input command.

| Input | EOF handling | Limit |
| --- | --- | --- |
| Integer | Error if no data is read | 1,024 bytes per line, including the newline |
| Character | -1 before a character starts; error in the middle of a character | One UTF-8 character |
| String | Return the final content as an array; error if there is no data | 1 MiB per line, excluding LF/CRLF |

The character input EOF value, -1, is not a valid character. Check for EOF before printing it as a character or as part of a character array. [character_echo.yaha](examples/character_echo.yaha) includes this check and echoes characters until EOF.

Invalid UTF-8, invalid integers, and input exceeding the limits are errors. Input already consumed is not restored after an error. All three input commands share the same input stream.

## Arrays

Use `yada hayaha` to create an empty array. Unlike a number, this phrase does not need a closing `yaha`. Arrays contain integers only, and indices start at 0.

| Phrase | Stack change | Action |
| --- | --- | --- |
| `yada hayaha` | Push a new array | Create an empty array |
| `yada ppuru` | `A, value → A` | Append an integer |
| `yada ura` | `A → A, value` | Remove the last integer and push it |
| `yada urara` | `A → A, copy` | Copy the contents into an independent array |
| `yada ppururu` | `A → length` | Get the length |
| `yada iya` | `A, index → value` | Get an element |
| `yada hat` | `A, index, value → nothing` | Set an element |
| `yada iiiiyaha` | `A → nothing` | Print the array as characters |

Append and pop leave the array reference on the stack. Length, lookup, assignment, and output consume it, so use `ppuru` first if you need the array again.

### Appending and popping

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

This appends 1 and 2, then pops them in reverse order and prints `21`.

| After | Stack |
| --- | --- |
| `yada hayaha` | `[[]]` |
| Push 1, then `yada ppuru` | `[[1]]` |
| Push 2, then `yada ppuru` | `[[1, 2]]` |
| `yada ura` | `[[1], 2]` |
| `iiiiyaha ppururu` | `[[1]]` |
| `yada ura` | `[[], 1]` |
| `iiiiyaha ppururu` | `[[]]` |

The brackets here show the stack and array contents. The program itself needs no symbols.

### References and copies

`ppuru` duplicates a reference to the same array. Mutating, appending, or popping through either reference changes the array seen by both.

`yada urara` copies the contents into a separate array. It keeps the original reference and places the copy on top.

```text
yada hayaha
hayaha ppuru yaha
yada ppuru
yada urara
hayaha ppuruppuru yaha
yada ppuru
```

In the REPL, `:stack` now shows `[[1], [1, 2]]`. Using `ppuru` instead of `yada urara` would produce `[[1, 2], [1, 2]]`.

### Existing symbol syntax

Array literals and symbol commands remain supported.

| Symbol | Equivalent phrase |
| --- | --- |
| `[]` | `yada hayaha` |
| `#` | `yada ppururu` |
| `@` | `yada iya` |
| `=` | `yada hat` |

Separate array literal elements with commas. `[hayaha yaha, hayaha ppuru yaha]` creates `[0, 1]`. Arrays cannot contain other arrays.

## REPL

Complete submissions execute immediately, and the stack and arrays remain available for the next submission. Unfinished numbers, arrays, blocks, and phrases continue at the `...>` prompt without executing until complete.

| Command | Action |
| --- | --- |
| `:stack` | Display the stack from bottom to top |
| `:reset` | Clear the stack, arrays, and pending source |
| `:q`, `:quit` | Discard pending source and exit |

A syntax error discards only the pending source. A runtime error leaves the state at the point of failure and returns to the prompt; earlier instructions are not rolled back. EOF with unfinished source reports an error.

Separate complete submissions do not combine into phrases. Entering only `iiiiyaha` on a line runs integer input immediately; it does not wait for `ppururu` on the next source line.

Program input commands use the same stream as REPL source. While an input command is running, the next line is read as program data rather than source code.

The REPL adds a display newline when needed to keep the next prompt off the program’s output. File execution does not add this newline.

## Language settings

Select source spellings with `--lang en`, `--lang ko`, or `--lang ja`. A program cannot mix spellings from different languages.

```sh
cargo run -- set lang en
```

This saves the default spelling. An explicit `--lang` takes priority over the saved setting. English is used if neither is set.

Phrase structure stays the same; only the sounds change. For example, `yada urara` is `야다 우라라` in Korean and `ヤダ ウララ` in Japanese.

| English | Korean | Japanese |
| --- | --- | --- |
| hayaha | 하야하 | ハヤハ |
| haa | 하아 | ハア |
| ppuru | 뿌루 | プル |
| ppururu | 뿌루루 | プルル |
| yaha | 야하 | ヤハ |
| iiiiyaha | 이이이야하 | イイイヤハ |
| urara | 우라라 | ウララ |
| iya | 이야 | イヤ |
| yada | 야다 | ヤダ |
| ura | 우라 | ウラ |
| hat | 핫 | ハッ |
| yat | 얏 | ヤッ |

Settings are stored as TOML, for example `lang = "en"`.

| OS | Path |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/yahalang/config.toml` or `~/.config/yahalang/config.toml` |
| macOS | `~/Library/Application Support/yahalang/config.toml` |
| Windows | `%APPDATA%/yahalang/config.toml` |

## Execution tracing

Add `--trace` to `run` or `repl` to see executed phrases, source locations, and stack contents.

```sh
cargo run -- run examples/trace_stack.yaha --lang ko --trace
cargo run -- repl --lang en --trace
```

The example duplicates 2, adds the two values, and prints `4`. The addition produces these trace entries:

```text
[trace] examples/trace_stack.yaha:3:1 before "야하" stack=[2, 2]
[trace] examples/trace_stack.yaha:3:1 after "야하" stack=[4]
```

Locations use `file:line:column`, with lines and columns starting at 1. Columns count characters, not bytes. A multiline phrase uses its starting location; whitespace is collapsed only for display, without changing the source.

| Status | Meaning |
| --- | --- |
| `before` | Stack before execution, including before a command waits for input |
| `after` | Stack after a successful command |
| `error` | Stack and error reason at the point of failure; no `after` entry follows for that command |

The stack uses the same order as `:stack`, with arrays shown by their current contents. Each loop iteration is traced, but skipped bodies are not. A conditional closing phrase has no executable instruction and therefore no entry. Instructions skipped by a break or `yat` are not shown.

REPL entries use `<repl>` as the filename and line numbers relative to the current submission. State persistence and reset behavior are unchanged. Source rejected during parsing produces no execution trace.

Trace entries go to stderr; program output stays on stdout. To save them separately:

```sh
cargo run -- run examples/trace_stack.yaha --lang ko --trace > result.txt 2> trace.log
```

`result.txt` contains `4`, and `trace.log` contains the trace. Cargo messages may also appear on stderr. Large arrays and long-running loops can produce large logs. A trace write failure stops execution without rolling back completed instructions.

## Understanding errors

Errors use Usagi sounds by default. Add `--error-style friendly` to see the cause and location.

```sh
cargo run -- run examples/array_access.yaha --lang ko --error-style friendly
cargo run -- repl --lang en --error-style friendly
```

Unknown sounds, invalid phrases, unclosed numbers or blocks, and breaks outside loops are checked before execution. Runtime errors include stack underflow, wrong types, integer overflow, division by zero, and invalid indices, character values, or input.

A runtime error does not always restore the stack to its previous state: some commands may already have consumed operands. Array append, pop, and copy leave the state unchanged on stack underflow or invalid operands. Popping an empty array also leaves the state unchanged.

Character array output validates character values before writing and keeps the array reference on write or flush failure. Bytes already written or input already consumed cannot be rolled back.

## Example programs

| File | Behavior |
| --- | --- |
| [trace_stack.yaha](examples/trace_stack.yaha) | Trace stack changes through duplication and addition |
| [hello_world.yaha](examples/hello_world.yaha) | Print `Hello, world!` and a newline |
| [add_two_numbers.yaha](examples/add_two_numbers.yaha) | Print the sum of two integers |
| [integer_echo.yaha](examples/integer_echo.yaha) | Read and print one integer |
| [character_echo.yaha](examples/character_echo.yaha) | Echo characters as they are read until EOF |
| [string_echo.yaha](examples/string_echo.yaha) | Read a line into an array and print it |
| [string_length.yaha](examples/string_length.yaha) | Print the number of code points in a line |
| [print_character_array.yaha](examples/print_character_array.yaha) | Collect characters until EOF, then print the array |
| [array_push_pop.yaha](examples/array_push_pop.yaha) | Append and pop 1 and 2, printing `21` |
| [array_access.yaha](examples/array_access.yaha) | Set and get array elements |
| [array_copy.yaha](examples/array_copy.yaha) | Print the copy’s length (2) and the original’s length (1) |
| [reverse_input.yaha](examples/reverse_input.yaha) | Reverse characters read until EOF |
| [compare.yaha](examples/compare.yaha) | Print `1` if the first input is less than the second |
| [countdown.yaha](examples/countdown.yaha) | Print `321` |
| [break_on_zero.yaha](examples/break_on_zero.yaha) | Print integers until 0 ends the loop |
| [truth_machine.yaha](examples/truth_machine.yaha) | Print 0 once or 1 repeatedly |

`reverse_input.yaha` reverses code points, not whole combining sequences or multi-code-point emoji. An input newline is reversed along with the rest of the text.

End input to `break_on_zero.yaha` with 0. If you enter 1 into `truth_machine.yaha`, stop its repeated output with Ctrl-C.

### Algorithms and text processing

These programs use the existing language commands. The [example guide](examples/README.md), written in Korean, documents input formats, sample runs, and the working array layouts.

| File | Behavior |
| --- | --- |
| [factorial.yaha](examples/factorial.yaha) | Compute N factorial |
| [gcd.yaha](examples/gcd.yaha) | Compute the greatest common divisor of two integers |
| [fibonacci.yaha](examples/fibonacci.yaha) | Print the first N Fibonacci terms |
| [is_prime.yaha](examples/is_prime.yaha) | Test whether an integer is prime |
| [number_stats.yaha](examples/number_stats.yaha) | Print the sum, minimum, and maximum of N integers |
| [array_sort.yaha](examples/array_sort.yaha) | Sort a copy of the input in ascending order |
| [palindrome.yaha](examples/palindrome.yaha) | Test whether a string is a palindrome |
| [character_frequency.yaha](examples/character_frequency.yaha) | Count a given character in a string |
| [caesar_cipher.yaha](examples/caesar_cipher.yaha) | Encrypt or decrypt ASCII letters with a Caesar shift |
| [postfix_calculator.yaha](examples/postfix_calculator.yaha) | Evaluate an integer postfix expression |

## Compiler written in Yahalang

[compiler/yahac.yaha](compiler/yahac.yaha) is yahac, a Yahalang-to-C compiler written in Yahalang. It reads a program on standard input and writes a self-contained C program to standard output. yahac detects whether the program uses English, Korean or Japanese spelling, but each program must use only one of them.

```sh
cargo run --release -- run compiler/yahac.yaha --lang ko < examples/countdown.yaha > countdown.c
cc -O2 -o countdown countdown.c && ./countdown
```

The compiled program follows the same rules as the interpreter: arithmetic overflow, UTF-8 input and output, and errors print `haa? haa?` or `ui` and exit with status 1. If a program can't be compiled, yahac writes nothing and stops with a runtime error.

yahac can also compile itself. The native compiler built from that output produces identical C again:

```sh
cargo run --release -- run compiler/yahac.yaha --lang ko < compiler/yahac.yaha > yahac.c
cc -O1 -o yahac yahac.c
./yahac < compiler/yahac.yaha | cmp - yahac.c
```

Yahalang has no comments or names, so [compiler/gen_yahac.py](compiler/gen_yahac.py) generates `yahac.yaha` from a readable description. [compiler/runtime.c](compiler/runtime.c) is the C runtime included in every generated program. After changing either file, regenerate the compiler and run the tests in this order:

```sh
python3 compiler/gen_yahac.py
cargo build --release
python3 compiler/test_yahac.py
python3 compiler/test_regressions.py
```

The differential tests use `target/release/yahalang`, so build the release interpreter before running them.
