use pretty_assertions::assert_eq;
use redt::Range;
use renc::Encoding;

#[test]
fn encoding_ascii() {
    let encoding = Encoding::Ascii;
    assert_eq!(encoding.name(), "ASCII");
    assert_eq!(encoding.allows_surrogates(), false);
    assert_eq!(encoding.min_codepoint(), 0);
    assert_eq!(encoding.max_codepoint(), 0x7F);
    assert_eq!(encoding.codepoint_ranges(), &[Range::new(0, 0x7f)]);
}

#[test]
fn encoding_utf8() {
    let encoding = Encoding::Utf8;
    assert_eq!(encoding.name(), "UTF-8");
    assert_eq!(encoding.allows_surrogates(), false);
    assert_eq!(encoding.min_codepoint(), 0);
    assert_eq!(encoding.max_codepoint(), 0x10FFFF);
    assert_eq!(
        encoding.codepoint_ranges(),
        &[Range::new(0, 0xD7FF), Range::new(0xE000, 0x10FFFF)]
    );
}
