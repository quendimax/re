use crate::hir::Hir;
use crate::str::error::{Result, err};
use crate::str::lexis::{Lexer, tok};
use renc::Encoder;

pub struct Parser<C: Encoder> {
    encoder: C,
}

impl<C: Encoder> Parser<C> {
    pub fn new(encoder: C) -> Self {
        Parser { encoder }
    }

    pub fn parse(&self, pattern: &str) -> Result<Hir> {
        let mut parser = ParserImpl::new(Lexer::new(pattern), &self.encoder);
        parser.parse()
    }
}

struct ParserImpl<'s, 'c, C: Encoder> {
    lexer: Lexer<'s>,
    coder: &'c C,
}

impl<'s, 'c, C: Encoder> ParserImpl<'s, 'c, C> {
    fn new(lexer: Lexer<'s>, coder: &'c C) -> Self {
        ParserImpl { lexer, coder }
    }

    fn parse(&mut self) -> Result<Hir> {
        _ = self.lexer.lex();
        let hir = self.parse_disjunct()?;
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
        let mut alternatives = vec![];
        let hir = self.parse_concat()?;
        alternatives.push(hir);
        loop {
            let token = self.lexer.peek();
            if token.kind() == tok::pipe {
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
        let mut items = vec![];
        while let Some(hir) = self.parse_item()? {
            items.push(hir);
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
    fn parse_item(&mut self) -> Result<Option<Hir>> {
        let token = self.lexer.peek();
        let hir = match token.kind() {
            tok::l_paren => self.parse_group()?,
            tok::l_paren_question => self.parse_named_group()?,
            tok::dot | tok::l_square | tok::l_square_caret => self.parse_class()?,
            _ => {
                let mut literal = vec![];
                while let Some(c) = self.parse_term()? {
                    if self.lexer.peek().is_term() {
                        let len = literal.len();
                        literal.resize(len + 4, 0);
                        match self.coder.encode_ucp(c, &mut literal[len..len + 4]) {
                            Ok(bytes_num) => literal.resize(len + bytes_num, 0),
                            Err(error) => return err::encoder_error(error, token.span()),
                        }
                    }
                }
                todo!()
            }
        };
        if let Some((iter_min, iter_max)) = self.parse_postfix()? {
            let repeat_hir = Hir::repeat(hir, iter_min, iter_max);
            Ok(Some(repeat_hir))
        } else {
            Ok(Some(hir))
        }
    }

    fn parse_postfix(&mut self) -> Result<Option<(usize, Option<usize>)>> {
        todo!()
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
        self.lexer.expect(tok::char('<'))?;
        if let Some(num) = self.parse_decimal()? {
            self.lexer.expect(tok::char('>'))?;
            let hir = self.parse_disjunct()?;
            self.lexer.expect(tok::r_paren)?;
            Ok(Hir::group(num, hir))
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
    /// ```
    fn parse_class(&mut self) -> Result<Hir> {
        todo!()
    }

    /// Parses a single character as is, and as an escape sequence.
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
    ///     "\x" oct hex
    ///
    /// unicode_escape
    ///     "\u{" hex "}"
    ///     "\u{" hex hex "}"
    ///     "\u{" hex hex hex "}"
    ///     "\u{" hex hex hex hex "}"
    ///     "\u{" hex hex hex hex hex "}"
    ///     "\u{" hex hex hex hex hex hex "}"
    /// ```
    fn parse_term(&mut self) -> Result<Option<u32>> {
        match self.lexer.lex().kind() {
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
                'u' => todo!(),
                _ => panic!("unsupported escape sequence"),
            },
            _ => Ok(None),
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
        if first_digit > '7' {
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

    /// Parses decimal secquence into `u32` value.
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
    fn parse_decimal(&mut self) -> Result<Option<u32>> {
        let mut token = self.lexer.peek();
        if let tok::char(sym) = token.kind()
            && sym.is_ascii_digit()
        {
        } else {
            return Ok(None);
        }

        let mut num: Option<u32> = Some(0);
        let start = token.span().start;
        while let tok::char(sym) = self.lexer.peek().kind()
            && sym.is_ascii_digit()
        {
            token = self.lexer.lex();
            let next_digit = sym as u32 - '0' as u32;
            num = num
                .and_then(|num| num.checked_mul(10))
                .and_then(|num| num.checked_add(next_digit));
        }
        if let Some(num) = num {
            Ok(Some(num))
        } else {
            let span = start..token.span().end;
            let slice = self.lexer.slice(span.clone());
            err::out_of_range(slice, span, "`u32` range")
        }
    }
}

#[cfg(test)]
mod utest;
