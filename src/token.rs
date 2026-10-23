use crate::language::Language;

macro_rules! define_token_kinds {
    (
        $($kind:ident => [$english:literal, $korean:literal, $japanese:literal]),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum TokenKind {
            $($kind,)+
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
                match self {
                    $(
                        Self::$kind => Some(match language {
                            Language::English => $english,
                            Language::Korean => $korean,
                            Language::Japanese => $japanese,
                        }),
                    )+
                    Self::Semicolon => Some(";"),
                    Self::LeftBracket => Some("["),
                    Self::RightBracket => Some("]"),
                    Self::Comma => Some(","),
                    Self::At => Some("@"),
                    Self::Equal => Some("="),
                    Self::Hash => Some("#"),
                    Self::HorizontalWhitespace | Self::Newline | Self::Eof => None,
                }
            }
        }
    };
}

define_token_kinds! {
    Hayaha => ["hayaha", "하야하", "ハヤハ"],
    Ura => ["ura", "우라", "ウラ"],
    Yaha => ["yaha", "야하", "ヤハ"],
    Ppuru => ["ppuru", "뿌루", "プル"],
    Ppururu => ["ppururu", "뿌루루", "プルル"],
    Iiiiyaha => ["iiiiyaha", "이이이야하", "イイイヤハ"],
    Urara => ["urara", "우라라", "ウララ"],
    Iya => ["iya", "이야", "イヤ"],
    Yada => ["yada", "야다", "ヤダ"],
    Hat => ["hat", "핫", "ハッ"],
    Yat => ["yat", "얏", "ヤッ"],
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
