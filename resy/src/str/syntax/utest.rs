use crate::str::error::err;
use crate::str::lexis::Lexer;
use crate::str::syntax::ParserImpl;
use pretty_assertions::assert_eq;
use renc::Utf8Encoder;

#[test]
fn parse_term() {
    let parse_term = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::new(lexer, &Utf8Encoder);
        parser.parse_term()
    };
    assert_eq!(parse_term("a"), Ok(Some('a' as u32)));
    assert_eq!(parse_term("/"), Ok(Some('/' as u32)));
    assert_eq!(parse_term(r"\\"), Ok(Some('\\' as u32)));
    assert_eq!(parse_term(r"\."), Ok(Some('.' as u32)));
    assert_eq!(parse_term(r"\*"), Ok(Some('*' as u32)));
    assert_eq!(parse_term(r"\+"), Ok(Some('+' as u32)));
    assert_eq!(parse_term(r"\-"), Ok(Some('-' as u32)));
    assert_eq!(parse_term(r"\?"), Ok(Some('?' as u32)));
    assert_eq!(parse_term(r"\|"), Ok(Some('|' as u32)));
    assert_eq!(parse_term(r"\("), Ok(Some('(' as u32)));
    assert_eq!(parse_term(r"\)"), Ok(Some(')' as u32)));
    assert_eq!(parse_term(r"\["), Ok(Some('[' as u32)));
    assert_eq!(parse_term(r"\]"), Ok(Some(']' as u32)));
    assert_eq!(parse_term(r"\{"), Ok(Some('{' as u32)));
    assert_eq!(parse_term(r"\}"), Ok(Some('}' as u32)));
    assert_eq!(parse_term(r"\0"), Ok(Some('\0' as u32)));
    assert_eq!(parse_term(r"\n"), Ok(Some('\n' as u32)));
    assert_eq!(parse_term(r"\r"), Ok(Some('\r' as u32)));
    assert_eq!(parse_term(r"\t"), Ok(Some('\t' as u32)));
    // \x escape sequences (ASCII only, 0-127)
    assert_eq!(parse_term(r"\x00"), Ok(Some('\0' as u32)));
    assert_eq!(parse_term(r"\x20"), Ok(Some(' ' as u32)));
    assert_eq!(parse_term(r"\x41"), Ok(Some('A' as u32)));
    assert_eq!(parse_term(r"\x61"), Ok(Some('a' as u32)));
    assert_eq!(parse_term(r"\x7F"), Ok(Some('\x7F' as u32)));
    assert_eq!(parse_term(r"\x7f"), Ok(Some('\x7F' as u32)));
    // Test case sensitivity
    assert_eq!(
        parse_term(r"\xFF"),
        err::out_of_range(r"`\xFF`", 0..4, "ASCII range")
    );
    assert_eq!(
        parse_term(r"\x80"),
        err::out_of_range(r"`\x80`", 0..4, "ASCII range")
    );
    // Test invalid hex digits - just check that they return errors
    assert_eq!(
        parse_term(r"\xGH"),
        err::unexpected("GH", 2..4, "two hexadecimal digits")
    );
    assert_eq!(
        parse_term(r"\x1Z"),
        err::unexpected("1Z", 2..4, "two hexadecimal digits")
    );
    // Test incomplete sequences
    assert_eq!(
        parse_term(r"\x["),
        err::unexpected("[", 2..3, "a hexadecimal digit")
    );
    assert_eq!(
        parse_term(r"\x1"),
        err::unexpected("", 3..3, "a hexadecimal digit")
    );
}

#[test]
fn parse_decimal() {
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
        err::out_of_range("1000000000000", 0..13, "`u32` range")
    );
}
