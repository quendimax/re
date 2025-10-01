use redt::{RangeU8, SetU8};

/// A set of symbols that can be used to represent a language alphabet + Epsilon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolSet {
    set: SetU8,
    epsilon: bool,
}

impl SymbolSet {
    /// Creates a new empty symbol set.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            set: SetU8::default(),
            epsilon: false,
        }
    }
}

impl std::default::Default for SymbolSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for SymbolSet {
    type Target = SetU8;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl std::ops::DerefMut for SymbolSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

macro_rules! impl_op_from_set {
    ($trait:ident, $method:ident, $type:ty, $ret_type:ty) => {
        impl redt::ops::$trait<$type> for SymbolSet {
            fn $method(&self, rhs: $type) -> $ret_type {
                self.set.$method(rhs)
            }
        }
    };
    ($trait:ident, $method:ident(mut), $type:ty, $ret_type:ty) => {
        impl redt::ops::$trait<$type> for SymbolSet {
            fn $method(&mut self, rhs: $type) -> $ret_type {
                self.set.$method(rhs)
            }
        }
    };
}

impl_op_from_set!(ContainOp, contains, u8, bool);
impl_op_from_set!(ContainOp, contains, RangeU8, bool);
impl_op_from_set!(ContainOp, contains, SetU8, bool);
impl_op_from_set!(ContainOp, contains, &SetU8, bool);
impl_op_from_set!(IntersectOp, intersects, u8, bool);
impl_op_from_set!(IntersectOp, intersects, RangeU8, bool);
impl_op_from_set!(IntersectOp, intersects, SetU8, bool);
impl_op_from_set!(IntersectOp, intersects, &SetU8, bool);
impl_op_from_set!(IncludeOp, include(mut), u8, ());
impl_op_from_set!(IncludeOp, include(mut), RangeU8, ());
impl_op_from_set!(IncludeOp, include(mut), SetU8, ());
impl_op_from_set!(IncludeOp, include(mut), &SetU8, ());
impl_op_from_set!(ExcludeOp, exclude(mut), u8, ());
impl_op_from_set!(ExcludeOp, exclude(mut), RangeU8, ());
impl_op_from_set!(ExcludeOp, exclude(mut), SetU8, ());
impl_op_from_set!(ExcludeOp, exclude(mut), &SetU8, ());

impl redt::ops::ContainOp<Epsilon> for SymbolSet {
    #[inline]
    fn contains(&self, _: Epsilon) -> bool {
        self.epsilon
    }
}

impl redt::ops::ContainOp<&Self> for SymbolSet {
    fn contains(&self, other: &Self) -> bool {
        self.epsilon & other.epsilon == other.epsilon && self.set.contains(&other.set)
    }
}

impl redt::ops::ContainOp<&mut Self> for SymbolSet {
    #[inline]
    fn contains(&self, other: &mut Self) -> bool {
        redt::ops::ContainOp::<&Self>::contains(self, other)
    }
}

impl redt::ops::IntersectOp<Epsilon> for SymbolSet {
    #[inline]
    fn intersects(&self, _: Epsilon) -> bool {
        self.epsilon
    }
}

impl redt::ops::IntersectOp<&Self> for SymbolSet {
    fn intersects(&self, other: &Self) -> bool {
        (self.epsilon && other.epsilon) || self.set.intersects(&other.set)
    }
}

impl redt::ops::IntersectOp<&mut Self> for SymbolSet {
    #[inline]
    fn intersects(&self, other: &mut Self) -> bool {
        redt::ops::IntersectOp::<&Self>::intersects(self, other)
    }
}

impl redt::ops::IncludeOp<Epsilon> for SymbolSet {
    #[inline]
    fn include(&mut self, _: Epsilon) {
        self.epsilon = true;
    }
}

impl redt::ops::IncludeOp<&Self> for SymbolSet {
    fn include(&mut self, other: &Self) {
        self.set.include(&other.set);
        self.epsilon |= other.epsilon;
    }
}

impl redt::ops::ExcludeOp<Epsilon> for SymbolSet {
    fn exclude(&mut self, _: Epsilon) {
        self.epsilon = false;
    }
}

impl redt::ops::ExcludeOp<&Self> for SymbolSet {
    fn exclude(&mut self, other: &Self) {
        self.set.exclude(&other.set);
        self.epsilon &= !other.epsilon;
    }
}

impl redt::ops::ExcludeOp<&mut Self> for SymbolSet {
    #[inline]
    fn exclude(&mut self, rhs: &mut Self) {
        redt::ops::ExcludeOp::<&Self>::exclude(self, rhs);
    }
}

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
