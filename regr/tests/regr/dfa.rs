use pretty_assertions::{assert_eq, assert_ne};
use regr::{edge, dfa, Range};

type Graph = dfa::Graph<char>;

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
    let a = graph.node();
    a.connect(graph.node(), Range::from('a'..='b'));
    a.connect_with_range(graph.node(), 'd');
    a.connect_with_range(graph.node(), 'e'..='f');
}

#[test]
#[should_panic]
fn node_connect_panic() {
    let graph = Graph::new();
    let node_a = graph.node();
    let node_b = graph.node();
    node_a.connect(node_b, edge!['a']);
    node_a.connect_with_range(node_b, 'a'..='c');
}
