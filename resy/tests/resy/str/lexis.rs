use pretty_assertions::assert_eq;
use resy::str::{Lexer, tok};

#[test]
fn lexer_new() {
    _ = Lexer::new("");
    _ = Lexer::new("hello");
}

#[test]
fn lexer_lex() {
    let mut lexer = Lexer::new("hello[[^^\\\\\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::literal);
    assert_eq!(token.span(), 0..5);
    assert_eq!(lexer.slice(token), "hello");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::l_square);
    assert_eq!(token.span(), 5..6);
    assert_eq!(lexer.slice(token), "[");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::l_square_caret);
    assert_eq!(token.span(), 6..8);
    assert_eq!(lexer.slice(token), "[^");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::caret);
    assert_eq!(token.span(), 8..9);
    assert_eq!(lexer.slice(token), "^");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::escape_char('\\'));
    assert_eq!(token.span(), 9..11);
    assert_eq!(lexer.slice(token), "\\\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::escape);
    assert_eq!(token.span(), 11..12);
    assert_eq!(lexer.slice(token), "\\");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 12..12);
    assert_eq!(lexer.slice(token), "");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 12..12);
    assert_eq!(lexer.slice(token), "");
}

#[test]
fn lexer_peek() {
    let mut lexer = Lexer::new("мяў*");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::literal);
    assert_eq!(token.span(), 0..6);
    assert_eq!(lexer.slice(token), "мяў");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::literal);
    assert_eq!(token.span(), 0..6);
    assert_eq!(lexer.slice(token), "мяў");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::literal);
    assert_eq!(token.span(), 0..6);
    assert_eq!(lexer.slice(token), "мяў");

    let token = lexer.lex();
    assert_eq!(token.kind(), tok::star);
    assert_eq!(token.span(), 6..7);
    assert_eq!(lexer.slice(token), "*");

    let token = lexer.peek();
    assert_eq!(token.kind(), tok::eof);
    assert_eq!(token.span(), 7..7);
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
fn lexer_lex_all_tokens() {
    let mut lexer = Lexer::new("\\a.*+-^?|()[]{}\\");
    loop {
        let token = lexer.lex();
        if token.kind() == tok::eof {
            break;
        }
    }
}
