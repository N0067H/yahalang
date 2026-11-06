//! Runs yahac, the Yahalang-to-C compiler written in Yahalang, on the interpreter.

use std::io::Cursor;

use yahalang::executor::{RuntimeError, execute};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::parse;

const YAHAC: &str = include_str!("../compiler/yahac.yaha");

fn yahac(source: &str) -> (Result<(), RuntimeError>, String) {
    let tokens = lex(YAHAC, Language::Korean).expect("yahac should lex");
    let program = parse(&tokens).expect("yahac should parse");
    let mut input = Cursor::new(source.as_bytes());
    let mut output = Vec::new();
    let result = execute(&program, &mut input, &mut output);
    (
        result,
        String::from_utf8(output).expect("C output is UTF-8"),
    )
}

fn body(c: &str) -> &str {
    let start = c.find("int main(void){\n").expect("C output has main") + 16;
    &c[start..]
}

#[test]
fn compiles_the_countdown_example_to_c() {
    let (result, c) = yahac(include_str!("../examples/countdown.yaha"));

    assert!(result.is_ok());
    assert!(c.starts_with(include_str!("../compiler/runtime.c")));
    assert_eq!(
        body(&c),
        "P(3);\nwhile(C()){\nU();\nO();\nP(1);\nS();\n}\nH();\nH();return 0;}\n"
    );
}

#[test]
fn detects_the_spelling_of_each_program() {
    for source in [
        "hayaha ppuruppuru yaha [hayaha ppuru yaha, hayaha yaha] ura hathat",
        "하야하 뿌루뿌루 야하 [하야하 뿌루 야하, 하야하 야하] 우라 핫핫",
        "ハヤハ プルプル ヤハ [ハヤハ プル ヤハ, ハヤハ ヤハ] ウラ ハッハッ",
    ] {
        let (result, c) = yahac(source);
        assert!(result.is_ok(), "{source}");
        assert_eq!(
            body(&c),
            "P(2);\n{static const Yi q[]={1,0};J(q,sizeof q/sizeof*q);}\nR();\nH();return 0;}\n"
        );
    }
}

#[test]
fn rejects_malformed_programs_without_output() {
    for source in [
        "hello",
        "hayaha ppuru",
        "ura yahayaha",
        "iya yadayada",
        "hayaha 야하",
        "[hayaha yaha,]",
        "hayaha ppuru yaha iya yahayaha ura iya ura yahayaha",
    ] {
        let (result, c) = yahac(source);
        assert!(result.is_err(), "{source}");
        assert!(c.is_empty(), "{source}");
    }
}
