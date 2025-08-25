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
        let mut parser = ParserImpl {
            lexer: Lexer::new(pattern),
            encoder: &self.encoder,
        };
        parser.parse()
    }
}

struct ParserImpl<'s, 'c, C: Encoder> {
    lexer: Lexer<'s>,
    encoder: &'c C,
}

impl<'s, 'c, C: Encoder> ParserImpl<'s, 'c, C> {
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
                        match self.encoder.encode_char(c, &mut literal[len..len + 4]) {
                            Ok(bytes_num) => literal.resize(len + bytes_num, 0),
                            Err(error) => return err::encoder_error(error, token),
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
        if let Some(num) = self.parse_decimal() {
            self.lexer.expect(tok::char('>'))?;
            let hir = self.parse_disjunct()?;
            self.lexer.expect(tok::r_paren)?;
            Ok(Hir::group(num, hir))
        } else {
            let unexpected_tok = self.lexer.peek();
            err::unexpected_token(
                "decimal".to_string(),
                self.lexer.slice(unexpected_tok).to_string(),
                unexpected_tok,
            )
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

    fn parse_term(&mut self) -> Result<Option<char>> {
        match self.lexer.lex().kind() {
            tok::char(c) => Ok(Some(c)),
            tok::escape_char(c) => match c {
                '\\' => Ok(Some('\\')),
                '.' => Ok(Some('.')),
                '*' => Ok(Some('*')),
                '+' => Ok(Some('+')),
                '-' => Ok(Some('-')),
                '?' => Ok(Some('?')),
                '|' => Ok(Some('|')),
                '(' => Ok(Some('(')),
                ')' => Ok(Some(')')),
                '[' => Ok(Some('[')),
                ']' => Ok(Some(']')),
                '{' => Ok(Some('{')),
                '}' => Ok(Some('}')),
                '0' => Ok(Some('\0')),
                'n' => Ok(Some('\n')),
                'r' => Ok(Some('\r')),
                't' => Ok(Some('\t')),
                'x' => todo!(),
                'u' => todo!(),
                _ => panic!("unsupported escape sequence"),
            },
            _ => panic!("unexpected token"),
        }
    }

    /// Parses decimal secquence into `u32` value. If there wasn't found any
    /// dicamal characters, returns `None`.
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
    fn parse_decimal(&mut self) -> Option<u32> {
        let mut num: Option<u32> = None;
        loop {
            if let tok::char(sym) = self.lexer.peek().kind()
                && sym.is_ascii_digit()
            {
                self.lexer.consume_peeked();
                let prev_num = num.unwrap_or(0);
                num = Some(prev_num + (sym as u32 - '0' as u32));
            } else {
                break;
            }
        }
        num
    }
}
