use recz::re;

#[test]
fn simple_regex() {
    let mut regex = re!("hello");
    let m = regex.match_at("hello", 0).unwrap();
    assert_eq!(m.as_str(), "hello")
}
