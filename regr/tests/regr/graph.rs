use pretty_assertions::assert_eq;
use regr::{AutomatonKind, Epsilon, Graph, Range, Translator};

#[test]
fn graph_ctor() {
    _ = Graph::nfa();
    _ = Graph::default();
    _ = Graph::with_capacity(150, AutomatonKind::NFA);
}

#[test]
fn graph_node() {
    let graph = Graph::nfa();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
    let graph = Graph::with_capacity(9, AutomatonKind::NFA);
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
}

#[test]
fn graph_start_node() {
    let graph = Graph::nfa();
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);

    let graph = Graph::nfa();
    assert_eq!(graph.node(), graph.start_node());
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
    assert_eq!(graph.start_node().id(), 0);
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_fmt_debug() {
    let graph = Graph::with_capacity(1, AutomatonKind::NFA);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, Range::new(b'a', u8::MAX));
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    c.connect(d, b'c');
    b.connect(a, Epsilon);
    d.connect(a, Epsilon);
    d.connect(b, Epsilon);
    d.connect(c, Epsilon);
    assert_eq!(
        format!("{:?}", graph).replace("\n", "\n        "),
        "\
        node 0 {
            ['a'-255] -> node 1
            [EPSILON] -> node 1
        }
        node 1 {
            [EPSILON] -> node 2
            [EPSILON] -> node 0
        }
        node 2 {
            ['c'] -> node 3
        }
        node 3 {
            [EPSILON] -> node 0
            [EPSILON] -> node 1
            [EPSILON] -> node 2
        }"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_fmt_debug_1() {
    let graph = Graph::nfa();
    let translator = Translator::new(&graph);
    let hir = regex_syntax::parse("[abd-z]a*").unwrap();
    translator.from_hir_to_nfa(&hir).unwrap();
    assert_eq!(
        format!("{:?}", graph).replace("\n", "\n        "),
        "\
        node 0 {
            ['a'-'b']['d'-'z'] -> node 1
        }
        node 1 {
            [EPSILON] -> node 2
            [EPSILON] -> node 4
        }
        node 2 {
            ['a'] -> node 3
        }
        node 3 {
            [EPSILON] -> node 4
            [EPSILON] -> node 2
        }
        node 4 {}"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_fmt_debug_2() {
    let graph = Graph::nfa();
    let translator = Translator::new(&graph);
    let hir = regex_syntax::parse("ab|cd").unwrap();
    translator.from_hir_to_nfa(&hir).unwrap();
    assert_eq!(
        format!("{:?}", graph).replace("\n", "\n        "),
        "\
        node 0 {
            [EPSILON] -> node 2
            [EPSILON] -> node 5
        }
        node 1 {}
        node 2 {
            ['a'] -> node 3
        }
        node 3 {
            ['b'] -> node 4
        }
        node 4 {
            [EPSILON] -> node 1
        }
        node 5 {
            ['c'] -> node 6
        }
        node 6 {
            ['d'] -> node 7
        }
        node 7 {
            [EPSILON] -> node 1
        }"
    );
}
