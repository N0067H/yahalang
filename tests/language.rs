use std::io::Cursor;

use yahalang::executor::{RuntimeError, execute};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::{Instruction, Program, parse};
use yahalang::token::TokenKind;

fn compile(source: &str, language: Language) -> Program {
    parse(&lex(source, language).expect("source should lex")).expect("source should parse")
}

fn run(source: &str, input: &str) -> (Result<(), RuntimeError>, String) {
    let program = compile(source, Language::English);
    let mut input = Cursor::new(input.as_bytes());
    let mut output = Vec::new();
    let result = execute(&program, &mut input, &mut output);
    (result, String::from_utf8(output).expect("output is UTF-8"))
}

#[test]
fn all_language_spellings_produce_the_same_core_tokens() {
    let spellings = [
        (
            Language::English,
            "ura yaha haa puru puruya huung hueung jureuri fuun ui i ua",
        ),
        (
            Language::Korean,
            "우라 야하 하아 푸루 푸루야 후응 후으응 쥬르리 후웅 우이 이 우아",
        ),
        (
            Language::Japanese,
            "ウラ ヤハ ハァ プル プルャ フゥン フウン ジュルリ フン ウイ イ ウア",
        ),
    ];
    let expected = [
        TokenKind::Ura,
        TokenKind::Yaha,
        TokenKind::Ha,
        TokenKind::Puru,
        TokenKind::Puruya,
        TokenKind::Huung,
        TokenKind::Hueung,
        TokenKind::Jureuri,
        TokenKind::Fuun,
        TokenKind::Ui,
        TokenKind::I,
        TokenKind::Ua,
        TokenKind::Eof,
    ];

    for (language, source) in spellings {
        let actual: Vec<_> = lex(source, language)
            .unwrap()
            .into_iter()
            .filter(|token| token.kind != TokenKind::HorizontalWhitespace)
            .map(|token| token.kind)
            .collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn whitespace_ends_a_count_and_adjacent_yaha_extends_it() {
    assert_eq!(
        compile("ura yahayaha\nura yaha yaha", Language::English).lines,
        vec![
            vec![Instruction::Push(2)],
            vec![Instruction::Push(1), Instruction::Add]
        ]
    );
}

#[test]
fn arithmetic_uses_checked_i64_operations_and_truncates_division() {
    assert_eq!(run("ui ui yaha i", "12\n30\n").1, "42");
    assert_eq!(run("ui ui yaha! i", "12\n30\n").1, "-18");
    assert_eq!(run("ui ui huung i", "6\n7\n").1, "42");
    assert_eq!(run("ui ui hueung i", "-7\n2\n").1, "-3");
    assert_eq!(run("ui ui jureuri i", "-7\n2\n").1, "-1");
    assert_eq!(run("ui fuun i", "42\n").1, "-42");
}

#[test]
fn stack_commands_duplicate_and_discard_values() {
    assert_eq!(run("ui puru yaha i", "5\n").1, "10");
    assert_eq!(
        run("ui puru! i", "5\n").0.unwrap_err().message,
        "stack underflow"
    );
    assert_eq!(run("ui ui puru! i", "5\n8\n").1, "5");
}

#[test]
fn zero_condition_runs_one_instruction_and_nonzero_skips_it() {
    assert_eq!(run("ui puru haa? i", "0\n").1, "0");
    assert_eq!(run("ui puru haa? i", "5\n").1, "");
}

#[test]
fn nested_conditionals_execute_only_when_each_condition_is_zero() {
    assert_eq!(run("ui puru puru haa? haa? i", "0\n").1, "0");
    assert_eq!(run("ui puru puru haa? haa? i", "5\n").1, "");
}

#[test]
fn jumps_count_blank_lines_and_discard_the_rest_of_the_source_line() {
    let source = "urayahayahayaha puruya ui\nui\n\nura yaha i";
    assert_eq!(run(source, "").1, "1");
}

#[test]
fn halt_discards_remaining_instructions() {
    assert_eq!(run("puruya! ui", "").0, Ok(()));
}

#[test]
fn empty_program_and_blank_lines_succeed() {
    assert_eq!(run("", "").0, Ok(()));
    assert_eq!(run("\n\n", "").0, Ok(()));
}

#[test]
fn runtime_errors_cover_underflow_overflow_zero_divisors_and_bad_jumps() {
    assert!(run("yaha", "").0.unwrap_err().message.contains("underflow"));
    assert!(
        run("ui ui yaha", "9223372036854775807\n1\n")
            .0
            .unwrap_err()
            .message
            .contains("overflow")
    );
    assert!(
        run("ui ui hueung", "1\n0\n")
            .0
            .unwrap_err()
            .message
            .contains("zero")
    );
    assert!(
        run("ui ui jureuri", "1\n0\n")
            .0
            .unwrap_err()
            .message
            .contains("zero")
    );
    assert!(
        run("ura puruya", "")
            .0
            .unwrap_err()
            .message
            .contains("target")
    );
}

#[test]
fn integer_input_requires_one_valid_i64_per_line() {
    assert_eq!(run("ui i", " +42 \r\n").1, "42");
    assert!(run("ui", "").0.is_err());
    assert!(run("ui", "  \n").0.is_err());
    assert!(run("ui", "1 2\n").0.is_err());
    assert!(run("ui", "9223372036854775808\n").0.is_err());
}

#[test]
fn character_output_accepts_unicode_scalars_and_rejects_invalid_values() {
    let valid = Program {
        lines: vec![vec![
            Instruction::Push('🐰' as i64),
            Instruction::PrintCharacter,
        ]],
    };
    let mut input = Cursor::new(b"");
    let mut output = Vec::new();
    execute(&valid, &mut input, &mut output).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "🐰");

    let invalid = Program {
        lines: vec![vec![Instruction::Push(0xD800), Instruction::PrintCharacter]],
    };
    let mut input = Cursor::new(b"");
    let mut output = Vec::new();
    assert!(
        execute(&invalid, &mut input, &mut output)
            .unwrap_err()
            .message
            .contains("Unicode scalar")
    );
}

#[test]
fn each_input_instruction_consumes_a_whole_line() {
    assert_eq!(run("ui ui yaha i", "4\n5\n").1, "9");
}
