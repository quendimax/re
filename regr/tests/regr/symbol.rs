use pretty_assertions::assert_eq;
use regr::Epsilon;

#[test]
fn epsilon_display() {
    assert_eq!(format!("{Epsilon}"), r"Epsilon");
}
