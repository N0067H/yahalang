use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ura,
    Yaha,
    Ha,
    Puruya,
    Ruru,
    Rururu,
    Rurururu,
    Fuun,
    Bang,
    Question,
    HorizontalWhitespace,
    Newline,
    Eof,
}

impl TokenKind {
    pub const fn spelling(self, language: Language) -> Option<&'static str> {
        match (self, language) {
            (Self::Ura, Language::English) => Some("ura"),
            (Self::Ura, Language::Korean) => Some("우라"),
            (Self::Ura, Language::Japanese) => Some("ウラ"),
            (Self::Yaha, Language::English) => Some("yaha"),
            (Self::Yaha, Language::Korean) => Some("야하"),
            (Self::Yaha, Language::Japanese) => Some("ヤハ"),
            (Self::Ha, Language::English) => Some("ha"),
            (Self::Ha, Language::Korean) => Some("하아"),
            (Self::Ha, Language::Japanese) => Some("ハァ"),
            (Self::Puruya, Language::English) => Some("puruya"),
            (Self::Puruya, Language::Korean) => Some("푸루야"),
            (Self::Puruya, Language::Japanese) => Some("プルャ"),
            (Self::Ruru, Language::English) => Some("ruru"),
            (Self::Ruru, Language::Korean) => Some("루루"),
            (Self::Ruru, Language::Japanese) => Some("ルル"),
            (Self::Rururu, Language::English) => Some("rururu"),
            (Self::Rururu, Language::Korean) => Some("루루루"),
            (Self::Rururu, Language::Japanese) => Some("ルルル"),
            (Self::Rurururu, Language::English) => Some("rurururu"),
            (Self::Rurururu, Language::Korean) => Some("루루루루"),
            (Self::Rurururu, Language::Japanese) => Some("ルルルル"),
            (Self::Fuun, Language::English) => Some("fuun"),
            (Self::Fuun, Language::Korean) => Some("후웅"),
            (Self::Fuun, Language::Japanese) => Some("フゥン"),
            (Self::Bang, _) => Some("!"),
            (Self::Question, _) => Some("?"),
            (Self::HorizontalWhitespace | Self::Newline | Self::Eof, _) => None,
        }
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
