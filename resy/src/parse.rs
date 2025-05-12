use crate::codec::Codec;
use crate::error::{Error::*, Result};
use regr::{Graph, Node};
use std::str::Chars;

pub struct Parser<'a, T: Codec> {
    nfa: &'a Graph,
    codec: T,
}

impl<'a, T: Codec> Parser<'a, T> {
    pub fn new(nfa: &'a Graph, codec: T) -> Self {
        assert!(nfa.is_nfa(), "`repy::Parser` can build only an NFA graph");
        Self { nfa, codec }
    }

    pub fn parse(&mut self, pattern: &str, mut prev_node: Node<'a>) -> Result<Node<'a>> {
        let mut lexer = pattern.chars();
        while let Some(c) = lexer.next() {
            prev_node = match c {
                '\\' => self.parse_escape(&mut lexer, prev_node)?,
                symbol => self.parse_char(symbol, prev_node)?,
            };
        }
        Ok(prev_node)
    }

    fn parse_escape(&mut self, lexer: &mut Chars, prev_node: Node<'a>) -> Result<Node<'a>> {
        if let Some(c) = lexer.next() {
            match c {
                '\\' => {
                    let new_node = self.nfa.node();
                    prev_node.connect(new_node, b'\\');
                    Ok(new_node)
                }
                _ => Err(UnsupportedEscape(c)),
            }
        } else {
            Err(UnexpectedEof(
                "the escape expression completion is expected",
            ))
        }
    }

    fn parse_char(&mut self, symbol: char, mut prev_node: Node<'a>) -> Result<Node<'a>> {
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
