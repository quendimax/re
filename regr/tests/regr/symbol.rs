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

#[test]
fn u32_steps_between() {
    assert_eq!(1u32.steps_between(2), 1);
    assert_eq!(1u32.steps_between(1), 0);
    assert_eq!(8u32.steps_between(2), 6);
}

#[test]
fn u32_forward() {
    assert_eq!(1u32.forward(3), Some(4));
    assert_eq!(1u32.forward(u32::MAX as usize), None);
}

#[test]
fn u32_backward() {
    assert_eq!(1u32.backward(1), Some(0));
    assert_eq!(1u32.backward(2), None);
}

#[test]
fn u32_adjoins() {
    assert!(1u32.adjoins(2));
    assert!(1u32.adjoins(0));
    assert!(!1u32.adjoins(1));
    assert!(!1u32.adjoins(3));
}

#[test]
fn char_steps_between() {
    assert_eq!('a'.steps_between('c'), 2);
    assert_eq!('a'.steps_between('a'), 0);
    assert_eq!('c'.steps_between('a'), 2);
    assert_eq!('\u{D7FF}'.steps_between('\u{E000}'), 1);
}

#[test]
fn char_forward() {
    assert_eq!('a'.forward(2), Some('c'));
    assert_eq!('a'.forward(0x10FFFF), None);
    assert_eq!('\u{D7FF}'.forward(1), Some('\u{E000}'));
}

#[test]
fn char_backward() {
    assert_eq!('b'.backward(1), Some('a'));
    assert_eq!('a'.backward(0x10FFFF), None);
    assert_eq!('\u{E000}'.backward(1), Some('\u{D7FF}'));
}

#[test]
fn char_adjoins() {
    assert!('a'.adjoins('b'));
    assert!('b'.adjoins('a'));
    assert!(!'a'.adjoins('c'));
    assert!('\u{D7FF}'.adjoins('\u{E000}'));
    assert!(!'\u{D7FF}'.adjoins('\u{E001}'));
}
