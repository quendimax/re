//! There is an implementation of regular expression parser.
//!
//! Grammar used here is described in `resy/docs/unic-gramm.mkf` using McKeeman
//! form.

use crate::error::{Error::*, Result, err};
use redt::Range;
use regr::{Epsilon, Graph, Node};
use renc::Encoder;
use std::collections::HashMap;
use std::str::Chars;

pub struct Parser<'g, 'n, 's, T: Encoder> {
    nfa: &'g Graph<'n>,
    lexer: Lexer<'s>,
    encoder: T,
}

impl<'g, 'n, 's, C: Encoder> Parser<'g, 'n, 's, C> {
    pub fn new(nfa: &'g Graph<'n>, encoder: C) -> Self {
        assert!(nfa.is_nfa(), "this parser can build only an NFA graph");
        let lexer = Lexer::empty();
        Self {
            nfa,
            lexer,
            encoder,
        }
    }

    /// Parses a regular expression specified with `pattern` using `start_node`
    /// as the first node for the builded graph.
    ///
    /// Returns last node of the result graph. This node is not accepable by
    /// default. Make this acceptable by yourself if it is needed.
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

    /// Parses a disjunction expression. If nothing was parsed, returns
    /// `start_node`, else the last node of new graph's tail.
    ///
    /// ```mkf
    /// disjunct
    ///     concat
    ///     concat '|' disjunct
    /// ```
    ///
    /// # Implementation
    ///
    /// This method uses the last node of the first concatination expression as
    /// the last node of the entire disjunction expression. All the other concat
    /// expressions are connected to this node with Epsilone joint.
    ///
    /// So it will build from pattern `a|b|c` the following graph:
    ///
    /// ```c
    ///  ╭────────'a'────────╮
    ///  │                   ↓
    /// (0)──'b'─→(2)───ε──→(1)
    ///  │                   ↑
    ///  ╰───'c'─→(3)───ε────╯
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
        debug_assert_ne!(start_node, end_node);
        Ok(end_node)
    }

    /// Parses a concatenation expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// concat
    ///     ""
    ///     item
    ///     item concat
    /// ```
    fn parse_concat(&mut self, start_node: Node<'n>) -> Result<Node<'n>> {
        let mut end_node = start_node;
        while let Some(node) = self.try_parse_item(end_node)? {
            end_node = node;
        }
        if start_node == end_node {
            // if nothing parsed, it is an Epsilon transition
            end_node = self.nfa.node();
            start_node.connect(end_node, Epsilon);
        }
        Ok(end_node)
    }

    /// Parses an item.
    ///
    /// # Syntax
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
                '|' => Ok(start_node),
                ')' => Ok(start_node),
                '[' => self.parse_class(start_node, self.nfa.node()),
                '.' => self.parse_dot_class(start_node, self.nfa.node()),
                _ => self.parse_term(start_node, None),
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

    /// Parses a postfix.
    ///
    /// # Syntax
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

    /// Parses a Kleene star operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```c
    ///  ╭────ε────╮
    ///  ↓         │
    /// (1)──'a'─→(2)──ε─→(3)
    ///  │                 ↑
    ///  ╰────────ε────────╯
    /// ```
    /// instead of
    /// ```c
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

    /// Parses a plus operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```c
    ///  ╭────ε────╮
    ///  ↓         │
    /// (1)──'a'─→(2)──ε─→(3)
    /// ```
    /// instead of
    /// ```c
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

    /// Parses a question operator.
    ///
    /// I use here a bit modified Thompson's construction:
    /// ```c
    /// (1)──'a'─→(2)──ε─→(3)
    ///  │                 ↑
    ///  ╰────────ε────────╯
    /// ```
    /// instead of
    /// ```c
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

    /// Parses a braces postfix.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// postfix
    ///     ; ...
    ///     '{' decimal '}'
    ///     '{' decimal ',' '}'
    ///     '{' decimal ',' decimal '}'
    /// ```
    fn parse_braces(&mut self, item_start: Node<'n>, item_end: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('{')?;
        let Some(first_num) = self.parse_decimal() else {
            let gotten = self.lexer.peek().unwrap();
            return err::unexpected_token(gotten, "decimal");
        };
        let sym = self.lexer.peek().ok_or_else(|| UnexpectedEof {
            aborted_expr: "braces".into(),
        })?;
        let second_num = match sym {
            '}' => Some(first_num),
            ',' => {
                self.lexer.take_peeked();
                if let Some(second_num) = self.parse_decimal() {
                    if second_num < first_num {
                        return err::unexpected_cond("expression `{n,m}` has invariant `n` <= `m`");
                    }
                    Some(second_num)
                } else {
                    None
                }
            }
            got => return err::unexpected_token(got, "'}' or ','"),
        };
        if let Some(second_num) = second_num {
            if first_num == 0 && second_num == 0 {
                // `x{0}` and `x{0,0}` are an empty transition. Because of it is
                // difficult to implement it `perfectly` in current implementation,
                // I just forbid it.
                return err::nonsense_value(first_num);
            }
        }
        self.lexer.expect('}')?;
        let (mut start_node, mut end_node) = (item_start, item_end);
        for _ in 1..first_num {
            (start_node, end_node) = self.clone_tail(start_node, end_node);
        }
        if let Some(second_num) = second_num {
            // as N `?`-operators
            if first_num < second_num {
                if first_num > 0 {
                    (start_node, end_node) = self.clone_tail(start_node, end_node);
                }
                let node = self.nfa.node();
                start_node.connect(node, Epsilon);
                end_node.connect(node, Epsilon);
                end_node = node;
                for _ in first_num..second_num - 1 {
                    (start_node, end_node) = self.clone_tail(start_node, end_node);
                }
            }
        } else {
            // as `*`-operator
            if first_num > 0 {
                (start_node, end_node) = self.clone_tail(start_node, end_node);
            }
            let node = self.nfa.node();
            start_node.connect(node, Epsilon);
            end_node.connect(start_node, Epsilon);
            end_node.connect(node, Epsilon);
            end_node = node;
        }
        Ok(end_node)
    }

    /// Parses parentheses:
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

    /// Parses dot class:
    ///
    /// ```mkf
    /// class
    ///     '.'
    /// ```
    fn parse_dot_class(&mut self, start_node: Node<'n>, end_node: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('.')?;
        self.encoder.encode_entire_range(|seq| {
            self.build_from_sequence(seq, start_node, end_node);
        })?;
        Ok(end_node)
    }

    /// Parses a class:
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// class
    ///     '[' elements ']'
    ///
    /// elements
    ///     element
    ///     element elements
    ///
    /// element
    ///     '.'
    ///     term
    ///     term '-' term
    /// ```
    fn parse_class(&mut self, start_node: Node<'n>, end_node: Node<'n>) -> Result<Node<'n>> {
        self.lexer.expect('[')?;
        if self.lexer.peek() == Some(']') {
            return Err(EmptyClass);
        }

        while let Some(sym) = self.lexer.peek() {
            match sym {
                ']' => break,
                '.' => _ = self.parse_dot_class(start_node, end_node)?,
                _ => {
                    let first_ucp = self.parse_term_codepoint()?;
                    if self.lexer.peek() == Some('-') {
                        _ = self.lexer.take_peeked();
                        let last_ucp = self.parse_term_codepoint()?;
                        self.encoder.encode_range(first_ucp, last_ucp, |seq| {
                            self.build_from_sequence(seq, start_node, end_node);
                        })?;
                    } else {
                        self.build_from_codepoint(first_ucp, start_node, Some(end_node))?;
                    }
                }
            };
        }
        self.lexer.expect(']')?;
        Ok(end_node)
    }

    /// Parses a terminal item, that is a single unicode code point.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// term
    ///     char
    ///     '\' escape
    /// ```
    fn parse_term(&mut self, start_node: Node<'n>, end_node: Option<Node<'n>>) -> Result<Node<'n>> {
        let codepoint = self.parse_term_codepoint()?;
        self.build_from_codepoint(codepoint, start_node, end_node)
    }

    fn parse_term_codepoint(&mut self) -> Result<u32> {
        match self.lexer.peek() {
            Some('\\') => self.parse_escape(),
            Some(_) => self.parse_char(),
            None => err::unexpected_eof("regular"),
        }
    }

    /// Parses an escape sequence.
    ///
    /// It is intended to be compatible with the Rust string literal escape
    /// sequences.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// escape
    ///     '"'
    ///     '.'
    ///     '*'
    ///     '+'
    ///     '-'
    ///     '?'
    ///     '\'
    ///     '|'
    ///     '('
    ///     ')'
    ///     '['
    ///     ']'
    ///     '{'
    ///     '}'
    ///     '0'
    ///     'n'
    ///     'r'
    ///     't'
    ///     'x' oct hex
    ///     'u' '{' hex '}'
    ///     'u' '{' hex hex '}'
    ///     'u' '{' hex hex hex '}'
    ///     'u' '{' hex hex hex hex '}'
    ///     'u' '{' hex hex hex hex hex '}'
    ///     'u' '{' hex hex hex hex hex hex '}'
    /// ```
    ///
    /// There are escape sequences compatible with Rust string literals:
    /// - `\"` - double quote,
    /// - `\x<two hexes between 00 and 7F>` - 7-bit code point escape,
    /// - `\u{<up to six hexes>}` - 24-bit code point escape,
    /// - `\n` - line feed escape,
    /// - `\r` - carriage return escape,
    /// - `\t` - horizontal tab escape,
    /// - `\0` - null escape,
    /// - `\\` - backslash.
    fn parse_escape(&mut self) -> Result<u32> {
        self.lexer.expect('\\')?;
        if let Some(c) = self.lexer.lex() {
            let codepoint: Option<u32> = match c {
                '"' | '.' | '*' | '+' | '-' | '?' | '\\' | '|' | '(' | ')' | '[' | ']' | '{'
                | '}' => Some(c as u32),
                'n' => Some('\n' as u32),
                'r' => Some('\r' as u32),
                't' => Some('\t' as u32),
                '0' => Some('\0' as u32),
                'x' => Some(self.parse_ascii_escape()?),
                'u' => Some(self.parse_unicode_escape()?),
                _ => None,
            };
            if let Some(codepoint) = codepoint {
                Ok(codepoint)
            } else {
                Err(UnsupportedEscape(c))
            }
        } else {
            err::unexpected_eof("escape")
        }
    }

    /// Parses an ascii escape sequence.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// escape
    ///     ; ...
    ///     'x' oct hex
    ///     ; ...
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
            return err::invalid_hex(hex_str);
        }
        if first_hex > '7' {
            let mut hex_str = first_hex.to_string();
            hex_str.push(second_hex);
            return err::out_of_range(hex_str, "[0x00..=0x7F]");
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

    /// Parses a unicode escape sequence.
    ///
    /// # Syntax
    /// ```mkf
    /// escape
    ///     ; ...
    ///     'u' '{' hex '}'
    ///     'u' '{' hex hex '}'
    ///     'u' '{' hex hex hex '}'
    ///     'u' '{' hex hex hex hex '}'
    ///     'u' '{' hex hex hex hex hex '}'
    ///     'u' '{' hex hex hex hex hex hex '}'
    /// ```
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
                return err::invalid_hex(c);
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
    /// # Syntax
    /// ```mkf
    /// char
    ///     '0000' . '10FFFF' - '\' - '|' - '.' - '*' - '+' - '?' - '(' - ')' - '[' - ']' - '{' - '}'
    /// ```
    fn parse_char(&mut self) -> Result<u32> {
        let symbol = self.lexer.lex().unwrap();
        if matches!(
            symbol,
            '\\' | '|' | '.' | '*' | '+' | '-' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
        ) {
            return err::escape_it(symbol);
        }
        Ok(symbol as u32)
    }

    /// Parses decimal secquence into `u32` value. If there wasn't found any
    /// dicamal characters, returns `None`.
    fn parse_decimal(&mut self) -> Option<u32> {
        let mut num: Option<u32> = None;
        while let Some(sym) = self.lexer.peek() {
            if sym.is_ascii_digit() {
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

        struct Lambda<'a, 'n> {
            map: &'a mut HashMap<Node<'n>, Node<'n>>,
            nfa: &'a Graph<'n>,
        }
        impl<'a, 'n> Lambda<'a, 'n> {
            // maybe `tr.clone()` can be replaced with something more efficient
            fn copy_tail(&mut self, node: Node<'n>, clone: Node<'n>) {
                self.map.insert(node, clone);
                let mut collect = Vec::new();
                for (target, tr) in node.targets().iter() {
                    if let Some(clone_target) = self.map.get(target) {
                        collect.push((*clone_target, tr.clone()));
                    } else {
                        let clone_target = self.nfa.node();
                        self.copy_tail(*target, clone_target);
                        collect.push((clone_target, tr.clone()));
                    }
                }
                for (clone_target, tr) in collect {
                    clone.connect(clone_target, &tr);
                }
            }
        }

        Lambda {
            map: &mut map,
            nfa: self.nfa,
        }
        .copy_tail(start_node, end_node);

        (end_node, map[&end_node])
    }

    fn build_from_codepoint(
        &self,
        codepoint: u32,
        start_node: Node<'n>,
        end_node: Option<Node<'n>>,
    ) -> Result<Node<'n>> {
        let mut buf = [0u8; 8];
        let len = self.encoder.encode_ucp(codepoint, &mut buf)?;

        let mut prev_node = start_node;
        for (i, byte) in buf[0..len].iter().enumerate() {
            let new_node = if i == len - 1 {
                end_node.unwrap_or_else(|| self.nfa.node())
            } else {
                self.nfa.node()
            };
            prev_node.connect(new_node, *byte);
            prev_node = new_node;
        }
        Ok(prev_node)
    }

    fn build_from_sequence(&self, seq: &[Range<u8>], start_node: Node<'n>, end_node: Node<'n>) {
        let mut prev_node = start_node;
        let len = seq.len();
        for (i, range) in seq.iter().enumerate() {
            let new_node = if i == len - 1 {
                end_node
            } else {
                self.nfa.node()
            };
            prev_node.connect(new_node, range);
            prev_node = new_node;
        }
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
                err::unexpected_token(gotten, format!("'{expected}'"))
            }
        } else {
            err::unexpected_eof("regular")
        }
    }

    fn take_peeked(&mut self) -> Option<char> {
        self.peeked.take()
    }
}
