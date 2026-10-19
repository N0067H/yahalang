use crate::language::Language;
use crate::token::{Span, Token, TokenKind};

const UTTERANCES: &[TokenKind] = &[
    TokenKind::Puruya,
    TokenKind::Jureuri,
    TokenKind::Hueung,
    TokenKind::Huung,
    TokenKind::Fuun,
    TokenKind::Puru,
    TokenKind::Ura,
    TokenKind::Yaha,
    TokenKind::Ha,
    TokenKind::Ui,
    TokenKind::Ua,
    TokenKind::I,
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

        let kind = match first {
            ' ' | '\t' => {
                self.offset += rest
                    .bytes()
                    .take_while(|byte| matches!(byte, b' ' | b'\t'))
                    .count();
                TokenKind::HorizontalWhitespace
            }
            '\n' => {
                self.offset += 1;
                TokenKind::Newline
            }
            '\r' if rest.starts_with("\r\n") => {
                self.offset += 2;
                TokenKind::Newline
            }
            '!' => {
                self.offset += 1;
                TokenKind::Bang
            }
            '?' => {
                self.offset += 1;
                TokenKind::Question
            }
            _ => {
                if let Some((kind, spelling)) = Self::matching_utterance(rest, self.language) {
                    self.offset += spelling.len();
                    kind
                } else {
                    return Err(LexError {
                        span: Span {
                            start,
                            end: start + first.len_utf8(),
                        },
                        character: first,
                    });
                }
            }
        };

        Ok(Token {
            kind,
            span: Span {
                start,
                end: self.offset,
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str, language: Language) -> Vec<TokenKind> {
        lex(source, language)
            .unwrap()
            .into_iter()
            .filter(|token| token.kind != TokenKind::HorizontalWhitespace)
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn lexes_core_spellings_in_each_language() {
        let expected = vec![
            TokenKind::Ura,
            TokenKind::Yaha,
            TokenKind::Ha,
            TokenKind::Puru,
            TokenKind::Puruya,
            TokenKind::Huung,
            TokenKind::Hueung,
            TokenKind::Jureuri,
            TokenKind::Fuun,
            TokenKind::Ui,
            TokenKind::I,
            TokenKind::Ua,
            TokenKind::Eof,
        ];
        assert_eq!(
            kinds(
                "ura yaha haa puru puruya huung hueung jureuri fuun ui i ua",
                Language::English
            ),
            expected
        );
        assert_eq!(
            kinds(
                "우라 야하 하아 푸루 푸루야 후응 후으응 쥬르리 후웅 우이 이 우아",
                Language::Korean
            ),
            expected
        );
        assert_eq!(
            kinds(
                "ウラ ヤハ ハァ プル プルャ フゥン フウン ジュルリ フン ウイ イ ウア",
                Language::Japanese
            ),
            expected
        );
    }

    #[test]
    fn preserves_spacing_punctuation_and_utf8_byte_spans() {
        let tokens = lex("우라\t야하!\r\n", Language::Korean).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ura);
        assert_eq!(tokens[0].span, Span { start: 0, end: 6 });
        assert_eq!(tokens[1].kind, TokenKind::HorizontalWhitespace);
        assert_eq!(tokens[2].kind, TokenKind::Yaha);
        assert_eq!(tokens[3].kind, TokenKind::Bang);
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[4].span, Span { start: 14, end: 16 });
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn chooses_the_longest_matching_utterance() {
        assert_eq!(
            kinds("puruya puru", Language::English),
            vec![TokenKind::Puruya, TokenKind::Puru, TokenKind::Eof]
        );
    }

    #[test]
    fn rejects_unknown_or_wrong_language_spelling() {
        let err = lex("우라", Language::English).unwrap_err();
        assert_eq!(err.character, '우');
        assert_eq!(err.span, Span { start: 0, end: 3 });
        assert!(lex("ura. yaha", Language::English).is_err());
        assert!(lex("ura\r", Language::English).is_err());
    }
}
