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
            _ => todo!(),
        };
        Ok(Some(hir))
    }

    fn parse_group(&mut self) -> Result<Hir> {
        self.lexer.expect(tok::l_paren)?;
        let hir = self.parse_disjunct()?;
        self.lexer.expect(tok::r_paren)?;
        Ok(hir)
    }

    fn parse_named_group(&mut self) -> Result<Hir> {
        self.lexer.expect(tok::l_paren_question)?;
        self.lexer.expect(tok::char('<'))?;
        if let Some(num) = self.parse_decimal() {
            self.lexer.expect(tok::char('>'))?;
            let hir = self.parse_disjunct()?;
            self.lexer.expect(tok::r_paren)?;
            Ok(Hir::group(num, hir))
        } else {
            let next_token = self.lexer.peek();
            err::unexpected_token(
                "decimal".to_string(),
                self.lexer.slice(next_token).to_string(),
                next_token,
            )
        }
    }

    fn parse_class(&mut self) -> Result<Hir> {
        todo!()
    }

    /// Parses decimal secquence into `u32` value. If there wasn't found any
    /// dicamal characters, returns `None`.
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
