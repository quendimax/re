use ntest::assert_panics;
use pretty_assertions::assert_eq;
use regr::Graph;
use resy::{Parser, Utf8Codec};

const CODEC: Utf8Codec = Utf8Codec;

fn dsp<T: std::fmt::Display + ?Sized>(obj: &T) -> String {
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

fn gotten(input: &str) -> String {
    let graph = Graph::nfa();
    let start_node = graph.start_node();
    let mut parser = Parser::new(&graph, CODEC);
    parser.parse(input, start_node).unwrap();
    format!("{graph}")
}

fn expect(chars: &[&str]) -> String {
    let mut res = "".to_string();
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
    dsp(&res)
}

#[test]
fn parse_escape() {
    assert_eq!(gotten("\""), expect(&["'\"'"]));
    assert_eq!(gotten(r"\n"), expect(&["\\x0A"]));
    assert_eq!(gotten(r"\r"), expect(&["\\x0D"]));
    assert_eq!(gotten(r"\t"), expect(&["\\x09"]));
    assert_eq!(gotten(r"\0"), expect(&["\\x00"]));
    assert_eq!(gotten(r"\x00"), expect(&["\\x00"]));
    assert_eq!(gotten(r"\x61"), expect(&["'a'"]));
    assert_eq!(gotten(r"\x7f"), expect(&["\\x7F"]));
    assert_eq!(gotten(r"\x7F"), expect(&["\\x7F"]));
    assert_eq!(gotten(r"\x7F"), expect(&["\\x7F"]));
    assert_eq!(gotten(r"\u{0}"), expect(&["\\x00"]));
    assert_eq!(gotten(r"\u{000000}"), expect(&["\\x00"]));
    assert_eq!(
        gotten(r"\u{10FFFF}"),
        expect(&["\\xF4", "\\x8F", "\\xBF", "\\xBF"])
    );
}

#[test]
fn parse_ascii_escape_panic() {
    assert_panics!({ gotten(r"\x80") });
    assert_panics!({ gotten(r"\xFF") });
}

#[test]
fn parse_unicode_escape_panic() {
    assert_panics!({ gotten(r"\u{0000000}") });
    assert_panics!({ gotten(r"\u{110000}") });
    assert_panics!({ gotten(r"\u{D800}") });
    assert_panics!({ gotten(r"\u{DBff}") });
    assert_panics!({ gotten(r"\u{DC00}") });
    assert_panics!({ gotten(r"\u{dFFf}") });
}

#[test]
fn parse_char() {
    assert_eq!(
        gotten("ab"),
        dsp("\
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
        gotten(r"ў"),
        dsp("\
        node(0) {
            [\\xD1] -> node(1)
        }
        node(1) {
            [\\x9E] -> node(2)
        }
        node(2) {}
        ")
    );
}
