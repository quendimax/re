use assert_matches::assert_matches;
use pretty_assertions::assert_eq;
use regr::{Arena, Graph};
use renc::{Error::*, Utf8Coder};
use resy::Error::*;
use resy::{Error, Parser};

const CODER: Utf8Coder = Utf8Coder;

fn fmt<T: std::fmt::Display + ?Sized>(obj: &T) -> String {
    let mut result = String::new();
    for line in format!("{obj}").split('\n') {
        if !line.ends_with('{') && !line.ends_with('}') {
            result.push_str("    ");
        }
        result.push_str(line.trim());
        result.push('\n');
    }
    result.trim().to_string()
}

fn try_parse(input: &str) -> Result<String, Error> {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let start_node = graph.start_node();
    let mut parser = Parser::new(&graph, CODER);
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
fn parser_new() {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    _ = Parser::new(&graph, CODER);
}

#[test]
#[should_panic(expected = "this parser can build only an NFA graph")]
fn parser_new_panics() {
    let mut arena = Arena::new();
    let graph = Graph::dfa_in(&mut arena);
    _ = Parser::new(&graph, CODER);
}

#[test]
fn parse_close_paren() {
    assert_eq!(
        parse(")"),
        "unexpected close bracket `)` encountered without open one"
    );
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
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn parse_disjunction() {
    assert_eq!(
        parse("a|b|c"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
                ['b'] -> node(2)
                ['c'] -> node(3)
            }
            node(1) {}
            node(2) {
                [Epsilon] -> node(1)
            }
            node(3) {
                [Epsilon] -> node(1)
            }
        ")
    );
    assert_eq!(
        parse("||"),
        fmt("\
            node(0) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(2)
                [Epsilon] -> node(3)
            }
            node(1) {}
            node(2) {
                [Epsilon] -> node(1)
            }
            node(3) {
                [Epsilon] -> node(1)
            }
        ")
    );
}

#[test]
fn parse_concatenation() {
    assert_eq!(parse(""), expect(&["Epsilon"]));
    assert_eq!(parse("ab"), expect(&["'a'", "'b'"]));
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
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
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
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
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
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
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
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
    assert_eq!(
        parse("a*{2}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
                [Epsilon] -> node(2)
            }
            node(1) {
                [Epsilon] -> node(0)
                [Epsilon] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
                [Epsilon] -> node(4)
            }
            node(3) {
                [Epsilon] -> node(2)
                [Epsilon] -> node(4)
            }
            node(4) {}
        ")
    );
    assert_eq!(
        parse("a+{3}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
            }
            node(1) {
                [Epsilon] -> node(0)
                [Epsilon] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                [Epsilon] -> node(2)
                [Epsilon] -> node(4)
            }
            node(4) {
                ['a'] -> node(5)
            }
            node(5) {
                [Epsilon] -> node(4)
                [Epsilon] -> node(6)
            }
            node(6) {}
        ")
    );

    assert_eq!(parse("a{}"), "expected decimal, but got '}'");
    assert_eq!(parse("a{,}"), "expected decimal, but got ','");
    assert_eq!(parse("a{-1}"), "expected decimal, but got '-'");
    assert_eq!(parse("a{0}"), "value 0 doesn't make sense here");
    assert_eq!(
        parse("a{0"),
        "unexpected end of file within braces expression"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn parse_braces_with_two_nums() {
    assert_eq!(parse("a{1,1}"), expect(&["'a'"]));
    assert_eq!(parse("a{2,2}"), expect(&["'a'", "'a'"]));

    assert_eq!(
        parse("a{0,}"),
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
        parse("a{1,}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
            }
            node(1) {
                ['a'] -> node(2)
                [Epsilon] -> node(3)
            }
            node(2) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(3)
            }
            node(3) {}
        ")
    );
    assert_eq!(
        parse("a{2,}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
            }
            node(1) {
                ['a'] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
                [Epsilon] -> node(4)
            }
            node(3) {
                [Epsilon] -> node(2)
                [Epsilon] -> node(4)
            }
            node(4) {}
        ")
    );

    assert_eq!(
        parse("a{0,1}"),
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
        parse("a{1,2}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
            }
            node(1) {
                ['a'] -> node(2)
                [Epsilon] -> node(3)
            }
            node(2) {
                [Epsilon] -> node(3)
            }
            node(3) {}
        ")
    );
    assert_eq!(
        parse("a{1,3}"),
        fmt("\
            node(0) {
                ['a'] -> node(1)
            }
            node(1) {
                ['a'] -> node(2)
                [Epsilon] -> node(3)
            }
            node(2) {
                [Epsilon] -> node(3)
            }
            node(3) {
                ['a'] -> node(4)
                [Epsilon] -> node(5)
            }
            node(4) {
                [Epsilon] -> node(5)
            }
            node(5) {}
        ")
    );

    assert_eq!(parse("a{0,0}"), "value 0 doesn't make sense here");
    assert_eq!(
        parse("a{3,1}"),
        "expected that expression `{n,m}` has invariant `n` <= `m`"
    );
    assert_eq!(parse("a{1,a}"), "expected '}', but got 'a'");
    assert_eq!(parse("a{1a"), "expected '}' or ',', but got 'a'");
}

#[test]
fn parse_escape() {
    assert_eq!(parse("\\\""), expect(&["'\"'"]));
    assert_eq!(parse(r"\."), expect(&["'.'"]));
    assert_eq!(parse(r"\*"), expect(&["'*'"]));
    assert_eq!(parse(r"\+"), expect(&["'+'"]));
    assert_eq!(parse(r"\-"), expect(&["'-'"]));
    assert_eq!(parse(r"\?"), expect(&["'?'"]));
    assert_eq!(parse(r"\\"), expect(&["'\\'"]));
    assert_eq!(parse(r"\|"), expect(&["'|'"]));
    assert_eq!(parse(r"\("), expect(&["'('"]));
    assert_eq!(parse(r"\)"), expect(&["')'"]));
    assert_eq!(parse(r"\["), expect(&["'['"]));
    assert_eq!(parse(r"\]"), expect(&["']'"]));
    assert_eq!(parse(r"\{"), expect(&["'{'"]));
    assert_eq!(parse(r"\}"), expect(&["'}'"]));
    assert_eq!(parse(r"\0"), expect(&["00h"]));
    assert_eq!(parse(r"\n"), expect(&["0Ah"]));
    assert_eq!(parse(r"\r"), expect(&["0Dh"]));
    assert_eq!(parse(r"\t"), expect(&["09h"]));
    assert_eq!(parse(r"\x00"), expect(&["00h"]));
    assert_eq!(parse(r"\x61"), expect(&["'a'"]));
    assert_eq!(parse(r"\x7f"), expect(&["7Fh"]));
    assert_eq!(parse(r"\x7F"), expect(&["7Fh"]));
    assert_eq!(parse(r"\x7F"), expect(&["7Fh"]));
    assert_eq!(parse(r"\u{0}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{00}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{000}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{00000}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{000000}"), expect(&["00h"]));
    assert_eq!(parse(r"\u{10FFFF}"), expect(&["F4h", "8Fh", "BFh", "BFh"]));

    assert_eq!(
        parse(r"\"),
        "unexpected end of file within escape expression"
    );
    assert_eq!(
        parse(r"\x"),
        "unexpected end of file within ascii escape expression"
    );
    assert_eq!(
        parse(r"\u"),
        "unexpected end of file within regular expression"
    );
    assert_eq!(
        parse(r"\u{"),
        "unexpected end of file within unicode escape expression"
    );
}

#[test]
fn parse_escape_fails() {
    assert_eq!(parse(r"\m"), r"escape expression '\m' is not supported");

    assert_matches!(try_parse(r"\x80"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\x"), Err(UnexpectedEof { .. }));
    assert_matches!(try_parse(r"\x0"), Err(UnexpectedEof { .. }));
    assert_matches!(try_parse(r"\x7h"), Err(InvalidHex(..)));
    assert_matches!(try_parse(r"\xqf"), Err(InvalidHex(..)));

    assert_matches!(try_parse(r"\u{}"), Err(EmptyEscape));
    assert_matches!(try_parse(r"\u{s}"), Err(InvalidHex(..)));
    assert_eq!(parse(r"\u{0000000}"), "expected '}', but got '0'");
    assert_matches!(
        try_parse(r"\u{110000}"),
        Err(EncoderError(InvalidCodePoint(..)))
    );
    assert_matches!(
        try_parse(r"\u{D800}"),
        Err(EncoderError(SurrogateUnsupported { .. }))
    );

    assert_matches!(try_parse(r"\x80"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\xFF"), Err(OutOfRange { .. }));
    assert_matches!(try_parse(r"\u{0000000}"), Err(UnexpectedToken { .. }));
    assert_matches!(
        try_parse(r"\u{110000}"),
        Err(EncoderError(InvalidCodePoint(..)))
    );
    assert_matches!(
        try_parse(r"\u{D800}"),
        Err(EncoderError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{DBff}"),
        Err(EncoderError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{DC00}"),
        Err(EncoderError(SurrogateUnsupported { .. }))
    );
    assert_matches!(
        try_parse(r"\u{dFFf}"),
        Err(EncoderError(SurrogateUnsupported { .. }))
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn parse_class() {
    assert_eq!(
        parse("[.]"),
        fmt("\
            node(0) {
                [00h-7Fh] -> node(1)
                [C2h-DFh] -> node(2)
                [E0h] -> node(3)
                [E1h-ECh] -> node(5)
                [EDh] -> node(7)
                [EEh-EFh] -> node(9)
                [F0h] -> node(11)
                [F1h-F3h] -> node(14)
                [F4h] -> node(17)
            }
            node(1) {}
            node(2) {
                [80h-BFh] -> node(1)
            }
            node(3) {
                [A0h-BFh] -> node(4)
            }
            node(4) {
                [80h-BFh] -> node(1)
            }
            node(5) {
                [80h-BFh] -> node(6)
            }
            node(6) {
                [80h-BFh] -> node(1)
            }
            node(7) {
                [80h-9Fh] -> node(8)
            }
            node(8) {
                [80h-BFh] -> node(1)
            }
            node(9) {
                [80h-BFh] -> node(10)
            }
            node(10) {
                [80h-BFh] -> node(1)
            }
            node(11) {
                [90h-BFh] -> node(12)
            }
            node(12) {
                [80h-BFh] -> node(13)
            }
            node(13) {
                [80h-BFh] -> node(1)
            }
            node(14) {
                [80h-BFh] -> node(15)
            }
            node(15) {
                [80h-BFh] -> node(16)
            }
            node(16) {
                [80h-BFh] -> node(1)
            }
            node(17) {
                [80h-8Fh] -> node(18)
            }
            node(18) {
                [80h-BFh] -> node(19)
            }
            node(19) {
                [80h-BFh] -> node(1)
            }
        ")
    );
    assert_eq!(
        parse("[a-z0-8]"),
        fmt("\
            node(0) {
                ['0'-'8' | 'a'-'z'] -> node(1)
            }
            node(1) {}
        ")
    );
    assert_eq!(
        parse("[a-bdf0-8]"),
        fmt("\
            node(0) {
                ['0'-'8' | 'a'-'b' | 'd' | 'f'] -> node(1)
            }
            node(1) {}
        ")
    );
    assert_eq!(
        parse("[\\n][\\r\\0]"),
        fmt("\
            node(0) {
                [0Ah] -> node(1)
            }
            node(1) {
                [00h | 0Dh] -> node(2)
            }
            node(2) {}
        ")
    );
    assert_eq!(parse("[]"), "empty class expression `[]` is not supported");
    assert_eq!(
        parse("["),
        "unexpected end of file within regular expression"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn parse_dot_class() {
    assert_eq!(
        parse("."),
        fmt("\
            node(0) {
                [00h-7Fh] -> node(1)
                [C2h-DFh] -> node(2)
                [E0h] -> node(3)
                [E1h-ECh] -> node(5)
                [EDh] -> node(7)
                [EEh-EFh] -> node(9)
                [F0h] -> node(11)
                [F1h-F3h] -> node(14)
                [F4h] -> node(17)
            }
            node(1) {}
            node(2) {
                [80h-BFh] -> node(1)
            }
            node(3) {
                [A0h-BFh] -> node(4)
            }
            node(4) {
                [80h-BFh] -> node(1)
            }
            node(5) {
                [80h-BFh] -> node(6)
            }
            node(6) {
                [80h-BFh] -> node(1)
            }
            node(7) {
                [80h-9Fh] -> node(8)
            }
            node(8) {
                [80h-BFh] -> node(1)
            }
            node(9) {
                [80h-BFh] -> node(10)
            }
            node(10) {
                [80h-BFh] -> node(1)
            }
            node(11) {
                [90h-BFh] -> node(12)
            }
            node(12) {
                [80h-BFh] -> node(13)
            }
            node(13) {
                [80h-BFh] -> node(1)
            }
            node(14) {
                [80h-BFh] -> node(15)
            }
            node(15) {
                [80h-BFh] -> node(16)
            }
            node(16) {
                [80h-BFh] -> node(1)
            }
            node(17) {
                [80h-8Fh] -> node(18)
            }
            node(18) {
                [80h-BFh] -> node(19)
            }
            node(19) {
                [80h-BFh] -> node(1)
            }
        ")
    );
}

#[test]
fn parse_char() {
    assert_eq!(parse("\""), expect(&["'\"'"]));
    assert_eq!(parse("a"), expect(&["'a'"]));
    assert_eq!(parse("ў"), expect(&["D1h", "9Eh"]));
    assert_eq!(parse("Ⲁ"), expect(&["E2h", "B2h", "80h"]));
    assert_eq!(parse("𐌰"), expect(&["F0h", "90h", "8Ch", "B0h"]));

    assert_eq!(
        parse("[*]"),
        "character `*` must be escaped with a prior backslash `\\`"
    );
}
