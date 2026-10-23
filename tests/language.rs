use std::io::Cursor;

use yahalang::executor::{RuntimeError, execute};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::{Program, parse};
use yahalang::token::{Span, Token, TokenKind};

fn compile(source: &str) -> Program {
    parse(&lex(source, Language::Korean).expect("source should lex")).expect("source should parse")
}
fn run(source: &str, input: &str) -> (Result<(), RuntimeError>, String) {
    let mut input = Cursor::new(input.as_bytes());
    let mut output = Vec::new();
    let result = execute(&compile(source), &mut input, &mut output);
    (result, String::from_utf8(output).unwrap())
}
fn number(value: usize) -> String {
    format!("하야하 {} 야하", "뿌루".repeat(value))
}
fn malformed(source: &str) -> bool {
    lex(source, Language::Korean)
        .map(|t| parse(&t).is_err())
        .unwrap_or(true)
}

#[test]
fn all_language_spellings_produce_the_same_program() {
    let sources = [
        (
            Language::Korean,
            "하야하 뿌루뿌루 야하 이야 야다야다 뿌루 이이이야하 뿌루루 하야하 뿌루 야하 우라 야하 우라 야다야다 얏",
        ),
        (
            Language::English,
            "hayaha ppuruppuru yaha iya yadayada ppuru iiiiyaha ppururu hayaha ppuru yaha ura yaha ura yadayada yat",
        ),
        (
            Language::Japanese,
            "ハヤハ プルプル ヤハ イヤ ヤダヤダ プル イイイヤハ プルル ハヤハ プル ヤハ ウラ ヤハ ウラ ヤダヤダ ヤッ",
        ),
    ];
    let expected = compile(sources[0].1);
    for (language, source) in sources {
        let program = parse(&lex(source, language).unwrap()).unwrap();
        assert_eq!(program, expected);
        let mut output = Vec::new();
        execute(&program, &mut Cursor::new(b""), &mut output).unwrap();
        assert_eq!(output, b"21");
    }
}

#[test]
fn longest_tokens_and_utf8_spans_are_preserved() {
    let tokens = lex("뿌루루 우라라 이이이야하 이야\r\n", Language::Korean).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Ppururu);
    assert_eq!(tokens[0].span, Span { start: 0, end: 9 });
    assert_eq!(tokens[2].kind, TokenKind::Urara);
    assert_eq!(tokens[4].kind, TokenKind::Iiiiyaha);
    assert_eq!(tokens[6].kind, TokenKind::Iya);
    assert_eq!(tokens[7].kind, TokenKind::Newline);
    assert_eq!(
        tokens[7].span.end,
        "뿌루루 우라라 이이이야하 이야\r\n".len()
    );
}

#[test]
fn numbers_have_explicit_endings_and_ignore_whitespace() {
    assert_eq!(run("하야하 뿌루\n 뿌루\t야하 이이이야하 뿌루루", "").1, "2");
    assert_eq!(run("하야하야하 이이이야하뿌루루", "").1, "0");
    assert_eq!(
        run(
            &format!("{} {} 야하 이이이야하 뿌루루", number(2), number(3)),
            ""
        )
        .1,
        "5"
    );
    for source in [
        "하야하",
        "하야하 뿌루",
        "하야하 ; 야하",
        "하야하 뿌루루 야하",
    ] {
        assert!(malformed(source), "{source}");
    }
}

