use std::io::{self, Cursor};

use yahalang::executor::{ExecutionState, TraceEvent, TraceStatus};
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::{Instruction, parse, parse_with_spans};

#[test]
fn source_spans_distinguish_commands_and_multiline_phrases() {
    let source = "  하야하\n 뿌루 야하 ; 뿌루\n이야 야다야다\n우라\n이야\n우라\n야다야다";
    let tokens = lex(source, Language::Korean).unwrap();
    let (program, spans) = parse_with_spans(&tokens).unwrap();
    assert_eq!(program, parse(&tokens).unwrap());
    let phrases: Vec<_> = spans
        .iter()
        .map(|span| &source[span.start..span.end])
        .collect();
    assert_eq!(
        phrases,
        vec![
            "하야하\n 뿌루 야하",
            "뿌루",
            "이야 야다야다",
            "우라\n이야",
            "우라\n야다야다"
        ]
    );
    assert_eq!(
        spans.len(),
        program.lines.iter().map(Vec::len).sum::<usize>()
    );
}

#[test]
fn trace_preserves_execution_and_records_stack_before_and_after() {
    let source = "하야하 뿌루 야하 뿌루 야하 이이이야하 뿌루루 얏 야하";
    let program = parse(&lex(source, Language::Korean).unwrap()).unwrap();
    let mut state = ExecutionState::default();
    let mut output = Vec::new();
    let mut events = Vec::new();
    state
        .execute_with_trace(&program, &mut Cursor::new(b""), &mut output, |event| {
            events.push((event.offset, format!("{:?}", event.status), event.stack));
            Ok(())
        })
        .unwrap();
    assert_eq!(output, b"2");
    assert_eq!(state.stack_display(), "[]");
    assert_eq!(events.len(), 10);
    assert_eq!(events[0], (0, "Before".to_owned(), "[]".to_owned()));
    assert_eq!(events[1], (0, "After".to_owned(), "[1]".to_owned()));
    assert_eq!(events[4], (2, "Before".to_owned(), "[1, 1]".to_owned()));
    assert_eq!(events[5], (2, "After".to_owned(), "[2]".to_owned()));
    assert_eq!(events.last().unwrap().0, 4);
}

#[test]
fn trace_visits_only_executed_branches_and_repeated_loop_checks() {
    let source = "하야하 뿌루뿌루 야하 이야 야다야다 하야하 야하 이야 야하야하 얏 우라 야하야하 하야하 뿌루 야하 우라 야하 우라 야다야다";
    let program = parse(&lex(source, Language::Korean).unwrap()).unwrap();
    let mut state = ExecutionState::default();
    let mut operations = Vec::new();
    let observer = |event: TraceEvent<'_>| {
        if event.status == TraceStatus::Before {
            operations.push(event.operation.clone());
        }
        Ok(())
    };
    state
        .execute_with_trace(&program, &mut Cursor::new(b""), &mut Vec::new(), observer)
        .unwrap();
    assert_eq!(
        operations
            .iter()
            .filter(|op| matches!(op, Instruction::WhileStart(_)))
            .count(),
        3
    );
    assert_eq!(
        operations
            .iter()
            .filter(|op| matches!(op, Instruction::WhileEnd(_)))
            .count(),
        2
    );
    assert!(!operations.contains(&Instruction::Halt));
    assert_eq!(state.stack_display(), "[0]");
}

#[test]
fn trace_array_snapshots_do_not_change_after_shared_mutation() {
    let source = "야다 하야하 뿌루 하야하 뿌루 야하 야다 뿌루";
    let program = parse(&lex(source, Language::Korean).unwrap()).unwrap();
    let mut snapshots = Vec::new();
    ExecutionState::default()
        .execute_with_trace(&program, &mut Cursor::new(b""), &mut Vec::new(), |event| {
            snapshots.push(event.stack);
            Ok(())
        })
        .unwrap();
    assert_eq!(snapshots[3], "[[], []]");
    assert_eq!(snapshots.last().unwrap(), "[[1], [1]]");
}

#[test]
fn trace_records_error_state_after_operands_have_been_consumed() {
    let program =
        parse(&lex("하야하 뿌루 야하\n하야하 야하 @", Language::Korean).unwrap()).unwrap();
    let mut events = Vec::new();
    let error = ExecutionState::default()
        .execute_with_trace(&program, &mut Cursor::new(b""), &mut Vec::new(), |event| {
            events.push((
                event.line,
                event.instruction,
                format!("{:?}", event.status),
                event.stack,
            ));
            Ok(())
        })
        .unwrap_err();
    assert!(error.message.contains("expected an array"));
    let last = events.last().unwrap();
    assert_eq!((last.0, last.1), (2, 2));
    assert!(last.2.starts_with("Error("));
    assert_eq!(last.3, "[]");
}

#[test]
fn trace_failure_stops_execution_at_the_observed_point() {
    let program = parse(&lex("하야하 뿌루 야하", Language::Korean).unwrap()).unwrap();
    for fail_before in [true, false] {
        let mut state = ExecutionState::default();
        let observer =
            |event: TraceEvent<'_>| match (event.status == TraceStatus::Before) == fail_before {
                true => Err(io::Error::other("trace writer unavailable")),
                false => Ok(()),
            };
        let error = state
            .execute_with_trace(&program, &mut Cursor::new(b""), &mut Vec::new(), observer)
            .unwrap_err();
        assert!(error.message.contains("trace output failed"));
        assert_eq!(
            state.stack_display(),
            if fail_before { "[]" } else { "[1]" }
        );
    }
}

#[test]
fn trace_notifies_before_consuming_program_input() {
    let program = parse(&lex("이이이야하", Language::Korean).unwrap()).unwrap();
    let mut input = Cursor::new(b"7\n");
    let error = ExecutionState::default()
        .execute_with_trace(&program, &mut input, &mut Vec::new(), |event| {
            assert_eq!(event.status, TraceStatus::Before);
            Err(io::Error::other("stop before input"))
        })
        .unwrap_err();
    assert!(error.message.contains("stop before input"));
    assert_eq!(input.position(), 0);
}
