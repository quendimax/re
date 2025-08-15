use redt::SetU8;

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

impl SymbolSet {
    pub(crate) fn contains_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    pub(crate) fn contains_symbol(&self, symbol: u8) -> bool {
        self.set.contains_byte(symbol)
    }

    pub(crate) fn contains_symset(&self, other: &SymbolSet) -> bool {
        self.epsilon & other.epsilon == other.epsilon && self.set.contains_set(&other.set)
    }

    pub(crate) fn intersects_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    pub(crate) fn intersects_symbol(&self, symbol: u8) -> bool {
        self.set.intersects_byte(symbol)
    }

    pub(crate) fn intersects_symset(&self, other: &SymbolSet) -> bool {
        (self.epsilon && other.epsilon) || self.set.intersects_set(&other.set)
    }

    pub(crate) fn merge_epsilon(&mut self, _: Epsilon) {
        self.epsilon = true;
    }

    pub(crate) fn merge_symbol(&mut self, symbol: u8) {
        self.set.merge_byte(symbol)
    }

    pub(crate) fn merge_symset(&mut self, other: &SymbolSet) {
        self.set.merge_set(&other.set);
        self.epsilon |= other.epsilon;
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
