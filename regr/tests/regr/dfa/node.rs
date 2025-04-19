use pretty_assertions::assert_eq;
use regr::dfa::{Graph, Node};
use regr::{Range, range};

#[test]
fn node_id() {
    let graph = Graph::new();
    let node = graph.node();
    assert_eq!(node.id(), 0);
}

#[test]
fn node_copy_clone() {
    let graph = Graph::new();
    let node = graph.node();
    let node_copy = node;
    let node_clone = node.clone();
    assert_eq!(node.id(), node_copy.id());
    assert_eq!(node.id(), node_clone.id());
}

#[test]
fn node_debug_fmt() {
    let graph = Graph::new();
    assert_eq!(format!("{:?}", graph.node()), "node(0)");
    assert_eq!(format!("{:?}", graph.node()), "node(1)");
}

#[test]
fn node_connect() {
    let graph = Graph::new();
    let (a, b, c) = (graph.node(), graph.node(), graph.node());
    a.connect(b, 1);
    a.connect(c, 2);
    // TODO: add check for targets id
}

#[test]
fn node_symbol_targets_iter() {
    let graph = Graph::new();
    let node = graph.node();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    node.connect(a, 0);
    node.connect(a, 1);
    node.connect(b, 2);
    node.connect(c, 255);

    let symbol_target_vec = node
        .symbol_target_pairs()
        .collect::<Vec<(Range, Node<'_>)>>();
    assert_eq!(
        vec![(range(0..=1), a), (range(2), b), (range(255), c)],
        symbol_target_vec
    );
}
