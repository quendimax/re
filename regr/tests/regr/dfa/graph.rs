use regr::dfa::Graph;
use pretty_assertions::assert_eq;

#[test]
fn graph_ctor() {
    _ = Graph::new();
    _ = Graph::default();
    _ = Graph::with_capacity(8);
}

#[test]
fn graph_node() {
    let graph = Graph::new();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
}

#[test]
fn graph_start_node() {
    let graph = Graph::new();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.start_node().id(), 0);

    let graph = Graph::new();
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.start_node().id(), 0);
}
