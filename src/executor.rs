use std::cell::RefCell;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

use crate::parser::{Instruction, Program};

type Array = Rc<RefCell<Vec<i64>>>;
const MAX_INTEGER_INPUT_BYTES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Integer(i64),
    Array(Array),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub line: usize,
    pub instruction: usize,
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let is_input_error =
            self.message.contains("integer input") || self.message.starts_with("input");
        let reaction = if is_input_error { "ui" } else { "haa? haa?" };

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
    let instructions: Vec<_> = program
        .lines
        .iter()
        .enumerate()
        .flat_map(|(line, body)| {
            body.iter()
                .enumerate()
                .map(move |(index, instruction)| (line + 1, index + 1, instruction))
        })
        .collect();

    let mut pc = 0;

    while let Some(&(line, index, instruction)) = instructions.get(pc) {
        let flow = execute_instruction(instruction, &mut stack, input, output, line, index)?;
        pc = next_instruction(flow, pc, instructions.len()).map_err(|message| RuntimeError {
            line,
            instruction: index,
            message,
        })?;
    }

    Ok(())
}

fn next_instruction(flow: Flow, current: usize, length: usize) -> Result<usize, String> {
    match flow {
        Flow::Continue => Ok(current + 1),
        Flow::Halt => Ok(length),
        Flow::Jump(target) if target <= length => Ok(target),
        Flow::Jump(_) => Err("invalid instruction target".to_owned()),
    }
}

fn execute_instruction<R: BufRead, W: Write>(
    instruction: &Instruction,
    stack: &mut Vec<Value>,
    input: &mut R,
    output: &mut W,
    line: usize,
    instruction_number: usize,
) -> Result<Flow, RuntimeError> {
    let error = |message: String| RuntimeError {
        line,
        instruction: instruction_number,
        message,
    };

    match instruction {
        Instruction::Push(value) => stack.push(Value::Integer(*value)),
        Instruction::Add => binary(stack, BinaryOperation::Add).map_err(error)?,
        Instruction::Subtract => binary(stack, BinaryOperation::Subtract).map_err(error)?,
        Instruction::Multiply => binary(stack, BinaryOperation::Multiply).map_err(error)?,
        Instruction::Divide => binary(stack, BinaryOperation::Divide).map_err(error)?,
        Instruction::Remainder => binary(stack, BinaryOperation::Remainder).map_err(error)?,

        Instruction::Negate => {
            let value = stack
                .last()
                .ok_or_else(|| error("stack underflow".to_owned()))?
                .as_integer()
                .map_err(error)?;
            let result = value
                .checked_neg()
                .ok_or_else(|| error("arithmetic overflow".to_owned()))?;

            *stack.last_mut().expect("top value was checked") = Value::Integer(result);
        }

        Instruction::Duplicate => {
            let value = stack
                .last()
                .ok_or_else(|| error("stack underflow".to_owned()))?
                .clone();

            stack.push(value);
        }
        Instruction::Discard => {
            stack
                .pop()
                .ok_or_else(|| error("stack underflow".to_owned()))?;
        }

        Instruction::WhileStart(target) => {
            let condition = peek_integer(stack).map_err(error)?;
            return Ok(conditional_flow(condition, *target));
        }
        Instruction::WhileEnd(target) => return Ok(Flow::Jump(*target)),
        Instruction::IfStart(target) => {
            let condition = pop_integer(stack).map_err(error)?;
            return Ok(conditional_flow(condition, *target));
        }

        Instruction::Halt => return Ok(Flow::Halt),
        Instruction::ReadInteger => stack.push(Value::Integer(read_integer(input).map_err(error)?)),
        Instruction::PrintInteger => {
            let value = pop_integer(stack).map_err(error)?;
            write!(output, "{value}").map_err(|err| error(format!("output failed: {err}")))?;
            output
                .flush()
                .map_err(|err| error(format!("output flush failed: {err}")))?;
        }
        Instruction::PrintCharacter => {
            let value = pop_integer(stack).map_err(error)?;
            let codepoint = u32::try_from(value)
                .ok()
                .and_then(char::from_u32)
                .ok_or_else(|| error(format!("{value} is not a Unicode scalar value")))?;
            write!(output, "{codepoint}").map_err(|err| error(format!("output failed: {err}")))?;
            output
                .flush()
                .map_err(|err| error(format!("output flush failed: {err}")))?;
        }

        Instruction::ArrayLiteral(values) => {
            stack.push(Value::Array(Rc::new(RefCell::new(values.clone()))));
        }
        Instruction::ArrayGet => {
            let index = pop_integer(stack).map_err(error)?;
            let array = pop_array(stack).map_err(error)?;
            let index = usize::try_from(index)
                .map_err(|_| error(format!("array index {index} is out of bounds")))?;
            let value = array
                .borrow()
                .get(index)
                .copied()
                .ok_or_else(|| error(format!("array index {index} is out of bounds")))?;

            stack.push(Value::Integer(value));
        }
        Instruction::ArraySet => {
            let value = pop_integer(stack).map_err(error)?;
            let index = pop_integer(stack).map_err(error)?;
            let array = pop_array(stack).map_err(error)?;
            let index = usize::try_from(index)
                .map_err(|_| error(format!("array index {index} is out of bounds")))?;
            let mut values = array.borrow_mut();
            let element = values
                .get_mut(index)
                .ok_or_else(|| error(format!("array index {index} is out of bounds")))?;

            *element = value;
        }
        Instruction::ArrayLength => {
            let array = pop_array(stack).map_err(error)?;
            let length = array.borrow().len();
            stack.push(Value::Integer(length as i64));
        }
    }

    Ok(Flow::Continue)
}

