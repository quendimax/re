use crate::codec::Codec;
use crate::error::{Error::*, Result};
use regr::{Graph, Node};
use std::str::Chars;

pub struct Parser<'n, 's, T: Codec> {
    nfa: &'n Graph,
    lexer: Lexer<'s>,
    codec: T,
}

impl<'n, 's, T: Codec> Parser<'n, 's, T> {
    pub fn new(nfa: &'n Graph, codec: T) -> Self {
        assert!(nfa.is_nfa(), "`repy::Parser` can build only an NFA graph");
        let lexer = Lexer::empty();
        Self { nfa, lexer, codec }
    }

    pub fn parse(&mut self, pattern: &'s str, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        assert!(prev_node.gid() == self.nfa.gid());
        self.lexer = Lexer::new(pattern);
        loop {
            prev_node = self.parse_expr(prev_node)?;
            if let Some(symbol) = self.lexer.peek() {
                prev_node = match symbol {
                    ')' => return Err(UnexpectedCloseParen),
                    _ => self.parse_expr(prev_node)?,
                };
            } else {
                break;
            }
        }
        self.lexer = Lexer::empty();
        Ok(prev_node)
    }

    fn parse_expr(&mut self, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        while let Some(c) = self.lexer.peek() {
            prev_node = match c {
                '(' => self.parse_parens(prev_node)?,
                ')' => break,

                '\\' => self.parse_escape(prev_node)?,

                _ => self.parse_char(prev_node)?,
            };
        }
        Ok(prev_node)
    }

    fn parse_parens(&mut self, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('(')?;
        prev_node = self.parse_expr(prev_node)?;
        self.lexer.expect(')')?;
        Ok(prev_node)
    }

    /// Parse an escape sequence.
    ///
    /// It is intended to be compatible with the Rust string literal escape
    /// sequences.
    ///
    /// Escape sequences compatible with Rust string literals:
    /// - `\"` - double quote,
    /// - `\x<two hexes between 00 and 7F>` - 7-bit code point escape,
    /// - `\u{<up to six hexes>}` - 24-bit code point escape,
    /// - `\n` - line feed escape,
    /// - `\r` - carriage return escape,
    /// - `\t` - horizontal tab escape,
    /// - `\0` - null escape,
    /// - `\\` - backslash.
    ///
    /// Escape sequences extending Rust string literals:
    /// - `\(` - left parenthesis,
    /// - `\)` - right parenthesis,
    fn parse_escape(&mut self, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('\\')?;
        if let Some(c) = self.lexer.lex() {
            let codepoint: Option<u32> = match c {
                '"' | '\\' | '(' | ')' => Some(c as u32),
                'n' => Some('\n' as u32),
                'r' => Some('\r' as u32),
                't' => Some('\t' as u32),
                '0' => Some('\0' as u32),
                'x' => Some(self.parse_ascii_escape()?),
                'u' => Some(self.parse_unicode_escape()?),
                _ => None,
            };
            if let Some(c) = codepoint {
                let mut buf = [0u8; 16];
                let len = self.codec.encode_ucp(c, &mut buf)?;

                for byte in &buf[0..len] {
                    let new_node = self.nfa.node();
                    prev_node.connect(new_node, *byte);
                    prev_node = new_node;
                }
                Ok(prev_node)
            } else {
                Err(UnsupportedEscape(c))
            }
        } else {
            Err(UnexpectedEof {
                aborted_expr: "escape".into(),
            })
        }
    }

    fn parse_ascii_escape(&mut self) -> Result<u32> {
        let Some(first_hex) = self.lexer.lex() else {
            return Err(UnexpectedEof {
                aborted_expr: "ascii escape".into(),
            });
        };
        let Some(second_hex) = self.lexer.lex() else {
            return Err(UnexpectedEof {
                aborted_expr: "ascii escape".into(),
            });
        };
        if !first_hex.is_ascii_hexdigit() || !second_hex.is_ascii_hexdigit() {
            let mut hex_str = first_hex.to_string();
            hex_str.push(second_hex);
            return Err(InvalidHex(hex_str));
        }
        if first_hex > '7' {
            let mut hex_str = first_hex.to_string();
            hex_str.push(second_hex);
            return Err(OutOfRange {
                value: hex_str,
                range: "[0x00..=0x7F]".into(),
            });
        }
        let mut codepoint = (first_hex as u32 - '0' as u32) << 4;
        if second_hex > '9' {
            const UPPERCASE_MASK: u32 = !0b0010_0000;
            codepoint |= ((second_hex as u32 - 'A' as u32) & UPPERCASE_MASK) + 10;
        } else {
            codepoint |= (second_hex as u32).wrapping_sub('0' as u32);
        }

        // for 7 bit codepoint must always be a correct unicode codepoint
        debug_assert!(char::from_u32(codepoint).is_some());
        Ok(codepoint)
    }

    fn parse_unicode_escape(&mut self) -> Result<u32> {
        self.lexer.expect('{')?;
        let mut codepoint = 0u32;
        for i in 0..6 {
            let Some(c) = self.lexer.lex() else {
                return Err(UnexpectedEof {
                    aborted_expr: "unicode escape".into(),
                });
            };
            if c == '}' {
                if i == 0 {
                    return Err(EmptyEscape);
                } else {
                    return Ok(codepoint);
                }
            }
            if !c.is_ascii_hexdigit() {
                return Err(InvalidHex(c.into()));
            }
            codepoint <<= 4;
            if c > '9' {
                const UPPERCASE_MASK: u32 = !0b0010_0000;
                codepoint |= ((c as u32 - 'A' as u32) & UPPERCASE_MASK) + 10;
            } else {
                codepoint |= (c as u32).wrapping_sub('0' as u32);
            }
        }
        self.lexer.expect('}')?;
        Ok(codepoint)
    }

    fn parse_char(&mut self, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        let symbol = self.lexer.lex().unwrap();
        let mut buffer = [0u8; 4];
        let len = self.codec.encode_char(symbol, &mut buffer)?;
        for byte in buffer[..len].iter() {
            let new_node = self.nfa.node();
            prev_node.connect(new_node, *byte);
            prev_node = new_node;
        }
        Ok(prev_node)
    }
}

struct Lexer<'s> {
    iter: Option<Chars<'s>>,
    peeked: Option<char>,
}

impl<'s> Lexer<'s> {
    fn new(source: &'s str) -> Self {
        Self {
            iter: Some(source.chars()),
            peeked: None,
        }
    }

    fn empty() -> Self {
        Self {
            iter: None,
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        if let Some(c) = self.peeked {
            return Some(c);
        }
        if let Some(iter) = self.iter.as_mut() {
            if let Some(c) = iter.next() {
                self.peeked = Some(c);
                return Some(c);
            }
        }
        None
    }

    fn lex(&mut self) -> Option<char> {
        if let Some(c) = self.peeked.take() {
            return Some(c);
        }
        if let Some(iter) = self.iter.as_mut() {
            return iter.next();
        }
        None
    }

    fn expect(&mut self, symbol: char) -> Result<()> {
        if let Some(c) = self.lex() {
            if c == symbol {
                Ok(())
            } else {
                Err(UnexpectedToken {
                    gotten: c.into(),
                    expected: symbol.into(),
                })
            }
        } else {
            Err(UnexpectedEof {
                aborted_expr: "regular".into(),
            })
        }
    }
}
