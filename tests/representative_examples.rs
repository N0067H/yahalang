use std::io::Cursor;

use yahalang::executor::{RuntimeError, execute};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::parse;

fn run(source: &str, input: &str) -> Result<String, RuntimeError> {
    let program = parse(&lex(source, Language::Korean).unwrap()).unwrap();
    let mut output = Vec::new();
    execute(&program, &mut Cursor::new(input.as_bytes()), &mut output)?;
    Ok(String::from_utf8(output).unwrap())
}

fn check(source: &str, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(run(source, input).unwrap(), *expected, "input: {input:?}");
    }
}

#[test]
fn factorial_handles_zero_and_checked_overflow() {
    let source = include_str!("../examples/factorial.yaha");
    check(
        source,
        &[
            ("0\n", "1\n"),
            ("1\n", "1\n"),
            ("5\n", "120\n"),
            ("20\n", "2432902008176640000\n"),
            ("-1\n", "-1\n"),
        ],
    );
    assert!(
        run(source, "21\n")
            .unwrap_err()
            .message
            .contains("overflow")
    );
}

#[test]
fn gcd_handles_zero_and_coprime_inputs() {
    check(
        include_str!("../examples/gcd.yaha"),
        &[
            ("48\n18\n", "6\n"),
            ("0\n0\n", "0\n"),
            ("0\n7\n", "7\n"),
            ("7\n0\n", "7\n"),
            ("17\n13\n", "1\n"),
            ("-2\n6\n", "-1\n"),
            ("9223372036854775807\n1\n", "1\n"),
        ],
    );
}

#[test]
fn fibonacci_handles_empty_and_largest_supported_sequence() {
    let source = include_str!("../examples/fibonacci.yaha");
    check(
        source,
        &[
            ("0\n", ""),
            ("1\n", "0\n"),
            ("2\n", "0\n1\n"),
            ("7\n", "0\n1\n1\n2\n3\n5\n8\n"),
            ("-1\n", "-1\n"),
            ("94\n", "-1\n"),
        ],
    );
    let output = run(source, "93\n").unwrap();
    assert_eq!(output.lines().count(), 93);
    assert_eq!(output.lines().last(), Some("7540113804746346429"));
}

#[test]
fn prime_classifies_negative_small_and_square_inputs() {
    let source = include_str!("../examples/is_prime.yaha");
    for number in [-7, 0, 1, 4, 9, 25, 49, 121, 1024] {
        assert_eq!(run(source, &format!("{number}\n")).unwrap(), "0\n");
    }
    for number in [2, 3, 5, 17, 97, 997] {
        assert_eq!(run(source, &format!("{number}\n")).unwrap(), "1\n");
    }
}

#[test]
fn statistics_prints_sum_minimum_and_maximum() {
    let source = include_str!("../examples/number_stats.yaha");
    check(
        source,
        &[
            ("5\n3\n-2\n7\n7\n0\n", "15\n-2\n7\n"),
            ("1\n-4\n", "-4\n-4\n-4\n"),
            ("0\n", "-1\n"),
            ("-1\n", "-1\n"),
        ],
    );
    assert!(
        run(source, "2\n9223372036854775807\n1\n")
            .unwrap_err()
            .message
            .contains("overflow")
    );
}

#[test]
fn sorting_handles_empty_duplicates_and_extreme_values() {
    let source = include_str!("../examples/array_sort.yaha");
    check(
        source,
        &[
            ("0\n", ""),
            ("-1\n", "-1\n"),
            ("1\n8\n", "8\n"),
            ("5\n3\n-2\n7\n3\n0\n", "-2\n0\n3\n3\n7\n"),
            ("3\n1\n2\n3\n", "1\n2\n3\n"),
            ("3\n3\n2\n1\n", "1\n2\n3\n"),
            (
                "2\n9223372036854775807\n-9223372036854775808\n",
                "-9223372036854775808\n9223372036854775807\n",
            ),
        ],
    );
}

#[test]
fn sorting_keeps_the_original_working_array() {
    use yahalang::executor::ExecutionState;
    let source = include_str!("../examples/array_sort.yaha");
    let program = parse(&lex(source, Language::Korean).unwrap()).unwrap();
    let mut state = ExecutionState::default();
    let mut output = Vec::new();
    state
        .execute(&program, &mut Cursor::new(b"3\n3\n1\n2\n"), &mut output)
        .unwrap();
    assert_eq!(output, b"1\n2\n3\n");
    assert_eq!(state.stack_display(), "[[3, 3, 0, 0, 0, 0, 3, 1, 2]]");
}

#[test]
fn palindrome_compares_exact_unicode_code_points() {
    check(
        include_str!("../examples/palindrome.yaha"),
        &[
            ("\n", "1\n"),
            ("a\n", "1\n"),
            ("racecar\n", "1\n"),
            ("토마토\n", "1\n"),
            ("가🦀가\n", "1\n"),
            ("ab\n", "0\n"),
            ("Racecar\n", "0\n"),
            ("a\0a\n", "1\n"),
        ],
    );
}

#[test]
fn frequency_counts_a_scalar_including_spaces() {
    check(
        include_str!("../examples/character_frequency.yaha"),
        &[
            ("banana\na", "3\n"),
            ("가나가\n가", "2\n"),
            ("a b c\n ", "2\n"),
            ("\nx", "0\n"),
            ("hello\nz", "0\n"),
            ("abc\n", "-1\n"),
            ("a\0b\0\n\0", "2\n"),
        ],
    );
}

#[test]
fn caesar_rotates_ascii_letters_and_preserves_other_characters() {
    let source = include_str!("../examples/caesar_cipher.yaha");
    check(
        source,
        &[
            ("Abc Z!\n2\n", "Cde B!\n"),
            ("Cde B!\n-2\n", "Abc Z!\n"),
            ("Zz Aa\n1\n", "Aa Bb\n"),
            ("한글 123!\n5\n", "한글 123!\n"),
            ("Abc\n26\n", "Abc\n"),
            ("\n0\n", "\n"),
            ("Abc\n-9223372036854775808\n", "Stu\n"),
        ],
    );
}

#[test]
fn postfix_parses_signed_integers_and_validates_tokens() {
    let source = include_str!("../examples/postfix_calculator.yaha");
    check(
        source,
        &[
            ("12 3 + 2 *\n", "30\n"),
            ("-7 2 /\n", "-3\n"),
            ("-7 2 %\n", "-1\n"),
            ("8 3 -\n", "5\n"),
            ("0\n", "0\n"),
            ("  12\t3 +  \n", "15\n"),
            ("-9223372036854775808\n", "-9223372036854775808\n"),
            ("\n", "-1\n"),
            ("1 +\n", "-1\n"),
            ("1 2\n", "-1\n"),
            ("1 2 &\n", "-1\n"),
            ("12+\n", "-1\n"),
            ("2x\n", "-1\n"),
            ("1 2 +3\n", "-1\n"),
            ("-\n", "-1\n"),
        ],
    );
    assert!(
        run(source, "1 0 /\n")
            .unwrap_err()
            .message
            .contains("division by zero")
    );
    assert!(
        run(source, "9223372036854775807 1 +\n")
            .unwrap_err()
            .message
            .contains("overflow")
    );
}