fn conditional_flow(condition: i64, target: usize) -> Flow {
    match condition {
        0 => Flow::Jump(target),
        _ => Flow::Continue,
    }
}

fn peek_integer(stack: &[Value]) -> Result<i64, String> {
    stack
        .last()
        .ok_or_else(|| "stack underflow".to_owned())?
        .as_integer()
}

fn pop_integer(stack: &mut Vec<Value>) -> Result<i64, String> {
    match stack.pop().ok_or_else(|| "stack underflow".to_owned())? {
        Value::Integer(value) => Ok(value),
        Value::Array(_) => Err("expected an integer but found an array".to_owned()),
    }
}

fn pop_array(stack: &mut Vec<Value>) -> Result<Array, String> {
    match stack.pop().ok_or_else(|| "stack underflow".to_owned())? {
        Value::Array(array) => Ok(array),
        Value::Integer(_) => Err("expected an array but found an integer".to_owned()),
    }
}

impl Value {
    fn as_integer(&self) -> Result<i64, String> {
        match self {
            Self::Integer(value) => Ok(*value),
            Self::Array(_) => Err("expected an integer but found an array".to_owned()),
        }
    }
}

fn binary(stack: &mut Vec<Value>, operation: BinaryOperation) -> Result<(), String> {
    let right = stack
        .last()
        .ok_or_else(|| "stack underflow".to_owned())?
        .as_integer()?;
    let left = stack
        .get(
            stack
                .len()
                .checked_sub(2)
                .ok_or_else(|| "stack underflow".to_owned())?,
        )
        .ok_or_else(|| "stack underflow".to_owned())?
        .as_integer()?;

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
    stack.push(Value::Integer(result));

    Ok(())
}

fn read_integer<R: BufRead>(input: &mut R) -> Result<i64, String> {
    let mut bytes = Vec::new();
    let length = input
        .take((MAX_INTEGER_INPUT_BYTES + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|err| format!("input failed: {err}"))?;
    if length > MAX_INTEGER_INPUT_BYTES {
        return Err("integer input line is too long".to_owned());
    }
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
