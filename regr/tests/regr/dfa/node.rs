use regr::dfa::{Node, Graph};
use regr::edge;
use pretty_assertions::assert_eq;

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
fn node_targets_iter() {
    let graph = Graph::new();
    let node = graph.node();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    node.connect(a, 0);
    node.connect(b, 1);
    node.connect(c, 255);

    assert_eq!(vec![(0, a), (1, b), (255, c)], node.targets().collect::<Vec<(u8, Node<'_>)>>());
}
