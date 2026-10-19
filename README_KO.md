# Yahalang

[English](README.md) | [한국어](README_KO.md) | [日本語](README_JP.md)

Yahalang은 *치이카와*의 우사기 말소리에서 착안한 스택 기반 난해한 프로그래밍 언어입니다.
영어, 한국어, 일본어 명령 표기와 부호 있는 64비트 정수, 수정 가능한 정수 배열을 지원합니다.

## 빌드 및 실행

Rust와 Cargo를 설치한 뒤 저장소 루트에서 실행합니다.

```sh
cargo build
cargo run -- run examples/hello_world.yaha --lang en
```

출력:

```text
Hello, world!
```

REPL을 시작하려면 다음 명령을 실행합니다.

```sh
cargo run -- repl --lang en
```

```text
yaha> ura yahayaha i
2
yaha> :q
```

REPL의 각 입력은 독립된 프로그램으로 실행됩니다.
스택과 배열은 다음 입력으로 이어지지 않습니다. 종료 명령은 `:q` 또는 `:quit`입니다.
파일은 전체가 하나의 프로그램으로 실행되며, 줄 사이에 상태가 유지됩니다.

아래 예제는 영어 표기를 사용합니다.
명시적으로 지정한 `--lang en`이 저장된 기본 언어보다 우선합니다.

## 숫자와 스택

`ura`는 0에서 숫자를 시작합니다. 이어 붙인 `yaha` 하나마다 값이 1씩 증가합니다.

| 코드 | 값 |
| --- | --- |
| `ura` | 0 |
| `ura yaha` | 1 |
| `ura yahayaha` | 2 |
| `ura yahayahayaha` | 3 |

한 숫자를 표현하는 `yaha`들은 서로 붙여 써야 합니다.
공백은 세기를 끝내므로 `ura yaha yaha`는 2가 아니라 1을 넣은 뒤 덧셈을 실행합니다.

다음 코드는 1과 2를 스택에 넣고 더한 뒤 `3`을 출력합니다.

```text
ura yaha ura yahayaha yaha i
```

이항 연산은 맨 위 두 값을 꺼내고 결과를 넣습니다.
뺄셈, 나눗셈, 나머지 연산에서 아래 값이 왼쪽 피연산자입니다.
오버플로, 0으로 나누기, 스택 값 부족은 실행 오류입니다.

## 명령

| 영어 | 한국어 | 일본어 | 동작 |
| --- | --- | --- | --- |
| `ura` | `우라` | `ウラ` | 정수 리터럴 시작 |
| `yaha` / `yaha!` | `야하` / `야하!` | `ヤハ` / `ヤハ!` | 덧셈 / 뺄셈 |
| `huung` | `후응` | `フゥン` | 곱셈 |
| `hueung` | `후으응` | `フウン` | 정수 나눗셈 |
| `jureuri` | `쥬르리` | `ジュルリ` | 나머지 |
| `fuun` | `후웅` | `フン` | 부호 반전 |
| `puru` / `puru!` | `푸루` / `푸루!` | `プル` / `プル!` | 맨 위 값 복제 / 버리기 |
| `haa?` | `하아?` | `ハァ?` | 꺼낸 값이 0이면 다음 명령 실행 |
| `puruya` / `puruya!` | `푸루야` / `푸루야!` | `プルャ` / `プルャ!` | 줄 번호로 점프 / 종료 |
| `ui` | `우이` | `ウイ` | 정수 입력 |
| `i` | `이` | `イ` | 정수 출력 |
| `ua` | `우아` | `ウア` | 코드 포인트로 유니코드 문자 하나 출력 |

`haa?`는 조건 값을 꺼내고 같은 줄의 다음 명령 하나를 조건부로 실행합니다.
조건 중첩은 최대 128단계입니다. 점프 대상 줄 번호는 1부터 시작하며 빈 줄도 셉니다.

`i`와 `ua`는 피연산자를 꺼내고 출력하며, 자동으로 줄바꿈을 붙이지 않습니다.
REPL은 출력과 다음 프롬프트를 구분하기 위해 필요한 줄바꿈을 추가합니다.
문자열 리터럴, 문자 리터럴, 주석 문법은 지원하지 않습니다.

