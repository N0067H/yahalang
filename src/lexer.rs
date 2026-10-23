use crate::language::Language;
use crate::token::{Span, Token, TokenKind};

const UTTERANCES: &[TokenKind] = &[
    TokenKind::Hayaha,
    TokenKind::Ura,
    TokenKind::Yaha,
    TokenKind::Ppuru,
    TokenKind::Ppururu,
    TokenKind::Iiiiyaha,
    TokenKind::Urara,
    TokenKind::Iya,
    TokenKind::Yada,
    TokenKind::Hat,
    TokenKind::Yat,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexError {
    pub span: Span,
    pub character: char,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("haa?")
    }
}

impl std::error::Error for LexError {}

pub struct Lexer<'a> {
    source: &'a str,
    language: Language,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub const fn new(source: &'a str, language: Language) -> Self {
        Self {
            source,
            language,
            offset: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while self.offset < self.source.len() {
            tokens.push(self.next_token()?);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.source.len(),
                end: self.source.len(),
            },
        });
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        let rest = &self.source[start..];
        let first = rest.chars().next().expect("offset is before end");

        let (kind, length) = match first {
            ' ' | '\t' => (TokenKind::HorizontalWhitespace, horizontal_length(rest)),
            '\n' => (TokenKind::Newline, 1),
            '\r' if rest.starts_with("\r\n") => (TokenKind::Newline, 2),

            ';' => (TokenKind::Semicolon, 1),
            '[' => (TokenKind::LeftBracket, 1),
            ']' => (TokenKind::RightBracket, 1),
            ',' => (TokenKind::Comma, 1),
            '@' => (TokenKind::At, 1),
            '=' => (TokenKind::Equal, 1),
            '#' => (TokenKind::Hash, 1),

            _ => self.utterance(rest, first)?,
        };
        self.offset += length;

        Ok(Token {
            kind,
            span: Span {
                start,
                end: self.offset,
            },
        })
    }

    fn utterance(&self, rest: &str, first: char) -> Result<(TokenKind, usize), LexError> {
        if let Some((kind, spelling)) = Self::matching_utterance(rest, self.language) {
            return Ok((kind, spelling.len()));
        }

        Err(LexError {
            span: Span {
                start: self.offset,
                end: self.offset + first.len_utf8(),
            },
            character: first,
        })
    }

    fn matching_utterance(rest: &str, language: Language) -> Option<(TokenKind, &'static str)> {
        UTTERANCES
            .iter()
            .filter_map(|kind| kind.spelling(language).map(|spelling| (*kind, spelling)))
            .filter(|(_, spelling)| rest.starts_with(spelling))
            .max_by_key(|(_, spelling)| spelling.len())
    }
}

pub fn lex(source: &str, language: Language) -> Result<Vec<Token>, LexError> {
    Lexer::new(source, language).tokenize()
}

fn horizontal_length(source: &str) -> usize {
    source
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()
}
