use crate::hir::Hir;
use crate::str::error::err;
use crate::str::lexis::Lexer;
use crate::str::syntax::ParserImpl;
use pretty_assertions::assert_eq;
use renc::Utf8Encoder;

#[test]
fn parse_concat() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.parse_concat()
    };
    assert_eq!(parse("asdgўsdf"), Ok(Hir::literal("asdgўsdf")));
    assert_eq!(
        parse("вжух+ыs"),
        Ok(Hir::concat(vec![
            Hir::literal("вжу"),
            Hir::repeat(Hir::literal("х"), 1, None),
            Hir::literal("ыs"),
        ]))
    );
    assert_eq!(parse("a{"), err::unexpected("", 2..2, "a decimal number"));
}

#[test]
fn parse_item() {
    let lexer = Lexer::new("aўў");
    let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
    let mut parse = || parser.try_parse_item();

    assert_eq!(parse(), Ok(Some(Hir::literal("a"))));
    assert_eq!(parse(), Ok(Some(Hir::literal("ў"))));
    assert_eq!(parse(), Ok(Some(Hir::literal([209, 158]))));
    assert_eq!(parse(), Ok(None));
}

#[test]
fn parse_postfix() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.try_parse_postfix()
    };
    assert_eq!(parse("*"), Ok(Some((0, None))));
    assert_eq!(parse("{0,}"), Ok(Some((0, None))));
    assert_eq!(parse("+"), Ok(Some((1, None))));
    assert_eq!(parse("{1,}"), Ok(Some((1, None))));
    assert_eq!(parse("?"), Ok(Some((0, Some(1)))));
    assert_eq!(parse("{0,1}"), Ok(Some((0, Some(1)))));
    assert_eq!(parse("."), Ok(None));
    assert_eq!(parse("{}"), err::unexpected("}", 1..2, "a decimal number"));
}

#[test]
fn parse_braces() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.parse_braces()
    };
    assert_eq!(parse("."), err::unexpected(".", 0..1, "`{`"));
    assert_eq!(
        parse("{1000000000000000000000"),
        err::out_of_range("1000000000000000000000", 1..23, "allowed range")
    );
    assert_eq!(
        parse("{1,1000000000000000000000}"),
        err::out_of_range("1000000000000000000000", 3..25, "allowed range")
    );
    assert_eq!(parse("{}"), err::unexpected("}", 1..2, "a decimal number"));
    assert_eq!(parse("{,}"), err::unexpected(",", 1..2, "a decimal number"));
    assert_eq!(parse("{0,s}"), err::unexpected("s", 3..4, "`}`"));
    assert_eq!(
        parse("{0s}"),
        err::unexpected("s", 2..3, "either `}` or `,`")
    );
    assert_eq!(parse("{0}"), err::zero_repetition(0..3));
    assert_eq!(parse("{0,0}"), err::zero_repetition(0..5));
    assert_eq!(parse("{3,0}"), err::invalid_repetition(0..5));
}

#[test]
fn parse_dot() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.parse_dot()
    };
    let Ok(hir) = parse(".") else {
        panic!("Failed to parse dot pattern");
    };
    assert!(hir.is_disjunct());
    assert_eq!(
        hir.to_string(),
        concat!(
            "[00h-7Fh] | ",
            "([C2h-DFh] & [80h-BFh]) | ",
            "([E0h] & [A0h-BFh] & [80h-BFh]) | ",
            "([E1h-ECh] & [80h-BFh] & [80h-BFh]) | ",
            "([EDh] & [80h-9Fh] & [80h-BFh]) | ",
            "([EEh-EFh] & [80h-BFh] & [80h-BFh]) | ",
            "([F0h] & [90h-BFh] & [80h-BFh] & [80h-BFh]) | ",
            "([F1h-F3h] & [80h-BFh] & [80h-BFh] & [80h-BFh]) | ",
            "([F4h] & [80h-8Fh] & [80h-BFh] & [80h-BFh])"
        )
    );
}

