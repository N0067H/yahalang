use crate::token::{Span, Token, TokenKind};

const MAX_BLOCK_DEPTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Instructions grouped by physical source line, for runtime diagnostics.
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
    /// Flat instruction offset after the matching loop end.
    WhileStart(usize),
    /// Flat instruction offset of the matching loop start.
    WhileEnd(usize),
    /// Flat instruction offset after the conditional body.
    IfStart(usize),
    Halt,
    ReadInteger,
    PrintInteger,
    PrintCharacter,
    ArrayLiteral(Vec<i64>),
    ArrayGet,
    ArraySet,
    ArrayLength,
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
    if tokens
        .last()
        .is_none_or(|token| token.kind != TokenKind::Eof)
        || tokens[..tokens.len().saturating_sub(1)]
            .iter()
            .any(|token| token.kind == TokenKind::Eof)
    {
        return Err(ParseError {
            span: tokens.last().map_or(Span { start: 0, end: 0 }, |t| t.span),
            message: "expected exactly one final EOF token".to_owned(),
        });
    }
    let mut line = 0;
    let significant = tokens
        .iter()
        .filter_map(|token| match token.kind {
            TokenKind::Newline => {
                line += 1;
                None
            }
            TokenKind::HorizontalWhitespace => None,
            _ => Some((*token, line)),
        })
        .collect();
    let mut parser = Parser {
        tokens: significant,
        cursor: 0,
        lines: vec![Vec::new(); line + 1],
        instruction_count: 0,
    };
    parser.sequence(None, 0)?;
    Ok(Program {
        lines: parser.lines,
    })
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    cursor: usize,
    lines: Vec<Vec<Instruction>>,
    instruction_count: usize,
}

impl Parser {
    fn sequence(&mut self, closing: Option<TokenKind>, depth: usize) -> Result<(), ParseError> {
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Semicolon) {
                self.cursor += 1;
                continue;
            }
            let is_loop_end = self.phrase(&[TokenKind::Ura, TokenKind::Yada, TokenKind::Yada]);
            let is_if_end = self.phrase(&[TokenKind::Ura, TokenKind::Yaha, TokenKind::Yaha]);
            if is_loop_end || is_if_end {
                let actual = if is_loop_end {
                    TokenKind::Yada
                } else {
                    TokenKind::Yaha
                };
                if closing != Some(actual) {
                    return Err(self.error("unmatched or mismatched block end"));
                }
                self.cursor += 3;
                return Ok(());
            }
            if self.at(TokenKind::Iya) {
                if depth >= MAX_BLOCK_DEPTH {
                    return Err(self.error("block nesting exceeds 128 levels"));
                }
                let line = self.line();
                self.cursor += 1;
                let marker = self.current().kind;
                if !matches!(marker, TokenKind::Yada | TokenKind::Yaha)
                    || !self.phrase(&[marker, marker])
                {
                    return Err(self.error("expected yada yada or yaha yaha after iya"));
                }
                self.cursor += 2;
                let start = self.instruction_count;
                let slot = self.emit(
                    line,
                    if marker == TokenKind::Yada {
                        Instruction::WhileStart(0)
                    } else {
                        Instruction::IfStart(0)
                    },
                );
                self.sequence(Some(marker), depth + 1)?;
                if marker == TokenKind::Yada {
                    // The closing phrase belongs to its own physical source line.
                    let end_line = self.tokens[self.cursor - 3].1;
                    self.emit(end_line, Instruction::WhileEnd(start));
                    self.lines[line][slot] = Instruction::WhileStart(self.instruction_count);
                } else {
                    self.lines[line][slot] = Instruction::IfStart(self.instruction_count);
                }
                continue;
            }
            let line = self.line();
            let instruction = self.instruction()?;
            self.emit(line, instruction);
        }
        if closing.is_some() {
            return Err(self.error("unclosed block"));
        }
        Ok(())
    }

    fn instruction(&mut self) -> Result<Instruction, ParseError> {
        use TokenKind::*;
        let kind = self.current().kind;
        self.cursor += 1;
        Ok(match kind {
            Hayaha => Instruction::Push(self.number()?),
            Yaha => Instruction::Add,
            Hat => Instruction::Multiply,
            Ppuru => Instruction::Duplicate,
            Yat => Instruction::Halt,
            Ura => {
                let suffix = self.current().kind;
                self.cursor += 1;
                match suffix {
                    Yaha => Instruction::Subtract,
                    Hat => {
                        if self.at(Hat) {
                            self.cursor += 1;
                            Instruction::Remainder
                        } else {
                            Instruction::Divide
                        }
                    }
                    Yat => Instruction::Negate,
                    Ppuru => Instruction::Discard,
                    _ => return Err(self.error("expected yaha, hat, yat, or ppuru after ura")),
                }
            }
            Iiiiyaha => match self.current().kind {
                Ppururu => {
                    self.cursor += 1;
                    Instruction::PrintInteger
                }
                Urara => {
                    self.cursor += 1;
                    Instruction::PrintCharacter
                }
                _ => Instruction::ReadInteger,
            },
            LeftBracket => {
                let mut values = Vec::new();
                if !self.at(RightBracket) {
                    loop {
                        self.expect(Hayaha)?;
                        values.push(self.number()?);
                        if !self.at(Comma) {
                            break;
                        }
                        self.cursor += 1;
                    }
                }
                self.expect(RightBracket)?;
                Instruction::ArrayLiteral(values)
            }
            At => Instruction::ArrayGet,
            Equal => Instruction::ArraySet,
            Hash => Instruction::ArrayLength,
            _ => {
                self.cursor -= 1;
                return Err(self.error("expected an instruction"));
            }
        })
    }

    /// Hayaha has already been consumed. Whitespace does not end a number.
    fn number(&mut self) -> Result<i64, ParseError> {
        let mut value = 0_i64;
        while self.at(TokenKind::Ppuru) {
            value = value
                .checked_add(1)
                .ok_or_else(|| self.error("integer literal overflow"))?;
            self.cursor += 1;
        }
        self.expect(TokenKind::Yaha)?;
        Ok(value)
    }

    fn emit(&mut self, line: usize, instruction: Instruction) -> usize {
        let slot = self.lines[line].len();
        self.lines[line].push(instruction);
        self.instruction_count += 1;
        slot
    }
    fn phrase(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().enumerate().all(|(offset, kind)| {
            self.tokens
                .get(self.cursor + offset)
                .is_some_and(|(t, _)| t.kind == *kind)
        })
    }
    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if !self.at(kind) {
            return Err(self.error(&format!("expected {kind:?}")));
        }
        self.cursor += 1;
        Ok(())
    }
    fn current(&self) -> Token {
        self.tokens
            .get(self.cursor)
            .unwrap_or_else(|| self.tokens.last().unwrap())
            .0
    }
    fn line(&self) -> usize {
        self.tokens[self.cursor].1
    }
    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            span: self.current().span,
            message: message.to_owned(),
        }
    }
}
