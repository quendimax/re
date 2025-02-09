use pretty_assertions::assert_eq;
use regr::NodeId;

#[test]
fn node_id() {
    let a: u32 = 1;
    let b: NodeId = 2;
    assert_eq!(a + b, 3);
}
