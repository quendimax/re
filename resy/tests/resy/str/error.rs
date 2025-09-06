use pretty_assertions::assert_eq;
use resy::str::Error;

#[test]
fn error_span() {
    let err = Error::EncoderError {
        cause: renc::Error::SmallBuffer,
        span: 0..11,
    };
    assert_eq!(err.error_span(), 0..11);

    let err = Error::Unexpected {
        found_spell: "asf".into(),
        found_span: 1..10,
        expected: "".into(),
    };
    assert_eq!(err.error_span(), 1..10);

    let err = Error::OutOfRange {
        value: "".into(),
        span: 0..5,
        range: "".into(),
    };
    assert_eq!(err.error_span(), 0..5);

    let err = Error::EmptyEscape { span: 0..6 };
    assert_eq!(err.error_span(), 0..6);

    let err = Error::UnsupportedEscape {
        sequence: "".into(),
        span: 32..44,
    };
    assert_eq!(err.error_span(), 32..44);

    let err = Error::ZeroRepetition { span: 2..5 };
    assert_eq!(err.error_span(), 2..5);

    let err = Error::InvalidRepetition { span: 0..3 };
    assert_eq!(err.error_span(), 0..3);
}
