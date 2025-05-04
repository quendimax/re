use pretty_assertions::assert_eq;
use regr::{AutomatonKind, Epsilon, Graph, Range, range};

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
fn graph_display_fmt_0() {
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
        format!("{}", graph).replace("\n", "\n        "),
        "\
        node(0) {
            ['a'-\\xFF] -> node(1)
            Epsilon -> node(1)
        }
        node(1) {
            Epsilon -> node(2)
            Epsilon -> node(0)
        }
        node(2) {
            ['c'] -> node(3)
        }
        node(3) {
            Epsilon -> node(0)
            Epsilon -> node(1)
            Epsilon -> node(2)
        }"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_1() {
    let graph = Graph::nfa();
    let n0 = graph.node();
    let n1 = graph.node();
    let n2 = graph.node();
    let n3 = graph.node();
    let n4 = graph.node();
    n0.connect(n1, range(b'a'..=b'b'));
    n0.connect(n1, range(b'd'..=b'z'));
    n1.connect(n2, Epsilon);
    n1.connect(n4, Epsilon);
    n2.connect(n3, b'a');
    n3.connect(n4, Epsilon);
    n3.connect(n2, Epsilon);
    assert_eq!(
        format!("{}", graph).replace("\n", "\n        "),
        "\
        node(0) {
            ['a'-'b' | 'd'-'z'] -> node(1)
        }
        node(1) {
            Epsilon -> node(2)
            Epsilon -> node(4)
        }
        node(2) {
            ['a'] -> node(3)
        }
        node(3) {
            Epsilon -> node(4)
            Epsilon -> node(2)
        }
        node(4) {}"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_2() {
    let graph = Graph::nfa();
    let n0 = graph.node();
    let n1 = graph.node();
    let n2 = graph.node();
    let n3 = graph.node();
    let n4 = graph.node();
    let n5 = graph.node();
    let n6 = graph.node();
    let n7 = graph.node();
    n0.connect(n2, Epsilon);
    n0.connect(n5, Epsilon);
    n2.connect(n3, b'a');
    n3.connect(n4, b'b');
    n4.connect(n1, Epsilon);
    n5.connect(n6, b'c');
    n6.connect(n7, b'd');
    n7.connect(n1, Epsilon);
    assert_eq!(
        format!("{}", graph).replace("\n", "\n        "),
        "\
        node(0) {
            Epsilon -> node(2)
            Epsilon -> node(5)
        }
        node(1) {}
        node(2) {
            ['a'] -> node(3)
        }
        node(3) {
            ['b'] -> node(4)
        }
        node(4) {
            Epsilon -> node(1)
        }
        node(5) {
            ['c'] -> node(6)
        }
        node(6) {
            ['d'] -> node(7)
        }
        node(7) {
            Epsilon -> node(1)
        }"
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_3() {
    let graph = Graph::dfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    a.connect(b, 1);
    b.connect(b, 3);
    b.connect(c, 1);
    assert_eq!(
        format!("{:?}", graph).replace("\n", "\n        "),
        "\
        node(0) {
            [1] -> node(1)
        }
        node(1) {
            [3] -> self
            [1] -> node(2)
        }
        node(2) {}"
    );
}
