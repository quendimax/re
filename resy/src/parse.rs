use crate::error::{Error, Result};
use regr::{Graph, Node};
use std::str::Chars;

pub struct Parser<'a> {
    nfa: &'a Graph,
    last_node: Node<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(nfa: &'a Graph, first_node: Node<'a>) -> Self {
        Self {
            nfa,
            last_node: first_node,
        }
    }

    pub fn parse(&mut self, pattern: &str) -> Result<()> {
        let mut lexer = pattern.chars();
        while let Some(c) = lexer.next() {
            match c {
                '\\' => self.parse_escape(&mut lexer)?,
                _ => todo!(),
            };
        }
        Ok(())
    }

    fn parse_escape(&mut self, lexer: &mut Chars) -> Result<()> {
        if let Some(c) = lexer.next() {
            return match c {
                '\\' => {
                    let node = self.nfa.node();
                    self.last_node.connect(node, b'\\');
                    Ok(())
                }
                _ => Err(Error::InvalidEscape(c)),
            };
        }
        Err(Error::UnexpectedEof)
    }
}
