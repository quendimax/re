use pretty_assertions::{assert_eq, assert_ne};
// use regr::adt::Set;
use regr::{Epsilon, Graph, NodeId, Range};

#[test]
fn node_id() {
    let a: u32 = 1;
    let b: NodeId = 2;
    assert_eq!(a + b, 3);
}

#[test]
fn node_partial_eq() {
    let graph = Graph::nfa();
    let node_1 = graph.node();
    assert_ne!(node_1, graph.node());

    let graph_2 = Graph::nfa();
    let node_2 = graph_2.node();
    assert_eq!(node_1, node_2);
}

#[test]
fn node_connect() {
    let graph = Graph::nfa();
    let node_a = graph.node();
    let node_b = graph.node();
    let node_c = graph.node();
    node_a.connect(node_b, b'a');
    node_a.connect(node_c, b'a');
    node_a.connect(node_c, b'a');
    node_c.connect(node_a, Epsilon);
}

// #[test]
// fn node_eclosure() {
//     let graph = Graph::nfa();
//     let a = graph.node();
//     let b = graph.node();
//     let c = graph.node();
//     let d = graph.node();

//     a.connect(b, Range::new(b'a', u8::MAX));
//     a.connect(b, Epsilon);
//     b.connect(c, Epsilon);
//     c.connect(d, b'c');
//     b.connect(a, Epsilon);
//     d.connect(a, Epsilon);
//     d.connect(b, Epsilon);
//     d.connect(c, Epsilon);

//     #[allow(clippy::mutable_key_type)]
//     let set = Set::from_iter(vec![a, b, c]);
//     assert_eq!(a.eclosure(), set)
// }

#[test]
fn node_symbol_targets() {
    let graph = Graph::nfa();
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

    assert_eq!(a.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![b]);
    assert_eq!(c.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![d]);
}

#[test]
fn node_epsilon_targets() {
    let graph = Graph::nfa();
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

    assert_eq!(a.epsilon_targets().collect::<Vec<_>>(), vec![b]);
}

#[test]
fn node_fmt_debug() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    assert_eq!(format!("{:?}", a), "node(0)");
    assert_eq!(format!("{:?}", b), "node(1)");
}
