use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::token::{Span, TokenKind};

const SOUNDS: [TokenKind; 12] = [
    TokenKind::Hayaha,
    TokenKind::Haa,
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

#[test]
fn adjacent_sounds_keep_longest_matches_and_byte_spans_in_every_language() {
    for language in [Language::English, Language::Korean, Language::Japanese] {
        check_adjacent_sounds(language);
    }
}

fn check_adjacent_sounds(language: Language) {
    for (first, second) in SOUNDS.into_iter().flat_map(|a| SOUNDS.map(|b| (a, b))) {
        let left = first.spelling(language).unwrap();
        let right = second.spelling(language).unwrap();
        let source = format!("{left}{right}");
        let tokens = lex(&source, language).unwrap();
        assert_eq!(tokens.len(), 3, "{source}");
        assert_eq!(tokens[0].kind, first, "{source}");
        assert_eq!(tokens[1].kind, second, "{source}");
        assert_eq!(tokens[2].kind, TokenKind::Eof);
        assert_eq!(
            tokens[0].span,
            Span {
                start: 0,
                end: left.len()
            }
        );
        assert_eq!(
            tokens[1].span,
            Span {
                start: left.len(),
                end: source.len()
            }
        );
    }
}

#[test]
fn sounds_from_another_language_are_rejected_at_the_original_offset() {
    for language in [Language::English, Language::Korean, Language::Japanese] {
        check_foreign_sounds(language);
    }
}

fn check_foreign_sounds(language: Language) {
    for other in [Language::English, Language::Korean, Language::Japanese] {
        if other == language {
            continue;
        }
        let prefix = TokenKind::Ppuru.spelling(language).unwrap();
        let foreign = TokenKind::Ppururu.spelling(other).unwrap();
        let error = lex(&format!("{prefix}{foreign}"), language).unwrap_err();
        assert_eq!(error.span.start, prefix.len());
        assert_eq!(error.character, foreign.chars().next().unwrap());
    }
}
