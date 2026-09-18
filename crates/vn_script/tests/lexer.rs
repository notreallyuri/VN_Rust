use vn_script::{TokenKind, tokenize};

const ALL_FEATURES: &str = include_str!("fixtures/all_features.story");

#[test]
fn fixture_covers_every_token_kind() {
    let tokens = tokenize(ALL_FEATURES);

    let expected = [
        TokenKind::Scene,
        TokenKind::Show,
        TokenKind::Remove,
        TokenKind::Clear,
        TokenKind::Dialogue,
        TokenKind::Narration,
        TokenKind::ChoiceBlock,
        TokenKind::ChoiceOption,
        TokenKind::Jump,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::Call,
        TokenKind::Set,
        TokenKind::Add,
        TokenKind::Commit,
    ];

    for kind in expected {
        assert!(
            tokens.iter().any(|t| t.kind == kind),
            "fixture has no {:?} token",
            kind
        );
    }
}

#[test]
fn skips_comments_and_blank_lines() {
    let tokens = tokenize(ALL_FEATURES);

    assert!(tokens.iter().all(|t| !t.payload.starts_with('#')));
    assert_eq!(tokens[0].kind, TokenKind::Scene);
    assert_eq!(tokens[0].line, 4);
}