#[test]
fn arithmetic_uses_checked_i64_and_truncates_division() {
    for (operation, input, expected) in [
        ("야하", "12\n30\n", "42"),
        ("우라 야하", "12\n30\n", "-18"),
        ("핫", "6\n7\n", "42"),
        ("우라 핫", "-7\n2\n", "-3"),
        ("우라 핫핫", "-7\n2\n", "-1"),
    ] {
        let (result, output) = run(
            &format!("이이이야하 이이이야하 {operation} 이이이야하 뿌루루"),
            input,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(output, expected);
    }
    assert_eq!(run("이이이야하 우라 얏 이이이야하 뿌루루", "42\n").1, "-42");
}

#[test]
fn explicit_boundaries_prevent_phrase_combination() {
    assert_eq!(
        run("이이이야하 ; 뿌루 야하 이이이야하 뿌루루", "5\n").1,
        "10"
    );
    assert_eq!(
        run(
            "이이이야하 이이이야하 이이이야하 우라 야하 ; 야하 이이이야하 뿌루루",
            "10\n8\n3\n"
        )
        .1,
        "15"
    );
    assert_eq!(
        run(
            "이이이야하 이이이야하 이이이야하 우라 핫핫핫 이이이야하 뿌루루",
            "3\n7\n2\n"
        )
        .1,
        "3"
    );
    assert!(malformed("우라 야하야하"));
    assert!(malformed("이야 ; 야다야다"));
}

#[test]
fn phrases_can_cross_lines() {
    assert_eq!(run("이이이야하\n뿌루\n이이이야하\n뿌루루", "8\n").1, "8");
    assert_eq!(
        run(
            "이이이야하 이야\n야하\n야하 이이이야하 이이이야하 뿌루루 우라\n야하 야하",
            "1\n9\n"
        )
        .1,
        "9"
    );
}

#[test]
fn stack_commands_duplicate_discard_and_halt() {
    assert_eq!(run("이이이야하 뿌루 야하 이이이야하 뿌루루", "5\n").1, "10");
    assert_eq!(
        run(
            "이이이야하 이이이야하 우라 뿌루 이이이야하 뿌루루",
            "5\n8\n"
        )
        .1,
        "5"
    );
    assert_eq!(run("얏 이이이야하", "").0, Ok(()));
}

#[test]
fn while_rechecks_the_top_without_consuming_it() {
    let source = include_str!("../examples/countdown.yaha");
    assert_eq!(run(source, ""), (Ok(()), "321".to_owned()));
    assert_eq!(
        run(
            "하야하 야하 이야 야다야다 이이이야하 우라 야다야다 이이이야하 뿌루루",
            ""
        )
        .1,
        "0"
    );
}

#[test]
fn conditional_consumes_the_condition_and_runs_a_whole_block() {
    let source =
        "이이이야하 이야 야하야하 하야하 뿌루 야하 뿌루 야하 이이이야하 뿌루루 우라 야하야하";
    assert_eq!(run(source, "0\n"), (Ok(()), String::new()));
    assert_eq!(run(source, "-1\n"), (Ok(()), "2".to_owned()));
    assert!(
        run(
            "이이이야하 이야 야하야하 우라 야하야하 이이이야하 뿌루루",
            "0\n"
        )
        .0
        .is_err()
    );
}

#[test]
fn nested_blocks_and_halt_inside_blocks_work() {
    let source = format!(
        "{} 이야 야다야다 {} 이야 야하야하 {} 이야 야다야다 뿌루 이이이야하 뿌루루 {} 우라 야하 우라 야다야다 우라 뿌루 우라 야하야하 {} 우라 야하 우라 야다야다",
        number(2),
        number(1),
        number(2),
        number(1),
        number(1)
    );
    assert_eq!(run(&source, ""), (Ok(()), "2121".to_owned()));
    assert_eq!(
        run(
            "하야하 뿌루 야하 이야 야다야다 얏 우라 야다야다 이이이야하",
            ""
        )
        .0,
        Ok(())
    );
    assert_eq!(
        run(
            "하야하 야하 이야 야하야하 이야 야다야다 이이이야하 우라 야다야다 우라 야하야하",
            ""
        )
        .0,
        Ok(())
    );
}

#[test]
fn empty_loops_skip_or_report_missing_or_wrong_conditions() {
    assert_eq!(run("하야하 야하 이야 야다야다 우라 야다야다", "").0, Ok(()));
    for source in [
        "이야 야다야다 우라 야다야다",
        "[] 이야 야다야다 우라 야다야다",
        "하야하 뿌루 야하 이야 야다야다 우라 뿌루 우라 야다야다",
        "하야하 뿌루 야하 이야 야다야다 우라 뿌루 [] 우라 야다야다",
        "[] 이야 야하야하 우라 야하야하",
    ] {
        assert!(run(source, "").0.is_err(), "{source}");
    }
}

#[test]
fn rejects_unmatched_mismatched_and_unclosed_blocks() {
    for source in [
        "이야",
        "이야 야다",
        "이야 야다야다",
        "우라 야다야다",
        "이야 야다야다 우라 야하야하",
        "이야 야하야하 우라 야다야다",
        "이야 야하야하 우라 ; 야하야하",
        "우라",
    ] {
        assert!(malformed(source), "{source}");
    }
}

#[test]
fn limits_block_depth_without_runtime_recursion() {
    let source = format!(
        "{}{}",
        "이야 야다야다 ".repeat(128),
        "우라 야다야다 ".repeat(128)
    );
    assert!(!malformed(&source));
    assert!(malformed(&format!("이야 야다야다 {source} 우라 야다야다")));
}

#[test]
fn arrays_share_references_and_check_types_and_bounds() {
    assert_eq!(
        run("[하야하 야하, 하야하 뿌루 야하] # 이이이야하 뿌루루", "").1,
        "2"
    );
    assert_eq!(
        run(
            &format!(
                "[{}, {}] 뿌루 {} {} = {} @ 이이이야하 뿌루루",
                number(0),
                number(1),
                number(1),
                number(2),
                number(1)
            ),
            ""
        )
        .1,
        "2"
    );
    assert_eq!(run("[] # 이이이야하 뿌루루", "").1, "0");
    for source in [
        "[] 하야하 야하 @",
        "[하야하 야하] 하야하 뿌루 야하 @",
        "[하야하 야하] 하야하 뿌루 야하 우라 얏 @",
        "하야하 야하 #",
        "[] [] @",
        "[] 하야하 야하 [] =",
    ] {
        assert!(run(source, "").0.is_err(), "{source}");
    }
    for source in ["[", "[하야하 야하,]", "[하야하 야하 하야하 야하]", "[[]]"] {
        assert!(malformed(source), "{source}");
    }
}

#[test]
fn runtime_errors_cover_underflow_overflow_and_zero_divisors() {
    assert!(run("야하", "").0.unwrap_err().message.contains("underflow"));
    for (source, input, message) in [
        (
            "이이이야하 이이이야하 야하",
            "9223372036854775807\n1\n",
            "overflow",
        ),
        ("이이이야하 우라 얏", "-9223372036854775808\n", "overflow"),
        (
            "이이이야하 이이이야하 핫",
            "9223372036854775807\n2\n",
            "overflow",
        ),
        (
            "이이이야하 이이이야하 우라 야하",
            "-9223372036854775808\n1\n",
            "overflow",
        ),
        ("이이이야하 이이이야하 우라 핫", "1\n0\n", "zero"),
        ("이이이야하 이이이야하 우라 핫핫", "1\n0\n", "zero"),
        (
            "이이이야하 이이이야하 우라 핫",
            "-9223372036854775808\n-1\n",
            "overflow",
        ),
        (
            "이이이야하 이이이야하 우라 핫핫",
            "-9223372036854775808\n-1\n",
            "overflow",
        ),
    ] {
        assert!(run(source, input).0.unwrap_err().message.contains(message));
    }
}

#[test]
fn errors_preserve_physical_source_lines_inside_blocks() {
    let error = run(
        "\n하야하 뿌루 야하\n이야 야다야다\n\n우라 뿌루\n야하\n우라 야다야다",
        "",
    )
    .0
    .unwrap_err();
    assert_eq!((error.line, error.instruction), (6, 1));
    let error = run(
        "하야하 뿌루 야하\n이야 야다야다\n우라 뿌루\n우라 야다야다",
        "",
    )
    .0
    .unwrap_err();
    assert_eq!(error.line, 2); // Rechecking the opening condition after the body.
}

#[test]
fn integer_input_requires_one_valid_i64_per_line() {
    assert_eq!(run("이이이야하 이이이야하 뿌루루", " +42 \r\n").1, "42");
    assert_eq!(
        run("이이이야하 이이이야하 야하 이이이야하 뿌루루", "4\n5\n").1,
        "9"
    );
    for input in ["", "  \n", "1 2\n", "9223372036854775808\n"] {
        assert!(run("이이이야하", input).0.is_err());
    }
}

#[test]
fn character_output_accepts_scalars_and_rejects_invalid_values() {
    assert_eq!(run("이이이야하 이이이야하 우라라", "128048\n").1, "🐰");
    for input in ["55296\n", "-1\n", "1114112\n"] {
        assert!(
            run("이이이야하 이이이야하 우라라", input)
                .0
                .unwrap_err()
                .message
                .contains("Unicode scalar")
        );
    }
}

#[test]
fn rejects_dashes_old_spellings_wrong_languages_and_punctuation() {
    for source in [
        "이-야-",
        "우-라-",
        "하야-하",
        "우이",
        "푸루",
        "후응",
        "야하!",
        "야하?",
        "Yeah",
        "(야하)",
        "우라\r",
    ] {
        assert!(malformed(source), "{source}");
    }
    assert!(lex("우라", Language::English).is_err());
    assert!(lex("ura", Language::Korean).is_err());
}

#[test]
fn parser_requires_a_single_final_eof() {
    assert!(parse(&[]).is_err());
    let eof = Token {
        kind: TokenKind::Eof,
        span: Span { start: 0, end: 0 },
    };
    assert!(parse(&[eof, eof]).is_err());
    assert!(
        parse(&[Token {
            kind: TokenKind::Yaha,
            ..eof
        }])
        .is_err()
    );
}

#[test]
fn empty_program_and_blank_lines_succeed() {
    for source in ["", "\n\n", "; ;\n"] {
        assert_eq!(run(source, "").0, Ok(()));
    }
}

#[test]
fn examples_produce_expected_outputs() {
    for (source, input, expected) in [
        (
            include_str!("../examples/hello_world.yaha"),
            "",
            "Hello, world!\n",
        ),
        (
            include_str!("../examples/add_two_numbers.yaha"),
            "12\n30\n",
            "42",
        ),
        (include_str!("../examples/integer_echo.yaha"), "-5\n", "-5"),
        (include_str!("../examples/array_access.yaha"), "", "2"),
        (include_str!("../examples/truth_machine.yaha"), "0\n", "0"),
    ] {
        assert_eq!(run(source, input), (Ok(()), expected.to_owned()));
    }
}
