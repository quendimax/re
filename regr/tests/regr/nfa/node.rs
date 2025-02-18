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
    let graph = Graph::new();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

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

#[test]
fn node_symbol_targets() {
    let graph = Graph::new();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, edge!['a', char::MAX]);
    a.connect_with_epsilon(b);
    b.connect_with_epsilon(c);
    c.connect(d, edge!['c']);
    b.connect_with_epsilon(a);
    d.connect_with_epsilon(a);
    d.connect_with_epsilon(b);
    d.connect_with_epsilon(c);

    assert_eq!(a.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![b]);
    assert_eq!(c.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![d]);
}

#[test]
fn node_epsilon_targets() {
    let graph = Graph::new();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, edge!['a', char::MAX]);
    a.connect_with_epsilon(b);
    b.connect_with_epsilon(c);
    c.connect(d, edge!['c']);
    b.connect_with_epsilon(a);
    d.connect_with_epsilon(a);
    d.connect_with_epsilon(b);
    d.connect_with_epsilon(c);

    assert_eq!(a.epsilon_targets().collect::<Vec<_>>(), vec![b]);
}

#[test]
fn node_fmt_debug() {
    let graph = Graph::new();
    let a = graph.node();
    let b = graph.node();
    assert_eq!(format!("{:?}", a), "node(0)");
    assert_eq!(format!("{:?}", b), "node(1)");
}
