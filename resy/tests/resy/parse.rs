use assert_matches::assert_matches;
use ntest::assert_panics;
use pretty_assertions::assert_eq;
use regr::Graph;
use resy::CodecError::*;
use resy::Error::*;
use resy::{Error, Parser, Utf8Codec};

const CODEC: Utf8Codec = Utf8Codec;

fn fmt<T: std::fmt::Display + ?Sized>(obj: &T) -> String {
    let mut result = String::new();
    for line in format!("{}", obj).split('\n') {
        if !line.ends_with('{') && !line.ends_with('}') {
            result.push_str("    ");
        }
        result.push_str(line.trim());
        result.push('\n');
    }
    result.trim().to_string()
}

fn try_parse(input: &str) -> Result<String, Error> {
    let graph = Graph::nfa();
    let start_node = graph.start_node();
    let mut parser = Parser::new(&graph, CODEC);
    parser.parse(input, start_node)?;
    Ok(format!("{graph}"))
}

fn parse(input: &str) -> String {
    try_parse(input).unwrap()
}

fn expect(chars: &[&str]) -> String {
    let mut res = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let j = i + 1;
        res += &format!(
            "\
            node({i}) {{
                [{c}] -> node({j})
            }}
            "
        );
    }
    res += &format!("node({}) {{}}", chars.len());
    fmt(&res)
}

#[test]
fn parse_escape() {
    assert_eq!(parse("\""), expect(&["'\"'"]));
    assert_eq!(parse(r"\n"), expect(&["0Ah"]));
    assert_eq!(parse(r"\r"), expect(&["0Dh"]));
    assert_eq!(parse(r"\t"), expect(&["09h"]));
    assert_eq!(parse(r"\0"), expect(&["00h"]));
    assert_eq!(parse(r"\x00"), expect(&["00h"]));
    assert_eq!(parse(r"\x61"), expect(&["'a'"]));
    assert_eq!(parse(r"\x7f"), expect(&["7Fh"]));
    assert_eq!(parse(r"\x7F"), expect(&["7Fh"]));
    assert_eq!(parse(r"\x7F"), expect(&["7Fh"]));
    assert_eq!(parse(r"\u{0}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{000000}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{10FFFF}"), expect(&["F4h", "8Fh", "BFh", "BFh"]));

    assert_panics!({ parse(r"\x80") });
    assert_panics!({ parse(r"\xFF") });
    assert_panics!({ parse(r"\u{0000000}") });
    assert_panics!({ parse(r"\u{110000}") });
    assert_panics!({ parse(r"\u{D800}") });
    assert_panics!({ parse(r"\u{DBff}") });
    assert_panics!({ parse(r"\u{DC00}") });
    assert_panics!({ parse(r"\u{dFFf}") });
}

#[test]
fn parse_escape_fails() {
    assert_matches!(try_parse(r"\x80"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\x"), Err(UnexpectedEof { .. }));
    assert_matches!(try_parse(r"\x0"), Err(UnexpectedEof { .. }));
    assert_matches!(try_parse(r"\x7h"), Err(InvalidHex(..)));
    assert_matches!(try_parse(r"\xqf"), Err(InvalidHex(..)));

    assert_matches!(try_parse(r"\u{}"), Err(EmptyEscape));
    assert_matches!(try_parse(r"\u{s}"), Err(InvalidHex(..)));
    assert_matches!(
        try_parse(r"\u{0000000}"),
        Err(Error::UnexpectedToken { unexpected, expected }) if unexpected == "0" && expected == "}"
    );
    assert_matches!(
        try_parse(r"\u{110000}"),
        Err(CodecError(InvalidCodePoint(..)))
    );
    assert_matches!(
        try_parse(r"\u{D800}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );
}

#[test]
fn parse_char() {
    assert_eq!(
        parse("ab"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
        }
        node(1) {
            ['b'] -> node(2)
        }
        node(2) {}
        ")
    );

    assert_eq!(
        parse(r"ў"),
        fmt("\
        node(0) {
            [D1h] -> node(1)
        }
        node(1) {
            [9Eh] -> node(2)
        }
        node(2) {}
        ")
    );
}
