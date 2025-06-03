//! There is an implementation of regular expression parser.
//!
//! Grammar used here is described in `resy/docs/unic-gramm.mkf` using McKeeman
//! form.

use crate::codec::Codec;
use crate::error::{Error::*, Result, err};
use regr::{Epsilon, Graph, Node};
use std::collections::HashMap;
use std::ops::Deref;
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

    /// Parse regular expression specified with `pattern` using `start_node` as
    /// the first node for the builded graph.
    ///
    /// Returns last node of the result graph. This node is not accepable by
    /// default. Make this acceptable by yourself if needed.
    pub fn parse(&mut self, pattern: &'s str, start_node: Node<'n>) -> Result<Node<'n>> {
        assert!(start_node.gid() == self.nfa.gid());
        self.lexer = Lexer::new(pattern);
        let finish_node = self.parse_disjunct(start_node)?;
        if let Some(symbol) = self.lexer.peek() {
            return match symbol {
                ')' | ']' | '}' => err::unexpected_close_bracket(symbol),
                _ => err::unexpected_token(symbol, "`|` or end of pattern"),
            };
        }
        self.lexer = Lexer::empty();
        Ok(finish_node)
    }

    /// Parse disjunction:
    ///
    /// ```mkf
    /// disjunct
    ///     ""
    ///     concat
    ///     concat '|' disjunct
    /// ```
    fn parse_disjunct(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        let end_node = self.parse_concat(start_node)?;
        while let Some(symbol) = self.lexer.peek() {
            if symbol == '|' {
                self.lexer.lex().unwrap();
                let last_node = self.parse_concat(start_node)?;
                last_node.connect(end_node, Epsilon);
            } else {
                return Ok(end_node);
            }
        }
        assert_ne!(start_node, end_node);
        Ok(end_node)
    }

    /// Parse concatenation:
    ///
    /// ```mkf
    /// concat
    ///     item
    ///     item concat
    /// ```
    fn parse_concat(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        let mut end_node = start_node;
        while let Some(node) = self.try_parse_item(end_node)? {
            end_node = node;
        }
        // if there is an empty expression, create an epsilone transition
        if start_node == end_node {
            end_node = self.nfa.node();
            start_node.connect(end_node, Epsilon);
        }
        Ok(end_node)
    }

    /// Parse item:
    ///
    /// ```mkf
    /// item
    ///     term
    ///     class
    ///     '(' disjunct ')'
    ///     item postfix
    /// ```
    fn try_parse_item(&mut self, start_node: Node<'n>) -> Result<Option<Node<'n>>> {
        if let Some(next_sym) = self.lexer.peek() {
            let res = match next_sym {
                '(' => self.parse_parens(start_node),
                ')' => Ok(start_node),
                '[' => self.parse_class(start_node),
                ']' => err::unexpected_close_bracket(next_sym),
                _ => self.parse_term(start_node),
            };
            let mut end_node = res?;
            if end_node == start_node {
                Ok(None)
            } else {
                while let Some(new_end_node) = self.try_parse_postfix(start_node, end_node)? {
                    end_node = new_end_node;
                }
                Ok(Some(end_node))
            }
        } else {
            Ok(None)
        }
    }

    /// Parses postfix:
    ///
    /// ```mkf
    /// postfix
    ///     '*'
    ///     '+'
    ///     '?'
    ///     '{' num '}'
    ///     '{' num ',' num '}'
    /// ```
    fn try_parse_postfix(
        &mut self,
        item_start: Node<'n>,
        item_end: Node<'n>,
    ) -> Result<Option<Node<'n>>> {
        let end_node = {
            if let Some(symbol) = self.lexer.peek() {
                match symbol {
                    '*' => self.parse_star(item_start, item_end)?,
                    '+' => self.parse_plus(item_start, item_end)?,
                    '?' => self.parse_question(item_start, item_end)?,
                    '{' => self.parse_braces(item_start, item_end)?,
                    _ => item_end,
                }
            } else {
                item_end
            }
        };
        if end_node == item_end {
            Ok(None)
        } else {
            Ok(Some(end_node))
        }
    }

    /// Parses Kleene star operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```
    ///  ╭────ε────╮
    ///  ↓         │
    /// (1)──'a'─→(2)──ε─→(3)
    ///  │                 ↑
    ///  ╰────────ε────────╯
    /// ```
    /// instead of
    /// ```
    ///          ╭────ε────╮
    ///          ↓         │
    /// (1)──ε─→(2)──'a'─→(3)──ε─→(4)
    ///  │                         ↑
    ///  ╰────────────ε────────────╯
    /// ```
    fn parse_star(&mut self, item_start: Node<'n>, item_end: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('*')?;
        let new_end_node = self.nfa.node();
        item_end.connect(item_start, Epsilon);
        item_end.connect(new_end_node, Epsilon);
        item_start.connect(new_end_node, Epsilon);
        Ok(new_end_node)
    }

    /// Parses plus operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```
    ///  ╭────ε────╮
    ///  ↓         │
    /// (1)──'a'─→(2)──ε─→(3)
    /// ```
    /// instead of
    /// ```
    ///          ╭────ε────╮
    ///          ↓         │
    /// (1)──ε─→(2)──'a'─→(3)──ε─→(4)
    /// ```
    fn parse_plus(&mut self, item_start: Node<'n>, item_end: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('+')?;
        let new_end_node = self.nfa.node();
        item_end.connect(item_start, Epsilon);
        item_end.connect(new_end_node, Epsilon);
        Ok(new_end_node)
    }

    /// Parses question operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```
    /// (1)──'a'─→(2)──ε─→(3)
    ///  │                 ↑
    ///  ╰────────ε────────╯
    /// ```
    /// instead of
    /// ```
    /// (1)──ε─→(2)──'a'─→(3)──ε─→(4)
    ///  │                         ↑
    ///  ╰────────────ε────────────╯
    /// ```
    fn parse_question(&mut self, item_start: Node<'n>, item_end: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('?')?;
        let new_end_node = self.nfa.node();
        item_end.connect(new_end_node, Epsilon);
        item_start.connect(new_end_node, Epsilon);
        Ok(new_end_node)
    }

    fn parse_braces(&mut self, item_start: Node<'n>, item_end: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('{')?;
        let Some(first_num) = self.parse_decimal() else {
            return err::unexpected_token("nothing", "<decimal>");
        };
        if first_num == 0 {
            return err::nonsense_value(first_num);
        }
        let sym = self.lexer.lex().ok_or_else(|| UnexpectedEof {
            aborted_expr: "braces".into(),
        })?;
        let _second_num = match sym {
            '}' => Some(first_num),
            ',' => {
                if let Some(second_num) = self.parse_decimal() {
                    if second_num < first_num {
                        todo!()
                    }
                    Some(second_num)
                } else {
                    None
                }
            }
            got => return err::unexpected_token(got, "'}' or ','"),
        };
        let (mut start_node, mut end_node) = (item_start, item_end);
        for _ in 1..first_num {
            (start_node, end_node) = self.clone_tail(start_node, end_node);
        }
        // if let Some(second_num) = second_num {
        //     todo!()
        // } else {
        //     todo!()
        // }
        Ok(end_node)
    }

    /// Parse parentheses:
    ///
    /// ```mkf
    ///     '(' disjunct ')'
    /// ```
    fn parse_parens(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('(')?;
        let end_node = self.parse_disjunct(start_node)?;
        self.lexer.expect(')')?;
        Ok(end_node)
    }

    fn parse_class(&mut self, _: Node<'n>) -> Result<Node<'n>> {
        todo!()
    }

    /// Parse terminal:
    ///
    /// ```mkf
    /// term
    ///     char
    ///     '\' escape
    /// ```
    fn parse_term(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        let next_sym = self.lexer.peek().unwrap();
        match next_sym {
            '\\' => self.parse_escape(start_node),
            _ => self.parse_char(start_node),
        }
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
            err::unexpected_eof("escape")
        }
    }

    fn parse_ascii_escape(&mut self) -> Result<u32> {
        let Some(first_hex) = self.lexer.lex() else {
            return err::unexpected_eof("ascii escape");
        };
        let Some(second_hex) = self.lexer.lex() else {
            return err::unexpected_eof("ascii escape");
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
                return err::unexpected_eof("unicode escape");
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

    /// Parse normal unescapable character:
    ///
    /// ```mkf
    /// char
    ///     '0000' . '10FFFF' - '\' - '|' - '.' - '*' - '+' - '?' - '(' - ')' - '[' - ']' - '{' - '}'
    /// ```
    fn parse_char(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        let symbol = self.lexer.lex().unwrap();
        if matches!(
            symbol,
            '\\' | '|' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
        ) {
            return err::escape_it(symbol);
        }
        let mut buffer = [0u8; 4];
        let len = self.codec.encode_char(symbol, &mut buffer)?;
        let mut end_node = start_node;
        for byte in buffer[..len].iter() {
            let new_node = self.nfa.node();
            end_node.connect(new_node, *byte);
            end_node = new_node;
        }
        Ok(end_node)
    }

    /// Parses decimal secquence into `u32` value. If there wasn't found any
    /// dicamal characters, returns zero.
    fn parse_decimal(&mut self) -> Option<u32> {
        let mut num: Option<u32> = None;
        while let Some(sym) = self.lexer.peek() {
            if '0' <= sym && sym <= '9' {
                self.lexer.take_peeked();
                let old_num = num.unwrap_or(0);
                num = Some(old_num + (sym as u32 - '0' as u32));
            } else {
                break;
            }
        }
        num
    }

    /// Returns clone of sub subgraph where all nodes are clones of all nodes
    /// accecible from this node (including a clone of the node itself), and
    /// connected with the same transitions.
    fn clone_tail(&self, start_node: Node<'n>, end_node: Node<'n>) -> (Node<'n>, Node<'n>) {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();

        #[allow(clippy::mutable_key_type)]
        fn rec<'n>(node: Node<'n>, clone: Node<'n>, map: &mut HashMap<Node<'n>, Node<'n>>) {
            map.insert(node, clone);
            for (target, tr) in node.symbol_targets() {
                if map.contains_key(&target) {
                    continue;
                }
                let clone_target = node.owner().node();
                rec(target, clone_target, map);
                clone.connect(clone_target, tr.deref());
            }
            for target in node.epsilon_targets() {
                if map.contains_key(&target) {
                    continue;
                }
                let clone_target = node.owner().node();
                rec(target, clone_target, map);
                clone.connect(clone_target, Epsilon);
            }
        }

        rec(start_node, end_node, &mut map);
        (end_node, map[&end_node])
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

    fn expect(&mut self, expected: char) -> Result<()> {
        if let Some(gotten) = self.lex() {
            if gotten == expected {
                Ok(())
            } else {
                err::unexpected_token(gotten, expected)
            }
        } else {
            err::unexpected_eof("regular")
        }
    }

    fn take_peeked(&mut self) {
        self.peeked = None;
    }
}
