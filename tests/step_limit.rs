use std::io::Cursor;

use yahalang::executor::{ExecutionState, TraceEvent, TraceStatus};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::{Program, parse};

fn program(source: &str) -> Program {
    parse(&lex(source, Language::English).unwrap()).unwrap()
}

#[test]
fn exact_budget_succeeds_and_the_next_instruction_has_no_effect() {
    let program = program("hayaha ppuru yaha ppuru yaha iiiiyaha ppururu");
    let mut state = ExecutionState::default();
    state.set_step_limit(Some(3));
    let mut output = Vec::new();
    let error = state
        .execute(&program, &mut Cursor::new(b""), &mut output)
        .unwrap_err();
    assert_eq!(error.instruction, 4);
    assert_eq!(error.message, "execution step limit exceeded");
    assert_eq!(state.stack_display(), "[2]");
    assert!(output.is_empty());

    state.reset();
    state.set_step_limit(Some(4));
    state
        .execute(&program, &mut Cursor::new(b""), &mut output)
        .unwrap();
    assert_eq!(output, b"2");
}

#[test]
fn zero_budget_allows_empty_source_but_does_not_read_input() {
    let mut state = ExecutionState::default();
    state.set_step_limit(Some(0));
    let mut input = Cursor::new(b"7\n");
    state
        .execute(&program(""), &mut input, &mut Vec::new())
        .unwrap();
    assert!(
        state
            .execute(&program("iiiiyaha"), &mut input, &mut Vec::new())
            .is_err()
    );
    assert_eq!(input.position(), 0);
    assert_eq!(state.stack_display(), "[]");
}

#[test]
fn empty_loop_is_bounded_and_trace_reports_the_blocked_instruction() {
    let mut state = ExecutionState::default();
    state.set_step_limit(Some(4));
    let mut events = Vec::new();
    let observer = |event: TraceEvent<'_>| {
        events.push((event.offset, format!("{:?}", event.status), event.stack));
        Ok(())
    };
    let error = state
        .execute_with_trace(
            &program("hayaha ppuru yaha\niya yadayada\nura yadayada"),
            &mut Cursor::new(b""),
            &mut Vec::new(),
            observer,
        )
        .unwrap_err();
    assert_eq!(error.line, 3);
    assert_eq!(events.len(), 9);
    assert_eq!(events.last().unwrap().0, 2);
    assert_eq!(
        events.last().unwrap().1,
        format!("{:?}", TraceStatus::Error("execution step limit exceeded"))
    );
    assert_eq!(state.stack_display(), "[1]");
}

#[test]
fn skipped_instructions_and_code_after_halt_do_not_use_budget() {
    let mut state = ExecutionState::default();
    state.set_step_limit(Some(3));
    state
        .execute(
            &program("hayaha yaha iya yahayaha yaha ura yahayaha yat yaha"),
            &mut Cursor::new(b""),
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(state.stack_display(), "[]");
}

#[test]
fn each_execution_gets_a_new_budget_even_after_reset_or_failure() {
    let mut state = ExecutionState::default();
    state.set_step_limit(Some(1));
    let push = program("hayaha ppuru yaha");
    for _ in 0..2 {
        state
            .execute(&push, &mut Cursor::new(b""), &mut Vec::new())
            .unwrap();
    }
    assert_eq!(state.stack_display(), "[1, 1]");
    state.reset();
    assert!(
        state
            .execute(&program("ppuru"), &mut Cursor::new(b""), &mut Vec::new())
            .is_err()
    );
    state
        .execute(&push, &mut Cursor::new(b""), &mut Vec::new())
        .unwrap();
    assert_eq!(state.stack_display(), "[1]");
}
