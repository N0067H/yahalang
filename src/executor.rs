use std::io::{BufRead, Write};

use crate::parser::{Instruction, Program};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub line: usize,
    pub instruction: usize,
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reaction =
            if self.message.contains("integer input") || self.message.starts_with("input") {
                "ui"
            } else {
                "haa? haa?"
            };
        f.write_str(reaction)
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Jump(usize),
    Halt,
}

#[derive(Clone, Copy)]
enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

pub fn execute<R: BufRead, W: Write>(
    program: &Program,
    input: &mut R,
    output: &mut W,
) -> Result<(), RuntimeError> {
    let mut stack = Vec::new();
    let mut line_index = 0;

    while line_index < program.lines.len() {
        let mut instruction_index = 0;
        let mut jumped = false;

        while let Some(instruction) = program.lines[line_index].get(instruction_index) {
            let flow = execute_instruction(
                instruction,
                &mut stack,
                input,
                output,
                program.lines.len(),
                line_index + 1,
                instruction_index + 1,
            )?;

            match flow {
                Flow::Continue => instruction_index += 1,
                Flow::Jump(target) => {
                    line_index = target - 1;
                    jumped = true;
                    break;
                }
                Flow::Halt => return Ok(()),
            }
        }

        if !jumped {
            line_index += 1;
        }
    }

    Ok(())
}

fn execute_instruction<R: BufRead, W: Write>(
    instruction: &Instruction,
    stack: &mut Vec<i64>,
    input: &mut R,
    output: &mut W,
    line_count: usize,
    line: usize,
    instruction_number: usize,
) -> Result<Flow, RuntimeError> {
    let error = |message: String| RuntimeError {
        line,
        instruction: instruction_number,
        message,
    };

    match instruction {
        Instruction::Push(value) => stack.push(*value),
        Instruction::Add => binary(stack, BinaryOperation::Add).map_err(error)?,
        Instruction::Subtract => binary(stack, BinaryOperation::Subtract).map_err(error)?,
        Instruction::Multiply => binary(stack, BinaryOperation::Multiply).map_err(error)?,
        Instruction::Divide => binary(stack, BinaryOperation::Divide).map_err(error)?,
        Instruction::Remainder => binary(stack, BinaryOperation::Remainder).map_err(error)?,
        Instruction::Negate => {
            let value = *stack
                .last()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            let result = value
                .checked_neg()
                .ok_or_else(|| error("arithmetic overflow".to_owned()))?;
            *stack.last_mut().expect("top value was checked") = result;
        }
        Instruction::Duplicate => {
            let value = *stack
                .last()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            stack.push(value);
        }
        Instruction::Discard => {
            stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
        }
        Instruction::Conditional(body) => {
            let condition = stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            if condition == 0 {
                return execute_instruction(
                    body,
                    stack,
                    input,
                    output,
                    line_count,
                    line,
                    instruction_number,
                );
            }
        }
        Instruction::Jump => {
            let target = stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            if target < 1 || target > line_count as i64 {
                return Err(error(format!(
                    "jump target {target} is outside 1..={line_count}"
                )));
            }
            return Ok(Flow::Jump(target as usize));
        }
        Instruction::Halt => return Ok(Flow::Halt),
        Instruction::ReadInteger => stack.push(read_integer(input).map_err(error)?),
        Instruction::PrintInteger => {
            let value = stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            write!(output, "{value}").map_err(|err| error(format!("output failed: {err}")))?;
            output
                .flush()
                .map_err(|err| error(format!("output flush failed: {err}")))?;
        }
        Instruction::PrintCharacter => {
            let value = stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
            let codepoint = u32::try_from(value)
                .ok()
                .and_then(char::from_u32)
                .ok_or_else(|| error(format!("{value} is not a Unicode scalar value")))?;
            write!(output, "{codepoint}").map_err(|err| error(format!("output failed: {err}")))?;
            output
                .flush()
                .map_err(|err| error(format!("output flush failed: {err}")))?;
        }
    }

    Ok(Flow::Continue)
}

fn binary(stack: &mut Vec<i64>, operation: BinaryOperation) -> Result<(), String> {
    let right = *stack.last().ok_or_else(|| "stack underflow".to_owned())?;
    let left = *stack
        .get(
            stack
                .len()
                .checked_sub(2)
                .ok_or_else(|| "stack underflow".to_owned())?,
        )
        .ok_or_else(|| "stack underflow".to_owned())?;

    if matches!(
        operation,
        BinaryOperation::Divide | BinaryOperation::Remainder
    ) && right == 0
    {
        return Err("division by zero".to_owned());
    }

    let result = match operation {
        BinaryOperation::Add => left.checked_add(right),
        BinaryOperation::Subtract => left.checked_sub(right),
        BinaryOperation::Multiply => left.checked_mul(right),
        BinaryOperation::Divide => left.checked_div(right),
        BinaryOperation::Remainder => left.checked_rem(right),
    }
    .ok_or_else(|| "arithmetic overflow".to_owned())?;
    stack.truncate(stack.len() - 2);
    stack.push(result);
    Ok(())
}

fn read_integer<R: BufRead>(input: &mut R) -> Result<i64, String> {
    let mut bytes = Vec::new();
    let length = input
        .read_until(b'\n', &mut bytes)
        .map_err(|err| format!("input failed: {err}"))?;
    if length == 0 {
        return Err("integer input reached EOF".to_owned());
    }

    let line = std::str::from_utf8(&bytes).map_err(|err| format!("input is not UTF-8: {err}"))?;
    let value = line.trim_matches(|character: char| character.is_ascii_whitespace());
    if value.is_empty() {
        return Err("integer input is empty".to_owned());
    }

    value
        .parse::<i64>()
        .map_err(|err| format!("invalid integer input: {err}"))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::language::Language;
    use crate::lexer::lex;
    use crate::parser::parse;

    use super::execute;

    fn run(source: &str, input: &str) -> (Result<(), super::RuntimeError>, String) {
        let tokens = lex(source, Language::English).unwrap();
        let program = parse(&tokens).unwrap();
        let mut input = Cursor::new(input.as_bytes());
        let mut output = Vec::new();
        let result = execute(&program, &mut input, &mut output);
        (result, String::from_utf8(output).unwrap())
    }

    #[test]
    fn reads_adds_and_prints_without_extra_newlines() {
        assert_eq!(run("ui ui yaha i", "12\n30\n"), (Ok(()), "42".to_owned()));
    }

    #[test]
    fn evaluates_arithmetic_and_stack_operations() {
        assert_eq!(
            run("ura yahayaha ura yahayaha huung i", ""),
            (Ok(()), "4".to_owned())
        );
    }

    #[test]
    fn conditionals_jump_and_halt_control_execution() {
        let source = "ura yahayahayaha puruya\nura yaha i\nura haa? puruya!\nui";
        assert_eq!(run(source, ""), (Ok(()), String::new()));
    }

    #[test]
    fn reports_underflow_overflow_and_invalid_jumps() {
        assert!(run("yaha", "").0.unwrap_err().message.contains("underflow"));
        assert!(
            run("ui fuun", "-9223372036854775808\n")
                .0
                .unwrap_err()
                .message
                .contains("overflow")
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
    fn reads_only_complete_signed_integer_lines() {
        assert_eq!(run("ui i", " +42 \r\n"), (Ok(()), "42".to_owned()));
        assert!(run("ui", "1 2\n").0.is_err());
        assert!(run("ui", "").0.is_err());
    }
}
