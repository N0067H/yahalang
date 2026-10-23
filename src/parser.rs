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
    let mut significant = Vec::new();

    for token in tokens {
        match token.kind {
            TokenKind::Newline => line += 1,
            TokenKind::HorizontalWhitespace => {}
            _ => significant.push((*token, line)),
        }
    }

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
        let mut closed = false;

        while !self.at(TokenKind::Eof) && !closed {
            closed = self.sequence_step(closing, depth)?;
        }

        if closing.is_some() && !closed {
            return Err(self.error("unclosed block"));
        }

        Ok(())
    }

    fn sequence_step(
        &mut self,
        closing: Option<TokenKind>,
        depth: usize,
    ) -> Result<bool, ParseError> {
        if self.take(TokenKind::Semicolon) {
            return Ok(false);
        }

        if let Some(marker) = self.block_end() {
            return self.close_block(closing, marker);
        }

        if self.at(TokenKind::Iya) {
            self.block(depth)?;
            return Ok(false);
        }

        let line = self.line();
        let instruction = self.instruction()?;
        self.emit(line, instruction);

        Ok(false)
    }

    fn block_end(&self) -> Option<TokenKind> {
        if self.phrase(&[TokenKind::Ura, TokenKind::Yada, TokenKind::Yada]) {
            return Some(TokenKind::Yada);
        }

        if self.phrase(&[TokenKind::Ura, TokenKind::Yaha, TokenKind::Yaha]) {
            return Some(TokenKind::Yaha);
        }

        None
    }

    fn close_block(
        &mut self,
        expected: Option<TokenKind>,
        actual: TokenKind,
    ) -> Result<bool, ParseError> {
        if expected != Some(actual) {
            return Err(self.error("unmatched or mismatched block end"));
        }

        self.cursor += 3;
        Ok(true)
    }

    fn block(&mut self, depth: usize) -> Result<(), ParseError> {
        if depth >= MAX_BLOCK_DEPTH {
            return Err(self.error("block nesting exceeds 128 levels"));
        }

        let line = self.line();
        self.cursor += 1;
        let marker = self.block_marker()?;
        let start = self.instruction_count;
        let instruction = match marker {
            TokenKind::Yada => Instruction::WhileStart(0),
            _ => Instruction::IfStart(0),
        };
        let slot = self.emit(line, instruction);

        self.sequence(Some(marker), depth + 1)?;
        self.finish_block(marker, line, slot, start);

        Ok(())
    }

    fn block_marker(&mut self) -> Result<TokenKind, ParseError> {
        let marker = self.current().kind;

        if !matches!(marker, TokenKind::Yada | TokenKind::Yaha) || !self.phrase(&[marker, marker]) {
            return Err(self.error("expected yada yada or yaha yaha after iya"));
        }

        self.cursor += 2;
        Ok(marker)
    }

    fn finish_block(&mut self, marker: TokenKind, line: usize, slot: usize, start: usize) {
        if marker == TokenKind::Yaha {
            self.lines[line][slot] = Instruction::IfStart(self.instruction_count);
            return;
        }

        let end_line = self.tokens[self.cursor - 3].1;
        self.emit(end_line, Instruction::WhileEnd(start));
        self.lines[line][slot] = Instruction::WhileStart(self.instruction_count);
    }

    fn instruction(&mut self) -> Result<Instruction, ParseError> {
        use TokenKind::*;

        let kind = self.current().kind;
        self.cursor += 1;

        match kind {
            Hayaha => self.number().map(Instruction::Push),
            Yaha => Ok(Instruction::Add),
            Hat => Ok(Instruction::Multiply),
            Ppuru => Ok(Instruction::Duplicate),
            Yat => Ok(Instruction::Halt),

            Ura => self.prefixed_instruction(),
            Iiiiyaha => Ok(self.io_instruction()),

            LeftBracket => self.array_literal(),
            At => Ok(Instruction::ArrayGet),
            Equal => Ok(Instruction::ArraySet),
            Hash => Ok(Instruction::ArrayLength),

            _ => self.invalid_instruction(),
        }
    }

    fn prefixed_instruction(&mut self) -> Result<Instruction, ParseError> {
        use TokenKind::*;

        let suffix = self.current().kind;
        self.cursor += 1;

        match suffix {
            Yaha => Ok(Instruction::Subtract),
            Hat if self.take(Hat) => Ok(Instruction::Remainder),
            Hat => Ok(Instruction::Divide),
            Yat => Ok(Instruction::Negate),
            Ppuru => Ok(Instruction::Discard),
            _ => Err(self.error("expected yaha, hat, yat, or ppuru after ura")),
        }
    }

    fn io_instruction(&mut self) -> Instruction {
        if self.take(TokenKind::Ppururu) {
            return Instruction::PrintInteger;
        }

        if self.take(TokenKind::Urara) {
            return Instruction::PrintCharacter;
        }

        Instruction::ReadInteger
    }

    fn array_literal(&mut self) -> Result<Instruction, ParseError> {
        let mut values = Vec::new();
        let mut more = !self.at(TokenKind::RightBracket);

        while more {
            self.expect(TokenKind::Hayaha)?;
            values.push(self.number()?);
            more = self.take(TokenKind::Comma);
        }

        self.expect(TokenKind::RightBracket)?;
        Ok(Instruction::ArrayLiteral(values))
    }

    fn invalid_instruction(&mut self) -> Result<Instruction, ParseError> {
        self.cursor -= 1;
        Err(self.error("expected an instruction"))
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

    fn take(&mut self, kind: TokenKind) -> bool {
        if !self.at(kind) {
            return false;
        }

        self.cursor += 1;
        true
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
