use crate::error::{Result, err};
use regr::{Graph, Node};
use std::str::Chars;

pub struct Parser<'a> {
    nfa: &'a Graph,
}

impl<'a> Parser<'a> {
    pub fn new(nfa: &'a Graph) -> Self {
        assert!(nfa.is_nfa(), "`repy::Parser` can build only NFA graph");
        Self { nfa }
    }

    pub fn parse(&mut self, pattern: &str, prev_node: Node<'a>) -> Result<Node<'a>> {
        let mut lexer = pattern.chars();
        let mut last_node = prev_node;
        while let Some(c) = lexer.next() {
            last_node = match c {
                '\\' => self.parse_escape(&mut lexer, prev_node)?,
                _ => todo!(),
            };
        }
        Ok(last_node)
    }

    fn parse_escape(&mut self, lexer: &mut Chars, prev_node: Node<'a>) -> Result<Node<'a>> {
        if let Some(c) = lexer.next() {
            match c {
                '\\' => {
                    let new_node = self.nfa.node();
                    prev_node.connect(new_node, b'\\');
                    Ok(new_node)
                }
                _ => err::unsupported_escape(c),
            }
        } else {
            err::unexpected_eof("the escape expression completion is expected")
        }
    }
}
