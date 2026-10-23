use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Hayaha,
    Ura,
    Yaha,
    Ppuru,
    Ppururu,
    Iiiiyaha,
    Urara,
    Iya,
    Yada,
    Hat,
    Yat,

    Semicolon,
    LeftBracket,
    RightBracket,
    Comma,
    At,
    Equal,
    Hash,

    HorizontalWhitespace,
    Newline,
    Eof,
}

impl TokenKind {
    pub const fn spelling(self, language: Language) -> Option<&'static str> {
        let spellings = match self {
            Self::Hayaha => ["hayaha", "하야하", "ハヤハ"],
            Self::Ura => ["ura", "우라", "ウラ"],
            Self::Yaha => ["yaha", "야하", "ヤハ"],
            Self::Ppuru => ["ppuru", "뿌루", "プル"],
            Self::Ppururu => ["ppururu", "뿌루루", "プルル"],
            Self::Iiiiyaha => ["iiiiyaha", "이이이야하", "イイイヤハ"],
            Self::Urara => ["urara", "우라라", "ウララ"],
            Self::Iya => ["iya", "이야", "イヤ"],
            Self::Yada => ["yada", "야다", "ヤダ"],
            Self::Hat => ["hat", "핫", "ハッ"],
            Self::Yat => ["yat", "얏", "ヤッ"],

            Self::Semicolon => return Some(";"),
            Self::LeftBracket => return Some("["),
            Self::RightBracket => return Some("]"),
            Self::Comma => return Some(","),
            Self::At => return Some("@"),
            Self::Equal => return Some("="),
            Self::Hash => return Some("#"),
            Self::HorizontalWhitespace | Self::Newline | Self::Eof => return None,
        };

        let index = match language {
            Language::English => 0,
            Language::Korean => 1,
            Language::Japanese => 2,
        };

        Some(spellings[index])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
