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
    fn parse_escape(&mut self, iter: &mut Chars<'s>, prev_node: Node<'n>) -> Result<Node<'n>> {
        if let Some(c) = iter.next() {
            let real_char = match c {
                '"' | '\\' => Some(c),
                'n' => Some('\n'),
                'r' => Some('\r'),
                't' => Some('\t'),
                '0' => Some('\0'),
                'x' => self.parse_7bit_codepoint(iter)?,
                // 'u' => self.parse_unicode_codepoint(iter)?,
                _ => None,
            };
            if let Some(c) = real_char {
                let new_node = self.nfa.node();
                let mut buf = [0u8; 16];
                self.codec.encode_char(c, &mut buf)?;

                prev_node.connect(new_node, c as u8);
                Ok(new_node)
            } else {
                Err(UnsupportedEscape(c))
            }
        } else {
            Err(UnexpectedEof {
                aborted_expr: "escape".into(),
            })
        }
    }

    fn parse_7bit_codepoint(&mut self, iter: &mut Chars<'s>) -> Result<Option<char>> {
        let Some(first_hex) = iter.next() else {
            return Err(UnexpectedEof {
                aborted_expr: "7-bit codepoint escape".into(),
            });
        };
        let Some(second_hex) = iter.next() else {
            return Err(UnexpectedEof {
                aborted_expr: "7-bit codepoint escape".into(),
            });
        };
        if !first_hex.is_ascii_hexdigit() || !second_hex.is_ascii_hexdigit() {
            let mut hex_str = first_hex.to_string();
            hex_str.push(second_hex);
            return Err(InvalidHex(hex_str));
        }
        if first_hex > '7' {
            return Err(OutOfRange {
                value: first_hex.into(),
                range: "[0x00..=0x7F]".into(),
            });
        }
        const UPPERCASE_MASK: u32 = !0b0010_0000;
        let mut codepoint = (first_hex as u32 - '0' as u32) << 4;
        codepoint |= ((second_hex as u32 - 'A' as u32) & UPPERCASE_MASK) + 10;

        debug_assert!(char::from_u32(codepoint).is_some());
        Ok(Some(unsafe { char::from_u32_unchecked(codepoint) }))
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
}
