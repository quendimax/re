use pretty_assertions::assert_eq;
use regr::Graph;
use resy::Parser;

#[test]
fn parse_escape() {
    let graph = Graph::nfa();
    let start_node = graph.start_node();
    let mut parser = Parser::new(&graph);
    let end_node = parser.parse(r"\\", start_node).unwrap();
    assert_eq!(end_node.nid(), 1);
}
