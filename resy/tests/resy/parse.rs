use assert_matches::assert_matches;
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
    try_parse(input).unwrap_or_else(|e| e.to_string())
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
fn parse_parens() {
    assert_eq!(parse("()"), expect(&["Epsilon"]));
    assert_eq!(parse("((((()))))"), expect(&["Epsilon"]));

    assert_matches!(try_parse(")"), Err(UnexpcetedCloseBracket(..)));
    assert_matches!(try_parse("))"), Err(UnexpcetedCloseBracket(..)));
    assert_matches!(try_parse("((())))"), Err(UnexpcetedCloseBracket(..)));
}

#[test]
fn parse_escape() {
    assert_eq!(parse(""), expect(&["Epsilon"]));
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
        Err(Error::UnexpectedToken { gotten, expected }) if gotten == "0" && expected == "}"
    );
    assert_matches!(
        try_parse(r"\u{110000}"),
        Err(CodecError(InvalidCodePoint(..)))
    );
    assert_matches!(
        try_parse(r"\u{D800}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );

    assert_matches!(try_parse(r"\x80"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\xFF"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\u{0000000}"), Err(UnexpectedToken { .. }));
    assert_matches!(
        try_parse(r"\u{110000}"),
        Err(CodecError(InvalidCodePoint(..)))
    );
    assert_matches!(
        try_parse(r"\u{D800}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{DBff}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{DC00}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{dFFf}"),
        Err(CodecError(SurrogateUnsupported { .. }))
    );
}

#[test]
fn parse_kleene_star() {
    assert_eq!(
        parse("a*"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
            [Epsilon] -> node(2)
        }
        node(1) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(2)
        }
        node(2) {}
        ")
    );
    assert_eq!(
        parse("a**"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
            [Epsilon] -> node(2)
            [Epsilon] -> node(3)
        }
        node(1) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(2)
        }
        node(2) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(3)
        }
        node(3) {}
        ")
    );
}

#[test]
fn parse_plus_operator() {
    assert_eq!(
        parse("a+"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
        }
        node(1) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(2)
        }
        node(2) {}
        ")
    );
    assert_eq!(
        parse("a++"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
        }
        node(1) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(2)
        }
        node(2) {
            [Epsilon] -> node(0)
            [Epsilon] -> node(3)
        }
        node(3) {}
        ")
    );
}

#[test]
fn parse_question() {
    assert_eq!(
        parse("a?"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
            [Epsilon] -> node(2)
        }
        node(1) {
            [Epsilon] -> node(2)
        }
        node(2) {}
        ")
    );
    assert_eq!(
        parse("a??"),
        fmt("\
        node(0) {
            ['a'] -> node(1)
            [Epsilon] -> node(2)
            [Epsilon] -> node(3)
        }
        node(1) {
            [Epsilon] -> node(2)
        }
        node(2) {
            [Epsilon] -> node(3)
        }
        node(3) {}
        ")
    );
}

#[test]
fn parse_braces_with_one_num() {
    assert_eq!(parse("a{1}"), expect(&["'a'"]));
    assert_eq!(parse("a{2}"), expect(&["'a'", "'a'"]));
    assert_eq!(parse("a{3}"), expect(&["'a'", "'a'", "'a'"]));

    assert_eq!(parse("ў{1}"), expect(&["D1h", "9Eh"]));
    assert_eq!(parse("ў{2}"), expect(&["D1h", "9Eh", "D1h", "9Eh"]));
    assert_eq!(
        parse("ў{3}"),
        expect(&["D1h", "9Eh", "D1h", "9Eh", "D1h", "9Eh"])
    );

    assert_eq!(
        parse("a*{1}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
                [Epsilon] -> node(2)
            }
            node(1) {
                [Epsilon] -> node(0)
                [Epsilon] -> node(2)
            }
            node(2) {}
        ")
    );
    // assert_eq!(
    //     parse("a*{2}"),
    //     fmt("\
    //         node(0) {
    //             ['a'] -> node(1)
    //             Epsilon -> node(2)
    //         }
    //         node(1) {
    //             Epsilon -> node(0)
    //             Epsilon -> node(2)
    //         }
    //         node(2) {
    //             ['a'] -> node(3)
    //             Epsilon -> node(4)
    //         }
    //         node(3) {
    //             Epsilon -> node(2)
    //             Epsilon -> node(4)
    //         }
    //         node(4) {}
    //     ")
    // );

    assert_eq!(parse("a{}"), "expected <decimal>, but got nothing");
    assert_eq!(parse("a{-1}"), "expected <decimal>, but got nothing");
    assert_eq!(parse("a{0}"), "value 0 doesn't make sense here");
}

#[test]
fn parse_braces_with_two_nums() {
    // assert_eq!(parse("a{1,}"), expect(&["'a'"]));
}

#[test]
fn parse_char() {
    assert_eq!(parse("ab"), expect(&["'a'", "'b'"]));
    assert_eq!(parse("ў"), expect(&["D1h", "9Eh"]));
    assert_eq!(parse("Ⲁ"), expect(&["E2h", "B2h", "80h"]));
    assert_eq!(parse("𐌰"), expect(&["F0h", "90h", "8Ch", "B0h"]));
}
