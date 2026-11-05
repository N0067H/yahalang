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
            "이이이야하 이야 야하야하 이이이야하 이이이야하 뿌루루 우라\n야하 야하",
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
fn swap_spellings_produce_the_same_instruction() {
    let expected = compile("이야 뿌루");

    for (language, source) in [
        (Language::Korean, "이야 뿌루"),
        (Language::English, "iya ppuru"),
        (Language::Japanese, "イヤ プル"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
}

#[test]
fn swap_preserves_lower_values_and_reverses_operands() {
    let source =
        "이이이야하 이이이야하 이이이야하 이야 뿌루 우라 야하 이이이야하 뿌루루 이이이야하 뿌루루";

    assert_eq!(run(source, "9\n2\n7\n"), (Ok(()), "59".to_owned()));
}

#[test]
fn swap_keeps_array_aliases() {
    let source = "[하야하 야하] 뿌루 하야하 뿌루 야하 이야 뿌루 이야 뿌루 하야하 야하 이야 뿌루 = 하야하 야하 @ 이이이야하 뿌루루";

    assert_eq!(run(source, ""), (Ok(()), "1".to_owned()));
}

#[test]
fn swap_reports_underflow() {
    for source in ["이야 뿌루", "하야하 야하 이야 뿌루"] {
        assert_eq!(run(source, "").0.unwrap_err().message, "stack underflow");
    }
}

#[test]
fn swap_respects_phrase_boundaries_inside_blocks() {
    let source = "하야하 뿌루 야하 이야 야하야하 이이이야하 이이이야하 이야\n뿌루 우라 야하 이이이야하 뿌루루 우라 야하야하";

    assert_eq!(run(source, "2\n7\n"), (Ok(()), "5".to_owned()));
    assert!(malformed("이야 ; 뿌루"));
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

#[test]
fn comparisons_cover_order_equality_and_i64_extremes() {
    for (left, right, expected) in [
        (2, 7, "010"),
        (7, 2, "100"),
        (0, 0, "001"),
        (-7, -2, "010"),
        (i64::MIN, i64::MAX, "010"),
        (i64::MAX, i64::MIN, "100"),
        (i64::MIN, i64::MIN, "001"),
    ] {
        let source = "이이이야하 이이이야하 하아 야하 이이이야하 뿌루루 이이이야하 이이이야하 하아 이야 이이이야하 뿌루루 이이이야하 이이이야하 하아 핫 이이이야하 뿌루루";
        let input = format!("{left}\n{right}\n").repeat(3);
        assert_eq!(run(source, &input), (Ok(()), expected.to_owned()));
    }
}

#[test]
fn comparison_spellings_and_phrase_boundaries() {
    let expected = compile("하아 야하 하아 이야 하아 핫");
    assert_eq!(
        expected.lines[0],
        vec![
            yahalang::parser::Instruction::CompareGreater,
            yahalang::parser::Instruction::CompareLess,
            yahalang::parser::Instruction::CompareEqual,
        ]
    );
    for (language, source) in [
        (Language::Korean, "하아야하 하아\n이야 하아 핫"),
        (Language::English, "haayaha haa\niya haa hat"),
        (Language::Japanese, "ハアヤハ ハア\nイヤ ハア ハッ"),
    ] {
        assert_eq!(
            parse(&lex(source, language).unwrap())
                .unwrap()
                .lines
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            expected.lines.iter().flatten().cloned().collect::<Vec<_>>()
        );
    }
    for source in ["하아", "하아 ; 야하", "하아 뿌루", "하아 우라", "야다 야하"] {
        assert!(malformed(source), "{source}");
    }
}

#[test]
fn comparisons_check_operands_and_report_the_opening_line() {
    for operation in ["하아 야하", "하아 이야", "하아 핫"] {
        for prefix in ["", "하야하 야하", "[] 하야하 야하", "하야하 야하 []"] {
            let error = run(&format!("{prefix}\n{operation}"), "").0.unwrap_err();
            assert_eq!(error.line, 2);
            let expected = if prefix.contains('[') {
                "expected an integer"
            } else {
                "stack underflow"
            };
            assert!(error.message.contains(expected));
        }
    }
}

#[test]
fn comparison_results_feed_conditions_and_preserve_lower_values() {
    let source = "이이이야하 이이이야하 이이이야하 하아 이야 이야 야하야하 하야하 뿌루 야하 이이이야하 뿌루루 우라 야하야하 이이이야하 뿌루루";
    assert_eq!(run(source, "9\n2\n7\n"), (Ok(()), "19".to_owned()));
    assert_eq!(run(source, "9\n7\n2\n"), (Ok(()), "9".to_owned()));
    assert_eq!(
        run(include_str!("../examples/compare.yaha"), "2\n7\n"),
        (Ok(()), "1".to_owned())
    );
}

#[test]
fn character_input_spellings_and_incomplete_phrases() {
    let expected = compile("우라라 이이이야하");
    assert_eq!(
        expected.lines[0],
        vec![yahalang::parser::Instruction::ReadCharacter]
    );
    for (language, source) in [
        (Language::Korean, "우라라이이이야하"),
        (Language::English, "uraraiiiiyaha"),
        (Language::Japanese, "ウラライイイヤハ"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
    assert!(
        parse(&lex("우라라", Language::Korean).unwrap())
            .unwrap_err()
            .incomplete
    );
    assert!(malformed("우라라 ; 이이이야하"));
    assert!(malformed("우라라 뿌루"));
    let (result, output) = run("우라라\n이이이야하 이이이야하 뿌루루", "가");
    assert_eq!((result, output), (Ok(()), "44032".to_owned()));
}

#[test]
fn character_input_reads_scalars_without_consuming_following_bytes() {
    for character in ['A', 'é', '가', '🦀', '\0', '\u{10ffff}', '\r', '\n', ' '] {
        let source = "우라라 이이이야하 이이이야하 뿌루루 우라라 이이이야하 이이이야하 뿌루루";
        let expected = format!("{}90", u32::from(character));
        assert_eq!(run(source, &format!("{character}Z")), (Ok(()), expected));
    }
    assert_eq!(
        run("우라라 이이이야하 이이이야하 뿌루루", ""),
        (Ok(()), "-1".to_owned())
    );
    assert_eq!(
        run("우라라 이이이야하 이이이야하 이이이야하 뿌루루", "A42\n"),
        (Ok(()), "42".to_owned())
    );
}

#[test]
fn character_input_rejects_invalid_and_truncated_utf8() {
    let program = compile("\n우라라 이이이야하");
    for bytes in [
        &b"\xff"[..],
        &b"\x80"[..],
        &b"\xc0\x80"[..],
        &b"\xe0\x80\x80"[..],
        &b"\xed\xa0\x80"[..],
        &b"\xf4\x90\x80\x80"[..],
        &b"\xe3\x81"[..],
    ] {
        let mut input = std::io::BufReader::with_capacity(1, Cursor::new(bytes));
        let error = execute(&program, &mut input, &mut Vec::new()).unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("UTF-8"), "{error:?}");
    }
    let mut input = std::io::BufReader::with_capacity(1, Cursor::new("가".as_bytes()));
    execute(&program, &mut input, &mut Vec::new()).unwrap();
}

#[test]
fn character_echo_handles_unicode_nul_and_eof() {
    let source = include_str!("../examples/character_echo.yaha");
    for input in ["", "Hello, world!\n", "한글 🦀\r\n", "\0a"] {
        assert_eq!(run(source, input), (Ok(()), input.to_owned()));
    }
}

#[test]
fn array_append_and_pop_spellings_and_boundaries() {
    let expected = compile("야다 뿌루 야다 우라");
    assert_eq!(
        expected.lines[0],
        vec![
            yahalang::parser::Instruction::ArrayAppend,
            yahalang::parser::Instruction::ArrayPop,
        ]
    );
    for (language, source) in [
        (Language::Korean, "야다뿌루 야다우라"),
        (Language::English, "yadappuru yadaura"),
        (Language::Japanese, "ヤダプル ヤダウラ"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
    for source in ["야다 ; 뿌루", "야다 얏", "야다 야하"] {
        assert!(malformed(source), "{source}");
    }
    assert!(
        parse(&lex("야다", Language::Korean).unwrap())
            .unwrap_err()
            .incomplete
    );
    let source = "[] 하야하 뿌루 야하 야다\n뿌루 야다\n우라 이이이야하 뿌루루 # 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "10".to_owned()));
}

#[test]
fn array_append_and_pop_update_shared_aliases() {
    let source = "[] 뿌루 하야하 뿌루 야하 야다 뿌루 이야 뿌루 야다 우라 이이이야하 뿌루루 # 이이이야하 뿌루루 # 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "100".to_owned()));
    assert_eq!(
        run(include_str!("../examples/array_push_pop.yaha"), ""),
        (Ok(()), "21".to_owned())
    );
}

#[test]
fn array_append_and_pop_preserve_state_on_errors() {
    use yahalang::executor::ExecutionState;
    for (setup, operation, expected_stack, message) in [
        ("", "야다 뿌루", "[]", "stack underflow"),
        ("[]", "야다 뿌루", "[[]]", "stack underflow"),
        (
            "하야하 야하 하야하 뿌루 야하",
            "야다 뿌루",
            "[0, 1]",
            "expected an array",
        ),
        ("[] []", "야다 뿌루", "[[], []]", "expected an integer"),
        ("", "야다 우라", "[]", "stack underflow"),
        ("하야하 야하", "야다 우라", "[0]", "expected an array"),
        ("[] 뿌루", "야다 우라", "[[], []]", "empty array"),
    ] {
        let mut state = ExecutionState::default();
        state
            .execute(&compile(setup), &mut Cursor::new(b""), &mut Vec::new())
            .unwrap();
        let error = state
            .execute(
                &compile(&format!("\n{operation}")),
                &mut Cursor::new(b""),
                &mut Vec::new(),
            )
            .unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains(message), "{error:?}");
        assert_eq!(state.stack_display(), expected_stack);
    }
}

#[test]
fn reverse_input_handles_empty_unicode_and_nul() {
    for input in ["", "abc", "가나🦀", "\0a\n"] {
        let expected: String = input.chars().rev().collect();
        assert_eq!(
            run(include_str!("../examples/reverse_input.yaha"), input),
            (Ok(()), expected)
        );
    }
}

#[test]
fn spoken_array_creation_spellings_and_boundaries() {
    let expected = compile("[]");
    for (language, source) in [
        (Language::Korean, "야다하야하"),
        (Language::English, "yadahayaha"),
        (Language::Japanese, "ヤダハヤハ"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
    assert!(malformed("야다 ; 하야하"));
    assert_eq!(
        run("야다\n하야하 # 이이이야하 뿌루루", ""),
        (Ok(()), "0".to_owned())
    );
}

#[test]
fn spoken_array_creation_allocates_independent_arrays_each_time() {
    use yahalang::executor::ExecutionState;
    let program = compile("야다 하야하");
    let mut state = ExecutionState::default();
    for _ in 0..2 {
        state
            .execute(&program, &mut Cursor::new(b""), &mut Vec::new())
            .unwrap();
    }
    state
        .execute(
            &compile("하야하 뿌루 야하 야다 뿌루"),
            &mut Cursor::new(b""),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(state.stack_display(), "[[], [1]]");
}

#[test]
fn spoken_array_access_spellings_match_existing_operations() {
    let expected = compile("# @ =");
    for (language, source) in [
        (Language::Korean, "야다뿌루루 야다이야 야다핫"),
        (Language::English, "yadappururu yadaiya yadahat"),
        (Language::Japanese, "ヤダプルル ヤダイヤ ヤダハッ"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
    let source = "야다 하야하 하야하 뿌루 야하 야다 뿌루 뿌루 야다 뿌루루 이이이야하 뿌루루 뿌루 하야하 야하 하야하 뿌루뿌루 야하 야다 핫 하야하 야하 야다 이야 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "12".to_owned()));
    for source in ["야다 ; 뿌루루", "야다 ; 이야", "야다 ; 핫"] {
        assert!(malformed(source), "{source}");
    }
}

#[test]
fn spoken_array_access_preserves_existing_errors() {
    for (source, expected) in [
        ("야다 이야", "stack underflow"),
        ("하야하 야하 야다 뿌루루", "expected an array"),
        ("야다 하야하 하야하 야하 야다 이야", "out of bounds"),
        (
            "야다 하야하 하야하 야하 하야하 뿌루 야하 야다 핫",
            "out of bounds",
        ),
    ] {
        assert!(
            run(source, "").0.unwrap_err().message.contains(expected),
            "{source}"
        );
    }
}

#[test]
fn loop_break_spellings_and_phrase_boundaries() {
    let expected = compile("이야 야다야다 우라 이야 우라 야다야다");
    for (language, source) in [
        (Language::Korean, "이야야다야다 우라이야 우라야다야다"),
        (Language::English, "iyayadayada uraiya urayadayada"),
        (Language::Japanese, "イヤヤダヤダ ウライヤ ウラヤダヤダ"),
    ] {
        assert_eq!(parse(&lex(source, language).unwrap()).unwrap(), expected);
    }
    let source = "하야하 뿌루 야하 이야 야다야다 우라\n이야 우라 야다야다 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "1".to_owned()));
    assert!(malformed("이야 야다야다 우라 ; 이야 우라 야다야다"));
}

#[test]
fn loop_break_rejects_use_outside_loops_even_in_unreachable_code() {
    for source in [
        "우라 이야",
        "이야 야하야하 우라 이야 우라 야하야하",
        "얏 우라 이야",
        "이야 야다야다 우라 야다야다 우라 이야",
    ] {
        let error = parse(&lex(source, Language::Korean).unwrap()).unwrap_err();
        assert_eq!(error.message, "loop break is only valid inside a loop");
        assert!(!error.incomplete);
    }
}

#[test]
fn loop_break_preserves_stack_and_skips_remaining_body() {
    let source = "하야하 뿌루 야하 이야 야다야다 우라 뿌루 우라 이야 야하 우라 야다야다 하야하 뿌루뿌루 야하 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "2".to_owned()));
    let source = "하야하 뿌루 야하 이야 야다야다 우라 뿌루 야다 하야하 우라 이야 야하 우라 야다야다 야다 뿌루루 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "0".to_owned()));
}

#[test]
fn loop_break_only_exits_the_nearest_loop_inside_conditions() {
    let source = "하야하 뿌루뿌루 야하 이야 야다야다 하야하 뿌루 야하 이야 야다야다 하야하 뿌루 야하 이야 야하야하 우라 이야 우라 야하야하 야하 우라 야다야다 이이이야하 뿌루루 하야하 뿌루 야하 우라 야하 우라 야다야다 우라 뿌루";
    assert_eq!(run(source, ""), (Ok(()), "11".to_owned()));
    assert_eq!(
        run(include_str!("../examples/break_on_zero.yaha"), "3\n2\n0\n"),
        (Ok(()), "32".to_owned())
    );
}

#[test]
fn skipped_loop_breaks_and_invalid_targets_are_checked() {
    let source = "하야하 야하 이야 야다야다 우라 이야 우라 야다야다 이이이야하 뿌루루";
    assert_eq!(run(source, ""), (Ok(()), "0".to_owned()));
    let program = Program {
        lines: vec![vec![yahalang::parser::Instruction::Break(2)]],
    };
    let error = execute(&program, &mut Cursor::new(b""), &mut Vec::new()).unwrap_err();
    assert_eq!(error.message, "invalid instruction target");
}
