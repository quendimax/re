use redt::Legible;

#[test]
fn u8_display() {
    assert_eq!(0.display().to_string(), r"00h");
    assert_eq!(b'\t'.display().to_string(), r"09h");
    assert_eq!(b'\r'.display().to_string(), r"0Dh");
    assert_eq!(b'\n'.display().to_string(), r"0Ah");
    assert_eq!(b'\''.display().to_string(), r"'\''");
    assert_eq!(b'"'.display().to_string(), r#"'"'"#);
    assert_eq!(b'\\'.display().to_string(), r"'\\'");
    assert_eq!(0x1B.display().to_string(), r"1Bh");
    assert_eq!(0x1f.display().to_string(), r"1Fh");
    assert_eq!(b' '.display().to_string(), "' '");
    assert_eq!(b'a'.display().to_string(), "'a'");
    assert_eq!(0x7F.display().to_string(), r"7Fh");
    assert_eq!(129.display().to_string(), r"81h");
    assert_eq!(255.display().to_string(), r"FFh");
}

#[test]
fn u8_arr_display() {
    assert_eq!(
        [0, 1, 2, b'\'', b'"', b'\\'].display().to_string(),
        r#""\x00\x01\x02'\"\\""#
    );
}
