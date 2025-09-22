/// The macro creates a raw multiline string literal from a series of
/// documentation comments.
///
/// It allows to build strings without any escaping sequences.
#[macro_export]
macro_rules! lit {
    (#[doc=$first_line:literal] $(#[doc=$lines:literal])*) => {
        concat!($first_line, $("\n", $lines),*)
    };
}