#[test]
fn parse_ascii_escape() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.try_parse_term()
    };
    assert_eq!(parse("a"), Ok(Some('a' as u32)));
    assert_eq!(parse("/"), Ok(Some('/' as u32)));
    assert_eq!(parse(r"\\"), Ok(Some('\\' as u32)));
    assert_eq!(parse(r"\."), Ok(Some('.' as u32)));
    assert_eq!(parse(r"\*"), Ok(Some('*' as u32)));
    assert_eq!(parse(r"\+"), Ok(Some('+' as u32)));
    assert_eq!(parse(r"\-"), Ok(Some('-' as u32)));
    assert_eq!(parse(r"\?"), Ok(Some('?' as u32)));
    assert_eq!(parse(r"\|"), Ok(Some('|' as u32)));
    assert_eq!(parse(r"\("), Ok(Some('(' as u32)));
    assert_eq!(parse(r"\)"), Ok(Some(')' as u32)));
    assert_eq!(parse(r"\["), Ok(Some('[' as u32)));
    assert_eq!(parse(r"\]"), Ok(Some(']' as u32)));
    assert_eq!(parse(r"\{"), Ok(Some('{' as u32)));
    assert_eq!(parse(r"\}"), Ok(Some('}' as u32)));
    assert_eq!(parse(r"\0"), Ok(Some('\0' as u32)));
    assert_eq!(parse(r"\n"), Ok(Some('\n' as u32)));
    assert_eq!(parse(r"\r"), Ok(Some('\r' as u32)));
    assert_eq!(parse(r"\t"), Ok(Some('\t' as u32)));
    // Unsupported escape sequences
    assert_eq!(parse(r"\a"), err::unsupported_escape(r"\a", 0..2));
    assert_eq!(parse(r"\U"), err::unsupported_escape(r"\U", 0..2));
    assert_eq!(parse(r"\Ў"), err::unsupported_escape(r"\Ў", 0..3));

    // Parsing special characters should be skipped
    assert_eq!(parse("\\"), Ok(None));
    assert_eq!(parse("."), Ok(None));
    assert_eq!(parse("*"), Ok(None));
    assert_eq!(parse("+"), Ok(None));
    assert_eq!(parse("-"), Ok(None));
    assert_eq!(parse("?"), Ok(None));
    assert_eq!(parse("|"), Ok(None));
    assert_eq!(parse("("), Ok(None));
    assert_eq!(parse(")"), Ok(None));
    assert_eq!(parse("["), Ok(None));
    assert_eq!(parse("]"), Ok(None));
    assert_eq!(parse("{"), Ok(None));
    assert_eq!(parse("}"), Ok(None));

    // \x escape sequences (ASCII only, 0-127)
    assert_eq!(parse(r"\x00"), Ok(Some('\0' as u32)));
    assert_eq!(parse(r"\x20"), Ok(Some(' ' as u32)));
    assert_eq!(parse(r"\x41"), Ok(Some('A' as u32)));
    assert_eq!(parse(r"\x61"), Ok(Some('a' as u32)));
    assert_eq!(parse(r"\x7F"), Ok(Some('\x7F' as u32)));
    assert_eq!(parse(r"\x7f"), Ok(Some('\x7F' as u32)));
    // Test case sensitivity
    assert_eq!(
        parse(r"\xFF"),
        err::out_of_range(r"`\xFF`", 0..4, "ASCII range")
    );
    assert_eq!(
        parse(r"\x80"),
        err::out_of_range(r"`\x80`", 0..4, "ASCII range")
    );
    // Test invalid hex digits - just check that they return errors
    assert_eq!(
        parse(r"\xGH"),
        err::unexpected("GH", 2..4, "two hexadecimal digits")
    );
    assert_eq!(
        parse(r"\x1Z"),
        err::unexpected("1Z", 2..4, "two hexadecimal digits")
    );
    // Test incomplete sequences
    assert_eq!(
        parse(r"\x["),
        err::unexpected("[", 2..3, "a hexadecimal digit")
    );
    assert_eq!(
        parse(r"\x1"),
        err::unexpected("", 3..3, "a hexadecimal digit")
    );
}

