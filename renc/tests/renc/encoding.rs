use pretty_assertions::assert_eq;
use renc::Encoding;

#[test]
fn encoding_ascii() {
    let encoding = Encoding::Ascii;
    assert_eq!(encoding.name(), "ASCII");
    assert_eq!(encoding.allows_surrogates(), false);
    assert_eq!(encoding.min_codepoint(), 0);
    assert_eq!(encoding.max_codepoint(), 0x7F);
}

#[test]
fn encoding_utf8() {
    let encoding = Encoding::Utf8;
    assert_eq!(encoding.name(), "UTF-8");
    assert_eq!(encoding.allows_surrogates(), false);
    assert_eq!(encoding.min_codepoint(), 0);
    assert_eq!(encoding.max_codepoint(), 0x10FFFF);
}
