use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ura,
    Yaha,
    Ha,
    Puru,
    Puruya,
    Huung,
    Hueung,
    Jureuri,
    Fuun,
    Ui,
    I,
    Ua,
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
            (Self::Ha, Language::English) => Some("haa"),
            (Self::Ha, Language::Korean) => Some("하아"),
            (Self::Ha, Language::Japanese) => Some("ハァ"),
            (Self::Puru, Language::English) => Some("puru"),
            (Self::Puru, Language::Korean) => Some("푸루"),
            (Self::Puru, Language::Japanese) => Some("プル"),
            (Self::Puruya, Language::English) => Some("puruya"),
            (Self::Puruya, Language::Korean) => Some("푸루야"),
            (Self::Puruya, Language::Japanese) => Some("プルャ"),
            (Self::Huung, Language::English) => Some("huung"),
            (Self::Huung, Language::Korean) => Some("후응"),
            (Self::Huung, Language::Japanese) => Some("フゥン"),
            (Self::Hueung, Language::English) => Some("hueung"),
            (Self::Hueung, Language::Korean) => Some("후으응"),
            (Self::Hueung, Language::Japanese) => Some("フウン"),
            (Self::Jureuri, Language::English) => Some("jureuri"),
            (Self::Jureuri, Language::Korean) => Some("쥬르리"),
            (Self::Jureuri, Language::Japanese) => Some("ジュルリ"),
            (Self::Fuun, Language::English) => Some("fuun"),
            (Self::Fuun, Language::Korean) => Some("후웅"),
            (Self::Fuun, Language::Japanese) => Some("フン"),
            (Self::Ui, Language::English) => Some("ui"),
            (Self::Ui, Language::Korean) => Some("우이"),
            (Self::Ui, Language::Japanese) => Some("ウイ"),
            (Self::I, Language::English) => Some("i"),
            (Self::I, Language::Korean) => Some("이"),
            (Self::I, Language::Japanese) => Some("イ"),
            (Self::Ua, Language::English) => Some("ua"),
            (Self::Ua, Language::Korean) => Some("우아"),
            (Self::Ua, Language::Japanese) => Some("ウア"),
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
