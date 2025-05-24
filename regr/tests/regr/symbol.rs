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
    assert_eq!(255u8.forward(1), None);
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
fn u8_display() {
    assert_eq!(0.display().to_string(), r"00h");
    assert_eq!(b'\t'.display().to_string(), r"09h");
    assert_eq!(b'\r'.display().to_string(), r"0Dh");
    assert_eq!(b'\n'.display().to_string(), r"0Ah");
    assert_eq!(b'\''.display().to_string(), r"'''");
    assert_eq!(b'"'.display().to_string(), r#"'"'"#);
    assert_eq!(b'\\'.display().to_string(), r"'\'");
    assert_eq!(0x1B.display().to_string(), r"1Bh");
    assert_eq!(0x1f.display().to_string(), r"1Fh");
    assert_eq!(b' '.display().to_string(), "' '");
    assert_eq!(b'a'.display().to_string(), "'a'");
    assert_eq!(0x7F.display().to_string(), r"7Fh");
    assert_eq!(129.display().to_string(), r"81h");
    assert_eq!(255.display().to_string(), r"FFh");
}
