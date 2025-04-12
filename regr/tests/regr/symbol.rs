use pretty_assertions::assert_eq;
use regr::Symbol;

#[test]
fn u8_steps_between() {
    assert_eq!(1u8.steps_between(2), 1);
    assert_eq!(1u8.steps_between(1), 0);
    assert_eq!(8u8.steps_between(2), 6);
}

#[test]
fn u8_forward() {
    assert_eq!(1u8.forward(3), Some(4));
    assert_eq!(1u8.forward(1000), None);
}

#[test]
fn u8_backward() {
    assert_eq!(1u8.backward(1), Some(0));
    assert_eq!(1u8.backward(2), None);
}

#[test]
fn u8_adjoins() {
    assert!(1u8.adjoins(2));
    assert!(1u8.adjoins(0));
    assert!(!1u8.adjoins(1));
    assert!(!1u8.adjoins(3));
}
