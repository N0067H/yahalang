use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub lines: Vec<Vec<Instruction>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Push(i64),
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Negate,
    Duplicate,
    Discard,
    Conditional(Box<Instruction>),
    Jump,
    Halt,
    ReadInteger,
    PrintInteger,
    PrintCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("haa?")
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: &[Token]) -> Result<Program, ParseError> {
    if !tokens
        .last()
        .is_some_and(|token| token.kind == TokenKind::Eof)
    {
        return Err(ParseError {
            span: tokens
                .last()
                .map_or(Span { start: 0, end: 0 }, |token| token.span),
            message: "토큰 끝이 없어".to_owned(),
        });
    };

    if let Some(token) = tokens[..tokens.len() - 1]
        .iter()
        .find(|token| token.kind == TokenKind::Eof)
    {
        return Err(ParseError {
            span: token.span,
            message: "토큰 중간에 끝이 나왔어".to_owned(),
        });
    }

    Parser::new(tokens).parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn parse_program(mut self) -> Result<Program, ParseError> {
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut line_has_source = false;

        while !self.at(TokenKind::Eof) {
            match self.current().kind {
                TokenKind::Newline => {
                    self.advance();
                    lines.push(std::mem::take(&mut line));
                    line_has_source = false;
                }
                TokenKind::HorizontalWhitespace => {
                    self.advance();
                    line_has_source = true;
                }
                _ => {
                    line_has_source = true;
                    line.push(self.parse_instruction()?);
                }
            }
        }

        if line_has_source {
            lines.push(line);
        }

        Ok(Program { lines })
    }

    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        match self.current().kind {
            TokenKind::Ura => self.parse_number(),
            TokenKind::Yaha => {
                let token = self.advance();
                if self.take_adjacent(TokenKind::Bang, token.span) {
                    Ok(Instruction::Subtract)
                } else {
                    Ok(Instruction::Add)
                }
            }
            TokenKind::Ha => self.parse_conditional(),
            TokenKind::Puru => {
                let token = self.advance();
                if self.take_adjacent(TokenKind::Bang, token.span) {
                    Ok(Instruction::Discard)
                } else {
                    Ok(Instruction::Duplicate)
                }
            }
            TokenKind::Puruya => {
                let token = self.advance();
                if self.take_adjacent(TokenKind::Bang, token.span) {
                    Ok(Instruction::Halt)
                } else {
                    Ok(Instruction::Jump)
                }
            }
            TokenKind::Huung => self.consume(Instruction::Multiply),
            TokenKind::Hueung => self.consume(Instruction::Divide),
            TokenKind::Jureuri => self.consume(Instruction::Remainder),
            TokenKind::Fuun => self.consume(Instruction::Negate),
            TokenKind::Ui => self.consume(Instruction::ReadInteger),
            TokenKind::I => self.consume(Instruction::PrintInteger),
            TokenKind::Ua => self.consume(Instruction::PrintCharacter),
            _ => Err(self.unexpected("instruction")),
        }
    }

    fn parse_number(&mut self) -> Result<Instruction, ParseError> {
        self.advance();
        self.skip_horizontal_whitespace();

        if !self.at(TokenKind::Yaha) || self.next_is_adjacent_bang() {
            return Ok(Instruction::Push(0));
        }

        let mut count = 0_i64;
        let mut previous: Option<Span> = None;

        while self.at(TokenKind::Yaha) {
            let token = self.current();
            if self.is_adjacent_bang(token.span)
                || previous.is_some_and(|span| span.end != token.span.start)
            {
                break;
            }

            self.advance();
            count = count.checked_add(1).ok_or_else(|| ParseError {
                span: token.span,
                message: "숫자 반복이 너무 길어".to_owned(),
            })?;
            previous = Some(token.span);
        }

        Ok(Instruction::Push(count))
    }

    fn parse_conditional(&mut self) -> Result<Instruction, ParseError> {
        let marker = self.advance();
        if !self.take_adjacent(TokenKind::Question, marker.span) {
            return Err(ParseError {
                span: marker.span,
                message: "조건 표현 뒤에 바로 `?`가 와야 해".to_owned(),
            });
        }

        self.skip_horizontal_whitespace();
        if self.at(TokenKind::Newline) || self.at(TokenKind::Eof) {
            return Err(ParseError {
                span: marker.span,
                message: "조건 뒤에 실행할 명령이 없어".to_owned(),
            });
        }

        Ok(Instruction::Conditional(Box::new(
            self.parse_instruction()?,
        )))
    }

    fn consume(&mut self, instruction: Instruction) -> Result<Instruction, ParseError> {
        self.advance();
        Ok(instruction)
    }

    fn take_adjacent(&mut self, kind: TokenKind, previous: Span) -> bool {
        if self.at(kind) && self.current().span.start == previous.end {
            self.advance();
            true
        } else {
            false
        }
    }

    fn next_is_adjacent_bang(&self) -> bool {
        let current = self.current();
        self.tokens
            .get(self.cursor + 1)
            .is_some_and(|next| next.kind == TokenKind::Bang && next.span.start == current.span.end)
    }

    fn is_adjacent_bang(&self, span: Span) -> bool {
        self.tokens
            .get(self.cursor + 1)
            .is_some_and(|next| next.kind == TokenKind::Bang && next.span.start == span.end)
    }

    fn skip_horizontal_whitespace(&mut self) {
        while self.at(TokenKind::HorizontalWhitespace) {
            self.advance();
        }
    }

    fn unexpected(&self, expected: &str) -> ParseError {
        ParseError {
            span: self.current().span,
            message: format!(
                "{expected}이 와야 하는데 {:?} 토큰이 나왔어",
                self.current().kind
            ),
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn current(&self) -> Token {
        self.tokens
            .get(self.cursor)
            .copied()
            .unwrap_or_else(|| *self.tokens.last().expect("lexer always emits EOF"))
    }

    fn advance(&mut self) -> Token {
        let token = self.current();
        if token.kind != TokenKind::Eof {
            self.cursor += 1;
        }
        token
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;
    use crate::lexer::lex;

    use super::{Instruction, parse};

    fn parse_english(source: &str) -> super::Program {
        parse(&lex(source, Language::English).unwrap()).unwrap()
    }

    #[test]
    fn parses_numbers_using_spaces_and_adjacency() {
        assert_eq!(
            parse_english("ura yahayaha\nura yaha yaha").lines,
            vec![
                vec![Instruction::Push(2)],
                vec![Instruction::Push(1), Instruction::Add]
            ]
        );
    }

    #[test]
    fn parses_suffixes_and_nested_conditionals() {
        assert_eq!(
            parse_english("haa? haa? yaha! puruya!").lines,
            vec![vec![
                Instruction::Conditional(Box::new(Instruction::Conditional(Box::new(
                    Instruction::Subtract
                )))),
                Instruction::Halt
            ]]
        );
    }

    #[test]
    fn preserves_blank_physical_lines_without_adding_trailing_line() {
        assert_eq!(
            parse_english("\nui\n\n").lines,
            vec![vec![], vec![Instruction::ReadInteger], vec![]]
        );
    }

    #[test]
    fn rejects_detached_suffixes_and_missing_conditional_body() {
        assert!(parse(&lex("yaha !", Language::English).unwrap()).is_err());
        assert!(parse(&lex("haa?\nui", Language::English).unwrap()).is_err());
        assert!(parse(&lex("haa", Language::English).unwrap()).is_err());
    }
}
