use crate::hir::Hir;
use crate::str::error::Result;
use crate::str::lexis::Lexer;
use renc::Encoder;

pub struct Parser<C: Encoder> {
    encoder: C,
}

impl<C: Encoder> Parser<C> {
    pub fn new(encoder: C) -> Self {
        Parser { encoder }
    }

    pub fn parse(&self, pattern: &str) -> Result<Hir> {
        let mut parser = ParserInner {
            lexer: Lexer::new(pattern),
            encoder: &self.encoder,
        };
        parser.parse()
    }
}

struct ParserInner<'s, 'c, C: Encoder> {
    lexer: Lexer<'s>,
    encoder: &'c C,
}

impl<'s, 'c, C: Encoder> ParserInner<'s, 'c, C> {
    fn parse(&mut self) -> Result<Hir> {
        _ = self.lexer.lex();
        todo!()
    }
}
