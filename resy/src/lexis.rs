use std::fmt::Write;

use crate::error::{Result, err};
use static_assertions::const_assert;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// `[`
    l_square,

    /// `[^`
    l_square_caret,

    /// `]`
    r_square,

    /// `{`
    l_brace,

    /// `}`
    r_brace,

    /// `(`
    l_paren,

    /// `(?`
    l_paren_question,

    /// `)`
    r_paren,

    /// `|`
    pipe,

    /// `*`
    star,

    /// `+`
    plus,

    /// `?`
    question,

    /// `-`
    minus,

    /// `.`
    dot,

    /// `\`. It can be got only at the end of a string.
    escape,

    /// `\<any character>`
    escape_char(char),

    /// sequence of not special characters
    char(char),

    /// end of input
    eof,
}

/// A helper module containing token kinds.
pub mod tok {
    pub use super::TokenKind::*;
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            tok::l_square => f.write_char('['),
            tok::l_square_caret => f.write_str("[^"),
            tok::r_square => f.write_char(']'),
            tok::l_brace => f.write_char('{'),
            tok::r_brace => f.write_char('}'),
            tok::l_paren => f.write_char('('),
            tok::l_paren_question => f.write_str("(?"),
            tok::r_paren => f.write_char(')'),
            tok::pipe => f.write_char('|'),
            tok::star => f.write_char('*'),
            tok::plus => f.write_char('+'),
            tok::question => f.write_char('?'),
            tok::minus => f.write_char('-'),
            tok::dot => f.write_char('.'),
            tok::escape => f.write_char('\\'),
            tok::escape_char(c) => write!(f, "\\{}", c),
            tok::char(c) => f.write_char(*c),
            tok::eof => f.write_str("EOF"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    span: (u32, u32),
}

// To be sure that `u32` is convertible to `usize`.
const_assert!(std::mem::size_of::<u32>() <= std::mem::size_of::<usize>());

impl Token {
    fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token {
            kind,
            span: (start as u32, end as u32),
        }
    }

    /// Returns the token's kind.
    #[inline]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Returns the token's span as a range of `usize`.
    pub fn span(&self) -> std::ops::Range<usize> {
        self.span.0 as usize..self.span.1 as usize
    }

    /// Returns the token's start position.
    pub fn start(&self) -> usize {
        self.span.0 as usize
    }

    /// Returns the token's end position.
    pub fn end(&self) -> usize {
        self.span.1 as usize
    }
}

/// Lexer for regular expression parsers.
pub struct Lexer<'s> {
    source: &'s str,
    iter: std::iter::Peekable<std::str::Chars<'s>>,
    pos: usize,
    peeked: Option<Token>,
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s str) -> Self {
        // just to be sure that u32 Token::span won't overflow
        assert!(source.len() < u32::MAX as usize);
        Lexer {
            source,
            iter: source.chars().peekable(),
            pos: 0,
            peeked: None,
        }
    }

    /// Returns the slice of the last token lexed.
    #[inline]
    pub fn slice(&self, span: std::ops::Range<usize>) -> &'s str {
        &self.source[span]
    }

    /// Returns the end position of the last token lexed.
    #[inline]
    pub fn end_pos(&self) -> usize {
        self.pos
    }

    /// Returns and consumes the next token if exists including the peeked one.
    /// Otherwise returns `None`.
    pub fn lex(&mut self) -> Token {
        let token = if let Some(token) = self.peeked.take() {
            token
        } else {
            self.lex_internal()
        };
        self.pos = token.span().end;
        token
    }

    pub fn expect(&mut self, expected: TokenKind) -> Result<Token> {
        let token = self.lex();
        if token.kind() != expected {
            let spell = self.slice(token.span());
            err::unexpected(spell, token.span(), format!("`{expected}`"))
        } else {
            Ok(token)
        }
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Token {
        if let Some(token) = self.peeked {
            token
        } else {
            let token = self.lex_internal();
            self.peeked = Some(token);
            token
        }
    }

    /// Consumes the peeked token if it exists.
    ///
    /// It moves the inner span to the peeked token.
    #[inline]
    pub fn consume_peeked(&mut self) {
        if let Some(token) = self.peeked.take() {
            self.pos = token.span().end;
        }
    }

    /// Returns the next token if exists, otherwise `None`.
    ///
    /// This method doesn't update the lexer's span.
    fn lex_internal(&mut self) -> Token {
        let start = self.pos;
        if let Some(c) = self.iter.next() {
            let mut end = start + c.len_utf8();
            let kind = match c {
                '\\' => {
                    if let Some(c) = self.iter.next() {
                        end += c.len_utf8();
                        tok::escape_char(c)
                    } else {
                        tok::escape
                    }
                }
                '.' => tok::dot,
                '*' => tok::star,
                '+' => tok::plus,
                '-' => tok::minus,
                '?' => tok::question,
                '|' => tok::pipe,
                '(' => {
                    if self.iter.next_if(|c| *c == '?').is_some() {
                        end += '?'.len_utf8();
                        tok::l_paren_question
                    } else {
                        tok::l_paren
                    }
                }
                ')' => tok::r_paren,
                '[' => {
                    if self.iter.next_if(|c| *c == '^').is_some() {
                        end += '^'.len_utf8();
                        tok::l_square_caret
                    } else {
                        tok::l_square
                    }
                }
                ']' => tok::r_square,
                '{' => tok::l_brace,
                '}' => tok::r_brace,
                sym => tok::char(sym),
            };
            Token::new(kind, start, end)
        } else {
            Token::new(tok::eof, start, start)
        }
    }
}