#[test]
fn parse_unicode_escape() {
    let parse = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.try_parse_term()
    };

    // \u{...} escape sequences (Unicode)
    // Basic ASCII characters
    assert_eq!(parse(r"\u{0}"), Ok(Some(0x0)));
    assert_eq!(parse(r"\u{41}"), Ok(Some('A' as u32)));
    assert_eq!(parse(r"\u{61}"), Ok(Some('a' as u32)));
    assert_eq!(parse(r"\u{7F}"), Ok(Some(0x7F)));

    // Multi-digit hex values
    assert_eq!(parse(r"\u{20}"), Ok(Some(' ' as u32)));
    assert_eq!(parse(r"\u{1F4}"), Ok(Some(0x1F4)));
    assert_eq!(parse(r"\u{1234}"), Ok(Some(0x1234)));
    assert_eq!(parse(r"\u{12345}"), Ok(Some(0x12345)));
    assert_eq!(parse(r"\u{123456}"), Ok(Some(0x123456)));

    // Case insensitive hex digits
    assert_eq!(parse(r"\u{aB}"), Ok(Some(0xAB)));
    assert_eq!(parse(r"\u{Cd}"), Ok(Some(0xCD)));
    assert_eq!(parse(r"\u{EF}"), Ok(Some(0xEF)));
    assert_eq!(parse(r"\u{abcdef}"), Ok(Some(0xABCDEF)));

    // Unicode characters
    assert_eq!(parse(r"\u{A9}"), Ok(Some('©' as u32))); // Copyright symbol
    assert_eq!(parse(r"\u{1F600}"), Ok(Some(0x1F600))); // Emoji
    assert_eq!(parse(r"\u{10FFFF}"), Ok(Some(0x10FFFF))); // Max Unicode

    // Empty escape sequence
    assert_eq!(parse(r"\u{}"), err::empty_escape(0..4));

    // Invalid hex digits
    assert_eq!(
        parse(r"\u{G}"),
        err::unexpected("G", 3..4, "either a hexadecimal digit or a closing brace")
    );
    assert_eq!(
        parse(r"\u{1Z}"),
        err::unexpected("Z", 4..5, "either a hexadecimal digit or a closing brace")
    );
    assert_eq!(
        parse(r"\u{XYZ}"),
        err::unexpected("X", 3..4, "either a hexadecimal digit or a closing brace")
    );
    // Missing opening brace
    assert_eq!(parse(r"\u10"), err::unexpected("1", 2..3, "`{`"));
    // Missing closing brace
    assert_eq!(
        parse(r"\u{123"),
        err::unexpected("", 6..6, "either a hexadecimal digit or a closing brace")
    );
    assert_eq!(parse(r"\u{10ffff"), err::unexpected("", 9..9, "`}`"));
}

#[test]
fn parse_decimal() {
    let parse_decimal = |pattern: &str| {
        let lexer = Lexer::new(pattern);
        let mut parser = ParserImpl::<Utf8Encoder, true>::new(lexer, &Utf8Encoder);
        parser.try_parse_decimal()
    };
    assert_eq!(parse_decimal("-1"), Ok(None));
    assert_eq!(parse_decimal("0"), Ok(Some(0)));
    assert_eq!(parse_decimal("000"), Ok(Some(0)));
    assert_eq!(parse_decimal("123"), Ok(Some(123)));
    assert_eq!(parse_decimal("1000000"), Ok(Some(1000000)));
    assert_eq!(
        parse_decimal("100000000000000000000"),
        err::out_of_range("100000000000000000000", 0..21, "allowed range")
    );
}
