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

    /// `^`
    caret,

    /// `\`. It can be got only at the end of a string.
    escape,

    /// `\<any character>`
    escape_char(char),

    /// sequence of not special characters
    literal,
}

/// A helper module containing token kinds.
pub mod tok {
    pub use super::TokenKind::*;
}

#[derive(Debug, Clone, Copy)]
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
}

/// Lexer for regular expression parsers.
pub struct Lexer<'s> {
    source: &'s str,
    iter: std::iter::Peekable<std::str::Chars<'s>>,
    span: std::ops::Range<usize>, // current token's span
    peeked: Option<Token>,
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s str) -> Self {
        // just to be sure that u32 Token::span won't overflow
        assert!(source.len() < u32::MAX as usize);
        Lexer {
            source,
            iter: source.chars().peekable(),
            span: 0..0,
            peeked: None,
        }
    }

    /// Returns and consumes the next token if exists including the peeked one.
    /// Otherwise returns `None`.
    pub fn lex(&mut self) -> Option<Token> {
        if let Some(token) = self.consume_peeked() {
            self.span = token.span();
            Some(token)
        } else if let Some(token) = self.lex_internal() {
            self.span = token.span();
            Some(token)
        } else {
            None
        }
    }

    /// Returns the next token if exists, otherwise `None`.
    ///
    /// This method doesn't update the lexer's span.
    fn lex_internal(&mut self) -> Option<Token> {
        let start = self.span.start;
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
                '^' => tok::caret,
                '?' => tok::question,
                '|' => tok::pipe,
                '(' => tok::l_paren,
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
                _ => {
                    while let Some(c) = self.iter.next_if(|c| is_not_special(*c)) {
                        end += c.len_utf8();
                    }
                    tok::literal
                }
            };
            Some(Token::new(kind, start, end))
        } else {
            None
        }
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Option<Token> {
        if let Some(token) = self.peeked {
            return Some(token);
        }
        if let Some(token) = self.lex_internal() {
            self.peeked = Some(token);
            Some(token)
        } else {
            None
        }
    }

    /// Consumes the peeked token if it exists.
    ///
    /// It moves the inner span to the peeked token.
    #[inline]
    pub fn consume_peeked(&mut self) -> Option<Token> {
        if let Some(token) = self.peeked.take() {
            self.span = token.span();
            Some(token)
        } else {
            None
        }
    }

    /// Returns the span of the last token lexed.
    #[inline]
    pub fn span(&self) -> std::ops::Range<usize> {
        self.span.clone()
    }

    /// Returns the slice of the last token lexed.
    #[inline]
    pub fn slice(&self) -> &'s str {
        &self.source[self.span.clone()]
    }
}

fn is_not_special(c: char) -> bool {
    !matches!(
        c,
        '\\' | '.' | '*' | '+' | '-' | '^' | '?' | '|' | '(' | ')' | '[' | ']' | '{' | '}'
    )
}
