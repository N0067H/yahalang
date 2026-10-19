use crate::language::Language;

macro_rules! define_token_kinds {
    (
        $($kind:ident => [$english:literal, $korean:literal, $japanese:literal]),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum TokenKind {
            $($kind,)+
            Bang,
            Question,
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
                    Self::Bang => Some("!"),
                    Self::Question => Some("?"),
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
    Ura => ["ura", "우라", "ウラ"],
    Yaha => ["yaha", "야하", "ヤハ"],
    Ha => ["haa", "하아", "ハァ"],
    Puru => ["puru", "푸루", "プル"],
    Puruya => ["puruya", "푸루야", "プルャ"],
    Huung => ["huung", "후응", "フゥン"],
    Hueung => ["hueung", "후으응", "フウン"],
    Jureuri => ["jureuri", "쥬르리", "ジュルリ"],
    Fuun => ["fuun", "후웅", "フン"],
    Ui => ["ui", "우이", "ウイ"],
    I => ["i", "이", "イ"],
    Ua => ["ua", "우아", "ウア"],
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
