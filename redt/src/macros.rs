/// The macro crates a string literal from a series of documentation comments.
/// It allows to have multiple raw strings without any escaping.
#[macro_export]
macro_rules! lit {
    (#[doc=$first_line:literal] $(#[doc=$lines:literal])*) => {
        concat!($first_line, $("\n", $lines),*)
    };
}
