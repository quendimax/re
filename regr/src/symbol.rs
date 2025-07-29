/// A special symbol out of the language alphabet that represents automaton's
/// transition with no real symbol.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Epsilon;

macro_rules! impl_fmt {
    (std::fmt::$trait:ident for $type:ident) => {
        impl std::fmt::$trait for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Epsilon")
            }
        }
    };
}

impl_fmt!(std::fmt::Display for Epsilon);
impl_fmt!(std::fmt::Debug for Epsilon);
impl_fmt!(std::fmt::Binary for Epsilon);
impl_fmt!(std::fmt::Octal for Epsilon);
impl_fmt!(std::fmt::LowerHex for Epsilon);
impl_fmt!(std::fmt::UpperHex for Epsilon);
