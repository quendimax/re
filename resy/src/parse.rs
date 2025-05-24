use crate::codec::Codec;
use crate::error::{Error::*, Result};
use regr::{Graph, Node};
use std::str::Chars;

pub struct Parser<'n, T: Codec> {
    nfa: &'n Graph,
    codec: T,
}

impl<'n, 's, T: Codec> Parser<'n, T> {
    pub fn new(nfa: &'n Graph, codec: T) -> Self {
        assert!(nfa.is_nfa(), "`repy::Parser` can build only an NFA graph");
        Self { nfa, codec }
    }

    pub fn parse(&mut self, pattern: &'s str, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        let mut iter = pattern.chars();
        while let Some(c) = iter.next() {
            prev_node = match c {
                '\\' => self.parse_escape(&mut iter, prev_node)?,
                symbol => self.parse_char(symbol, prev_node)?,
            };
        }
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
    /// - `\\` - backslash,
    fn parse_escape(&mut self, iter: &mut Chars<'s>, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        if let Some(c) = iter.next() {
            let codepoint: Option<u32> = match c {
                '"' | '\\' => Some(c as u32),
                'n' => Some('\n' as u32),
                'r' => Some('\r' as u32),
                't' => Some('\t' as u32),
                '0' => Some('\0' as u32),
                'x' => Some(self.parse_ascii_escape(iter)?),
                'u' => Some(self.parse_unicode_escape(iter)?),
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

    fn parse_ascii_escape(&mut self, iter: &mut Chars<'s>) -> Result<u32> {
        let Some(first_hex) = iter.next() else {
            return Err(UnexpectedEof {
                aborted_expr: "ascii escape".into(),
            });
        };
        let Some(second_hex) = iter.next() else {
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

    fn parse_unicode_escape(&mut self, iter: &mut Chars<'s>) -> Result<u32> {
        self.expect('{', iter)?;
        let mut codepoint = 0u32;
        for i in 0..6 {
            let Some(c) = iter.next() else {
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
        self.expect('}', iter)?;
        Ok(codepoint)
    }

    fn parse_char(&mut self, symbol: char, mut prev_node: Node<'n>) -> Result<Node<'n>> {
        let mut buffer = [0u8; 4];
        let len = self.codec.encode_char(symbol, &mut buffer)?;
        for byte in buffer[..len].iter() {
            let new_node = self.nfa.node();
            prev_node.connect(new_node, *byte);
            prev_node = new_node;
        }
        Ok(prev_node)
    }

    fn expect(&mut self, symbol: char, iter: &mut Chars<'s>) -> Result<()> {
        if let Some(c) = iter.next() {
            if c == symbol {
                Ok(())
            } else {
                Err(UnexpectedToken {
                    unexpected: c.into(),
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