## 정수 입력

`ui`는 한 줄 전체를 읽고, 그 줄에는 정수 하나만 있어야 합니다.

```sh
cargo run -- run examples/add_two_numbers.yaha --lang en
```

`10`을 입력하고 Enter를 누른 다음, `2`를 입력하고 다시 Enter를 누르면
`12`가 출력됩니다. `10 2`를 한 줄에 입력하면 오류가 발생합니다.

파이프로 입력을 전달할 수도 있습니다.

```sh
printf '10\n2\n' | cargo run -- run examples/add_two_numbers.yaha --lang en
```

## 배열

대괄호 안에 정수 리터럴을 쉼표로 구분해 배열을 만듭니다.

```text
[ura, ura yaha, ura yahayaha] ura yaha @ i
```

배열 `[0, 1, 2]`를 만들고 인덱스 1의 값을 읽어 `1`을 출력합니다.
인덱스는 0부터 시작합니다. 빈 배열은 `[]`이며, 중첩 배열은 지원하지 않습니다.

| 명령 | 피연산자 순서: 스택 아래에서 위로 | 결과 |
| --- | --- | --- |
| `#` | 배열 | 길이를 스택에 넣음 |
| `@` | 배열, 인덱스 | 해당 정수를 스택에 넣음 |
| `=` | 배열, 인덱스, 값 | 해당 원소를 수정 |

세 명령은 모두 피연산자를 스택에서 꺼냅니다.
배열을 다시 사용하려면 `puru`로 참조를 복제합니다.
복제한 참조는 같은 배열을 공유하므로 수정 내용도 함께 보입니다.
잘못된 타입이나 인덱스는 실행 오류를 발생시킵니다.

## 언어 설정

`run`과 `repl`에서 `--lang en`, `--lang ko`, `--lang ja`를 지정할 수 있습니다.
한 프로그램에서는 한 언어의 표기를 사용합니다.

기본 언어를 저장하려면 다음 명령을 실행합니다.

```sh
cargo run -- set lang ko
```

설정은 TOML로 저장됩니다.

```toml
lang = 'ko'
```

| 운영체제 | 설정 파일 |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/yahalang/config.toml`, 변수가 없으면 `~/.config/yahalang/config.toml` |
| macOS | `~/Library/Application Support/yahalang/config.toml` |
| Windows | `%APPDATA%/yahalang/config.toml` |

실행 시 지정한 `--lang`이 저장된 설정보다 우선합니다.
둘 다 없으면 영어를 사용합니다.

## 오류 메시지

기본 오류 스타일인 `usagi`는 `haa?`, `haa? haa?`, `ui` 같은 말소리를 출력합니다.
원인을 확인하려면 `friendly`를 지정합니다.

```sh
cargo run -- repl --lang en --error-style friendly
```

빈 스택에서 `yaha`를 입력하면 다음 오류가 표시됩니다.

```text
runtime error at line 1, instruction 1: stack underflow
```

`--error-style usagi|friendly`는 `run`과 `repl`에서 모두 사용할 수 있습니다.

## 예제

| 파일 | 동작 |
| --- | --- |
| [hello_world.yaha](examples/hello_world.yaha) | `Hello, world!` 출력 |
| [add_two_numbers.yaha](examples/add_two_numbers.yaha) | 정수 두 개를 별도 줄로 입력받아 합 출력 |
| [integer_echo.yaha](examples/integer_echo.yaha) | 정수 하나를 입력받아 그대로 출력 |
| [array_access.yaha](examples/array_access.yaha) | 배열 원소를 수정하고 조회 |
| [truth_machine.yaha](examples/truth_machine.yaha) | `0`을 한 번 출력하고 종료하거나 `1`을 계속 출력 |

`truth_machine.yaha`에는 `0` 또는 `1`을 입력합니다.
반복 출력은 Ctrl-C로 중단합니다.

언어 설계는 [DESIGN.md](DESIGN.md)를 참고하세요.
테스트는 `cargo test`로 실행합니다.

> [이 노래](https://www.youtube.com/watch?v=wFOjWCLPDDM)를 들으면서 하면 효율 x10000
