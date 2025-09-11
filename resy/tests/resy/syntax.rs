use pretty_assertions::assert_eq;
use renc::Utf8Encoder;
use resy::Parser;

#[test]
fn parser_parse() {
    let parse = |pattern: &str| {
        let parser = Parser::new(Utf8Encoder::new());
        match parser.parse(pattern) {
            Ok(hir) => hir.to_string(),
            Err(err) => err.to_string(),
        }
    };

    assert_eq!(parse("asdf|dfgh"), r#""asdf" | "dfgh""#);
    assert_eq!(parse("(asdf)|(?<1>dfgh)"), r#""asdf" | (?<1> "dfgh" )"#);
    assert_eq!(parse("[sdf]"), r#"['d'] | ['f'] | ['s']"#);

    assert_eq!(parse("asd\\f"), "unsupported escape sequence `\\f`");
    assert_eq!(parse("(abc))"), "expected `EOF`, but found `)`");
    assert_eq!(parse("[asd\\f]"), "unsupported escape sequence `\\f`");
    assert_eq!(
        parse(r"\u{D800}"),
        "encoder error: surrogate code point D800h is not supported by UTF-8"
    );
}
