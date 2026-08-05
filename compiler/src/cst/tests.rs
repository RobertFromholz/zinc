use super::*;

#[test]
fn test_combine_right_arrow() {
    let parts = vec![
        GreenToken { kind: TokenKind::Minus, length: 1 },
        GreenToken { kind: TokenKind::GreaterThan, length: 1 },
    ];
    let result = TokenKind::RightArrow.combine(&parts);
    assert_eq!(result, Some(GreenToken { kind: TokenKind::RightArrow, length: 2 }));
}

#[test]
fn test_combine_path_separator() {
    let parts = vec![
        GreenToken { kind: TokenKind::Colon, length: 1 },
        GreenToken { kind: TokenKind::Colon, length: 1 },
    ];
    let result = TokenKind::PathSeparator.combine(&parts);
    assert_eq!(result, Some(GreenToken { kind: TokenKind::PathSeparator, length: 2 }));
}

#[test]
fn test_combine_wrong_parts() {
    let parts = vec![
        GreenToken { kind: TokenKind::Minus, length: 1 },
        GreenToken { kind: TokenKind::Semicolon, length: 1 }
    ];
    let result = TokenKind::RightArrow.combine(&parts);
    assert_eq!(result, None);
}

#[test]
fn test_combine_non_consecutive() {
    let parts = vec![
        GreenToken { kind: TokenKind::Minus, length: 1 },
        GreenToken { kind: TokenKind::Whitespace, length: 1 },
        GreenToken { kind: TokenKind::GreaterThan, length: 1 }
    ];
    let result = TokenKind::RightArrow.combine(&parts);
    assert_eq!(result, None);
}

#[test]
fn test_combine_single_token() {
    let parts = vec![
        GreenToken { kind: TokenKind::Comma, length: 1 },
    ];
    let result = TokenKind::Comma.combine(&parts);
    assert_eq!(result, Some(GreenToken { kind: TokenKind::Comma, length: 1 }));
}

#[test]
fn test_combine_insufficient_parts() {
    let parts = vec![
        GreenToken { kind: TokenKind::Minus, length: 1 },
    ];
    let result = TokenKind::RightArrow.combine(&parts);
    assert_eq!(result, None);
}