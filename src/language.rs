#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    #[default]
    English,
    Korean,
    Japanese,
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Korean => "ko",
            Self::Japanese => "ja",
        }
    }
}
