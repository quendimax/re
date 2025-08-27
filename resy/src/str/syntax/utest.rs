use crate::str::error::err;
use crate::str::lexis::Lexer;
use crate::str::syntax::ParserImpl;
use pretty_assertions::assert_eq;
use renc::Utf8Encoder;

#[test]
fn parser_parse_decimal() {
    let parse_decimal = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::new(lexer, &Utf8Encoder);
        parser.parse_decimal()
    };
    assert_eq!(parse_decimal("-1"), Ok(None));
    assert_eq!(parse_decimal("0"), Ok(Some(0)));
    assert_eq!(parse_decimal("000"), Ok(Some(0)));
    assert_eq!(parse_decimal("123"), Ok(Some(123)));
    assert_eq!(parse_decimal("1000000"), Ok(Some(1000000)));
    assert_eq!(
        parse_decimal("1000000000000"),
        err::int_overflow("1000000000000".to_owned(), 0..13)
    );
}
