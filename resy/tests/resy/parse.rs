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

fn parse(input: &str) -> Graph {
    let graph = Graph::nfa();
    let start_node = graph.start_node();
    let mut parser = Parser::new(&graph, CODEC);
    parser.parse(input, start_node).unwrap();
    graph
}

#[test]
fn parse_escape() {
    let nfa = parse(r"\\");
    assert_eq!(
        format!("{nfa}"),
        dsp("\
        node(0) {
            ['\\'] -> node(1)
        }
        node(1) {}
        ")
    );
}

#[test]
fn parse_char() {
    let nfa = parse(r"ab");
    assert_eq!(
        format!("{nfa}"),
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

    // let nfa = parse(r"ў");
    // assert_eq!(
    //     format!("{nfa}"),
    //     dsp("\
    //     node(0) {
    //         ['ў'] -> node(1)
    //     }
    //     node(1) {}
    //     ")
    // );
}
