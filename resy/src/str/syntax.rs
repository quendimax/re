use crate::hir::Hir;
use crate::str::error::{Result, err};
use crate::str::lexis::{Lexer, tok};
use redt::{RangeSet, SetU8};
use renc::Encoder;

pub struct Parser<C: Encoder> {
    encoder: C,
}

impl<C: Encoder> Parser<C> {
    pub fn new(encoder: C) -> Self {
        Parser { encoder }
    }

    pub fn parse(&self, pattern: &str) -> Result<Hir> {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<C>::new(lexer, &self.encoder);
        parser.parse()
    }
}

struct ParserImpl<'s, 'c, C: Encoder, const UNICODE: bool = true> {
    lexer: Lexer<'s>,
    coder: &'c C,
}

impl<'s, 'c, C: Encoder, const UNICODE: bool> ParserImpl<'s, 'c, C, UNICODE> {
    fn new(lexer: Lexer<'s>, coder: &'c C) -> Self {
        ParserImpl { lexer, coder }
    }

    fn parse(&mut self) -> Result<Hir> {
        let hir = self.parse_disjunct()?;
        self.lexer.expect(tok::eof)?;
        Ok(hir)
    }

    /// Parses a disjunction expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// disjunct
    ///     concat
    ///     concat '|' disjunct
    /// ```
    fn parse_disjunct(&mut self) -> Result<Hir> {
        let first_hir = self.parse_concat()?;
        let mut alternatives = Vec::new();
        alternatives.push(first_hir);
        loop {
            if self.lexer.peek().kind() == tok::pipe {
                self.lexer.consume_peeked();
                let hir = self.parse_concat()?;
                alternatives.push(hir);
            } else {
                break;
            }
        }
        Ok(Hir::disjunct(alternatives))
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
    fn parse_concat(&mut self) -> Result<Hir> {
        let mut items = Vec::<Hir>::new();
        while let Some(hir) = self.try_parse_item()? {
            if let Hir::Literal(literal) = &hir
                && let Some(last_hir) = items.last_mut()
                && let Hir::Literal(last_literal) = last_hir
            {
                last_literal.extend_from_slice(literal);
            } else {
                items.push(hir);
            }
        }
        Ok(Hir::concat(items))
    }

    /// Parses a item expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// item
    ///     term
    ///     class
    ///     group
    ///     item postfix
    /// ```
    fn try_parse_item(&mut self) -> Result<Option<Hir>> {
        let token = self.lexer.peek();
        let mut hir = match token.kind() {
            tok::l_paren => self.parse_group()?,
            tok::l_paren_question => self.parse_named_group()?,
            tok::dot | tok::l_square | tok::l_square_caret => self.parse_class()?,
            _ => {
                if let Some(c) = self.try_parse_term()? {
                    let mut literal = vec![0, 0, 0, 0, 0, 0, 0, 0];
                    match self.coder.encode_ucp(c, &mut literal[..]) {
                        Ok(len) => literal.resize(len, 0),
                        Err(error) => return err::encoder_error(error, token.span()),
                    }
                    Hir::literal(literal)
                } else {
                    return Ok(None);
                }
            }
        };
        while let Some((iter_min, iter_max)) = self.try_parse_postfix()? {
            hir = Hir::repeat(hir, iter_min, iter_max);
        }
        Ok(Some(hir))
    }

    /// Parses postfix operators.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// postfix
    ///     '*'
    ///     '+'
    ///     '?'
    ///     '{' decimal '}'
    ///     '{' decimal ',' '}'
    ///     '{' decimal ',' decimal '}'
    /// ```
    fn try_parse_postfix(&mut self) -> Result<Option<(usize, Option<usize>)>> {
        let token = self.lexer.peek();
        match token.kind() {
            tok::star => {
                self.lexer.consume_peeked();
                Ok(Some((0, None)))
            }
            tok::plus => {
                self.lexer.consume_peeked();
                Ok(Some((1, None)))
            }
            tok::question => {
                self.lexer.consume_peeked();
                Ok(Some((0, Some(1))))
            }
            tok::l_brace => Ok(Some(self.parse_braces()?)),
            _ => Ok(None),
        }
    }

    /// Parses count of iterations within braces..
    ///
    /// # Syntax
    ///
    /// ```mkf
    ///     '{' decimal '}'
    ///     '{' decimal ',' '}'
    ///     '{' decimal ',' decimal '}'
    /// ```
    fn parse_braces(&mut self) -> Result<(usize, Option<usize>)> {
        let l_brace = self.lexer.expect(tok::l_brace)?;
        let Some(first_num) = self.try_parse_decimal()? else {
            let span = l_brace.end()..self.lexer.lex().end();
            let spell = self.lexer.slice(span.clone());
            return err::unexpected(spell, span, "a decimal number");
        };
        let peeked = self.lexer.peek();
        let second_num = match peeked.kind() {
            tok::r_brace => Some(first_num),
            tok::char(',') => {
                self.lexer.consume_peeked();
                self.try_parse_decimal()?
            }
            _ => {
                let spell = self.lexer.slice(peeked.span());
                return err::unexpected(spell, peeked.span(), "either `}` or `,`");
            }
        };
        let r_brace = self.lexer.expect(tok::r_brace)?;
        let span = l_brace.start()..r_brace.end();
        match (first_num, second_num) {
            (0, Some(0)) => err::zero_repetition(span),
            (n, Some(m)) if n > m => err::invalid_repetition(span),
            _ => Ok((first_num, second_num)),
        }
    }

    /// Parses a group expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// group
    ///     "(" disjunct ")"
    /// ```
    fn parse_group(&mut self) -> Result<Hir> {
        self.lexer.expect(tok::l_paren)?;
        let hir = self.parse_disjunct()?;
        self.lexer.expect(tok::r_paren)?;
        Ok(hir)
    }

    /// Parses a named group expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// group
    ///     "(?" label disjunct ")"
    ///
    /// label
    ///     '<' decimal '>'
    /// ```
    fn parse_named_group(&mut self) -> Result<Hir> {
        self.lexer.expect(tok::l_paren_question)?;
        let l_angle = self.lexer.expect(tok::char('<'))?;
        if let Some(num) = self.try_parse_decimal()? {
            if let Ok(num) = u32::try_from(num) {
                self.lexer.expect(tok::char('>'))?;
                let hir = self.parse_disjunct()?;
                self.lexer.expect(tok::r_paren)?;
                Ok(Hir::group(num, hir))
            } else {
                let span = l_angle.span().end..self.lexer.end_pos();
                let spell = self.lexer.slice(span.clone());
                err::out_of_range(spell, span, "`u32` range")
            }
        } else {
            let unexpected_token = self.lexer.peek();
            let slice = self.lexer.slice(unexpected_token.span());
            err::unexpected(slice, unexpected_token.span(), "decimal")
        }
    }

    /// Parses a class expression.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// class
    ///     '.'
    ///     "[" elements "]"
    ///     "[^" elements "]"
    ///
    /// elements
    ///     element
    ///     element elements
    ///
    /// element
    ///     term
    ///     term '-' term
    ///     class
    /// ```
    fn parse_class(&mut self) -> Result<Hir> {
        let token = self.lexer.peek();
        let range_set = match token.kind() {
            tok::dot => self.parse_dot()?,
            tok::l_square => self.parse_squares()?,
            tok::l_square_caret => self.parse_squares_negated()?,
            _ => {
                let slice = self.lexer.slice(token.span());
                return err::unexpected(slice, token.span(), "dot or square brackets");
            }
        };

        if range_set.is_empty() {
            return Ok(Hir::empty());
        }

        let mut alternatives = Vec::new();
        for cp_range in range_set.ranges() {
            let hir = self.convert(cp_range.start(), cp_range.last());
            alternatives.push(hir);
        }
        Ok(Hir::disjunct(alternatives))
    }

    fn parse_dot(&mut self) -> Result<RangeSet<u32>> {
        self.lexer.expect(tok::dot)?;
        let encoding = self.coder.encoding();
        Ok(RangeSet::from(encoding.codepoint_ranges()))
    }

    /// Parses a class with square brackets.
    fn parse_squares(&mut self) -> Result<RangeSet<u32>> {
        self.lexer.expect(tok::l_square)?;
        let mut ranges = RangeSet::default();
        loop {
            let token = self.lexer.peek();
            let range_set = match token.kind() {
                tok::dot => self.parse_dot()?,
                tok::l_square => self.parse_squares()?,
                tok::l_square_caret => self.parse_squares_negated()?,
                tok::r_square => break,
                _ => self.parse_range()?,
            };
            for range in range_set.ranges() {
                ranges.merge(range);
            }
        }
        self.lexer.expect(tok::r_square)?;
        Ok(ranges)
    }

    /// Parses a class with square brackets.
    fn parse_squares_negated(&mut self) -> Result<RangeSet<u32>> {
        self.lexer.expect(tok::l_square_caret)?;
        let encoding = self.coder.encoding();
        let mut ranges = RangeSet::from(encoding.codepoint_ranges());
        loop {
            let token = self.lexer.peek();
            let range_set = match token.kind() {
                tok::dot => self.parse_dot()?,
                tok::l_square => self.parse_squares()?,
                tok::l_square_caret => self.parse_squares_negated()?,
                tok::r_square => break,
                _ => self.parse_range()?,
            };
            for range in range_set.ranges() {
                ranges.exclude(range);
            }
        }
        self.lexer.expect(tok::r_square)?;
        Ok(ranges)
    }

    fn parse_range(&mut self) -> Result<RangeSet<u32>> {
        let start_codepoint = self.parse_term()?;
        if let tok::minus = self.lexer.peek().kind() {
            self.lexer.consume_peeked();
            let last_codepoint = self.parse_term()?;
            Ok(RangeSet::new(start_codepoint, last_codepoint))
        } else {
            Ok(RangeSet::new(start_codepoint, start_codepoint))
        }
    }

    /// Parses a sequence corresponding to one code point, i.e. either a single
    /// character or an escape sequence. If there is no one, returns `None`.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// term
    ///     char
    ///     escape
    ///
    /// char
    ///     '0000' . '10FFFF' - '\' - '.' - '*' - '+' - '-' - '?' - '|' - '(' - ')' - '[' - ']' - '{' - '}'
    ///
    /// escape
    ///     ascii_escape
    ///     unicode_escape
    ///
    /// ascii_escape
    ///     "\\"
    ///     "\."
    ///     "\*"
    ///     "\+"
    ///     "\-"
    ///     "\?"
    ///     "\|"
    ///     "\("
    ///     "\)"
    ///     "\["
    ///     "\]"
    ///     "\{"
    ///     "\}"
    ///     "\0"
    ///     "\n"
    ///     "\r"
    ///     "\t"
    /// ```
    fn try_parse_term(&mut self) -> Result<Option<u32>> {
        let token = self.lexer.lex();
        match token.kind() {
            tok::char(c) => Ok(Some(c as u32)),
            tok::escape_char(c) => match c {
                '\\' => Ok(Some('\\' as u32)),
                '.' => Ok(Some('.' as u32)),
                '*' => Ok(Some('*' as u32)),
                '+' => Ok(Some('+' as u32)),
                '-' => Ok(Some('-' as u32)),
                '?' => Ok(Some('?' as u32)),
                '|' => Ok(Some('|' as u32)),
                '(' => Ok(Some('(' as u32)),
                ')' => Ok(Some(')' as u32)),
                '[' => Ok(Some('[' as u32)),
                ']' => Ok(Some(']' as u32)),
                '{' => Ok(Some('{' as u32)),
                '}' => Ok(Some('}' as u32)),
                '0' => Ok(Some('\0' as u32)),
                'n' => Ok(Some('\n' as u32)),
                'r' => Ok(Some('\r' as u32)),
                't' => Ok(Some('\t' as u32)),
                'x' => Ok(Some(self.parse_hex_escape()?)),
                'u' if UNICODE => Ok(Some(self.parse_unicode_escape()?)),
                _ => {
                    let spell = self.lexer.slice(token.span());
                    err::unsupported_escape(spell, token.span())
                }
            },
            _ => Ok(None),
        }
    }

    fn parse_term(&mut self) -> Result<u32> {
        let start = self.lexer.end_pos();
        if let Some(codepoint) = self.try_parse_term()? {
            Ok(codepoint)
        } else {
            let span = start..self.lexer.end_pos();
            let spell = self.lexer.slice(span.clone());
            err::unexpected(spell, span, "a character or an escape sequence")
        }
    }

    /// Parses a hexadecimal escape sequence `\xOH` where O is an octal digit
    /// and H is a hex digit. Returns the value of corresponding ASCII character
    /// (0-127).
    ///
    /// # Syntax
    ///
    /// ```mkf
    ///     "\x" oct hex
    /// ```
    fn parse_hex_escape(&mut self) -> Result<u32> {
        let first_token = self.lexer.lex();
        let tok::char(first_digit) = first_token.kind() else {
            let slice = self.lexer.slice(first_token.span());
            return err::unexpected(slice, first_token.span(), "a hexadecimal digit");
        };
        let second_token = self.lexer.lex();
        let tok::char(second_digit) = second_token.kind() else {
            let slice = self.lexer.slice(second_token.span());
            return err::unexpected(slice, second_token.span(), "a hexadecimal digit");
        };
        if !first_digit.is_ascii_hexdigit() || !second_digit.is_ascii_hexdigit() {
            let span = first_token.span().start..second_token.span().end;
            let slice = self.lexer.slice(span.clone());
            return err::unexpected(slice, span, "two hexadecimal digits");
        }
        if UNICODE && first_digit > '7' {
            let span = first_token.span().start - 2..second_token.span().end;
            let slice = self.lexer.slice(span.clone());
            return err::out_of_range(format!("`{slice}`"), span, "ASCII range");
        }
        let mut codepoint = (first_digit as u32 - '0' as u32) << 4;
        if second_digit > '9' {
            const UPPERCASE_MASK: u32 = !0b0010_0000;
            codepoint |= ((second_digit as u32 - 'A' as u32) & UPPERCASE_MASK) + 10;
        } else {
            codepoint |= (second_digit as u32).wrapping_sub('0' as u32);
        }

        // for 7 bit codepoint must always be a correct unicode codepoint
        debug_assert!(char::from_u32(codepoint).is_some());
        Ok(codepoint)
    }

    /// Parses a unicode escape sequence.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// unicode_escape
    ///     "\u{" hex "}"
    ///     "\u{" hex hex "}"
    ///     "\u{" hex hex hex "}"
    ///     "\u{" hex hex hex hex "}"
    ///     "\u{" hex hex hex hex hex "}"
    ///     "\u{" hex hex hex hex hex hex "}"
    /// ```
    fn parse_unicode_escape(&mut self) -> Result<u32> {
        let l_brace = self.lexer.expect(tok::l_brace)?;
        let start = l_brace.span().start - 2;
        let mut codepoint = 0u32;
        for i in 0..6 {
            let token = self.lexer.lex();
            match token.kind() {
                tok::r_brace => {
                    if i == 0 {
                        return err::empty_escape(start..token.span().end);
                    } else {
                        return Ok(codepoint);
                    }
                }
                tok::char(c) if c.is_ascii_hexdigit() => {
                    codepoint <<= 4;
                    if c > '9' {
                        const UPPERCASE_MASK: u32 = !0b0010_0000;
                        codepoint |= ((c as u32 - 'A' as u32) & UPPERCASE_MASK) + 10;
                    } else {
                        codepoint |= (c as u32).wrapping_sub('0' as u32);
                    }
                }
                _ => {
                    let spell = self.lexer.slice(token.span());
                    return err::unexpected(
                        spell,
                        token.span(),
                        "either a hexadecimal digit or a closing brace",
                    );
                }
            }
        }
        self.lexer.expect(tok::r_brace)?;
        Ok(codepoint)
    }

    /// Parses decimal secquence into `usize` value.
    ///
    /// If successfully parsed, returns `Ok(Some(value))`. If there wasn't found
    /// any decimal characters, returns `Ok(None)`. If the found value is out of
    /// range of `u32`, returns `Err(Error::Overflow)`.
    ///
    /// # Syntax
    ///
    /// ```mkf
    /// decimal
    ///     dec
    ///     dec decimal
    ///
    /// dec
    ///     '0' . '9'
    /// ```
    fn try_parse_decimal(&mut self) -> Result<Option<usize>> {
        let token = self.lexer.peek();
        if let tok::char(sym) = token.kind()
            && sym.is_ascii_digit()
        {
        } else {
            return Ok(None);
        }

        let mut num: Option<usize> = Some(0);
        while let tok::char(sym) = self.lexer.peek().kind()
            && sym.is_ascii_digit()
        {
            self.lexer.consume_peeked();
            let next_digit = sym as usize - '0' as usize;
            num = num
                .and_then(|num| num.checked_mul(10))
                .and_then(|num| num.checked_add(next_digit));
        }
        if let Some(num) = num {
            Ok(Some(num))
        } else {
            let span = token.span().start..self.lexer.end_pos();
            let slice = self.lexer.slice(span.clone());
            err::out_of_range(slice, span, "allowed range")
        }
    }

    /// Converts a range of code points to a Hir.
    fn convert(&self, first_codepoint: u32, last_codepoint: u32) -> Hir {
        let mut alternatives = Vec::new();
        self.coder
            .encode_range(first_codepoint, last_codepoint, |seq| {
                let mut items = Vec::new();
                for b_range in seq {
                    let mut b_set = SetU8::new();
                    b_set.merge_range(*b_range);
                    items.push(Hir::class(b_set));
                }
                if items.len() == 1 {
                    alternatives.push(items.pop().unwrap());
                } else {
                    alternatives.push(Hir::concat(items));
                }
            });
        Hir::disjunct(alternatives)
    }
}

#[cfg(test)]
#[path = "syntax.utest.rs"]
mod utest;
