use pretty_assertions::assert_eq;
use resy::str::{Lexer, tok};

#[test]
fn lexer_new() {
    _ = Lexer::new("");
    _ = Lexer::new("hello");
}

#[test]
fn lexer_lex() {
    let mut lexer = Lexer::new("h[[^^\\\\\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::char('h'));
    assert_eq!(token.span(), 0..1);
    assert_eq!(lexer.slice(token), "h");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::l_square);
    assert_eq!(token.span(), 1..2);
    assert_eq!(lexer.slice(token), "[");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::l_square_caret);
    assert_eq!(token.span(), 2..4);
    assert_eq!(lexer.slice(token), "[^");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::caret);
    assert_eq!(token.span(), 4..5);
    assert_eq!(lexer.slice(token), "^");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::escape_char('\\'));
    assert_eq!(token.span(), 5..7);
    assert_eq!(lexer.slice(token), "\\\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::escape);
    assert_eq!(token.span(), 7..8);
    assert_eq!(lexer.slice(token), "\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 8..8);
    assert_eq!(lexer.slice(token), "");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 8..8);
    assert_eq!(lexer.slice(token), "");
}

#[test]
fn lexer_peek() {
    let mut lexer = Lexer::new("ў*");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::char('ў'));
    assert_eq!(token.span(), 0..2);
    assert_eq!(lexer.slice(token), "ў");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::char('ў'));
    assert_eq!(token.span(), 0..2);
    assert_eq!(lexer.slice(token), "ў");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::char('ў'));
    assert_eq!(token.span(), 0..2);
    assert_eq!(lexer.slice(token), "ў");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::star);
    assert_eq!(token.span(), 2..3);
    assert_eq!(lexer.slice(token), "*");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 3..3);
    assert_eq!(lexer.slice(token), "");
}

#[test]
fn lexer_consume_peeked() {
    let mut lexer = Lexer::new("+?");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::plus);
    assert_eq!(token.span(), 0..1);
    assert_eq!(lexer.slice(token), "+");

    lexer.consume_peeked();

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::question);
    assert_eq!(token.span(), 1..2);
    assert_eq!(lexer.slice(token), "?");

    lexer.consume_peeked();
}

#[test]
fn lexer_expect() {
    let mut lexer = Lexer::new("a?");
    assert_eq!(
        lexer.expect(tok::char('a')).map(|t| t.kind()),
        Ok(tok::char('a'))
    );
    assert_eq!(
        lexer.expect(tok::dot).unwrap_err().to_string(),
        "expected '.', but found '?'"
    );
}

#[test]
fn lexer_lex_all_tokens() {
    let mut lexer = Lexer::new("\\a.*+-^?|()[]{}(?[^\\");
    loop {
        let token = lexer.lex();
        if token.kind() == tok::eof {
            break;
        }
    }
}

#[test]
fn token_display_fmt() {
    let mut lexer = Lexer::new("[[^]{}((?)|*+?-.^\\aa\\");
    let mut next = || format!("{}", lexer.lex().kind());
    assert_eq!(next(), "[");
    assert_eq!(next(), "[^");
    assert_eq!(next(), "]");
    assert_eq!(next(), "{");
    assert_eq!(next(), "}");
    assert_eq!(next(), "(");
    assert_eq!(next(), "(?");
    assert_eq!(next(), ")");
    assert_eq!(next(), "|");
    assert_eq!(next(), "*");
    assert_eq!(next(), "+");
    assert_eq!(next(), "?");
    assert_eq!(next(), "-");
    assert_eq!(next(), ".");
    assert_eq!(next(), "^");
    assert_eq!(next(), "\\a");
    assert_eq!(next(), "a");
    assert_eq!(next(), "\\");
    assert_eq!(next(), "EOF");
    assert_eq!(next(), "EOF");
}
