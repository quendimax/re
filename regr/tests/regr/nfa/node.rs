use pretty_assertions::{assert_eq, assert_ne};
use regr::adt::Set;
use regr::{edge, nfa};

type Graph = nfa::Graph<char>;

#[test]
fn node_partial_eq() {
    let graph = Graph::new();
    let node_1 = graph.node();
    assert_ne!(node_1, graph.node());

    let graph_2 = Graph::new();
    let node_2 = graph_2.node();
    assert_eq!(node_1, node_2);
}

#[test]
fn node_connect() {
    let graph = Graph::new();
    let node_a = graph.node();
    let node_b = graph.node();
    let node_c = graph.node();
    node_a.connect(node_b, edge!['a']);
    node_a.connect(node_c, edge!['a']);
    node_a.connect(node_c, edge!['a']);
    node_c.connect_with_epsilon(node_a);
}

#[test]
fn node_eclosure() {
    let arena = Graph::new();
    let a = arena.node();
    let b = arena.node();
    let c = arena.node();
    let d = arena.node();

    a.connect(b, edge!['a', char::MAX]);
    a.connect_with_epsilon(b);
    b.connect_with_epsilon(c);
    c.connect(d, edge!['c']);
    b.connect_with_epsilon(a);
    d.connect_with_epsilon(a);
    d.connect_with_epsilon(b);
    d.connect_with_epsilon(c);

    let set = Set::from_iter(vec![a, b, c]);
    assert_eq!(a.eclosure(), set)
}
