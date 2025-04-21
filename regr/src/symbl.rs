/// A symbol that specify transition condition for DFA/NFA.
///
/// It is just a wrapper around a byte.
pub struct Symbl(u8);

/// Constructor for `Symbl`.
#[inline]
pub fn symbl(value: impl Into<u8>) -> Symbl {
    Symbl(value.into())
}

impl Symbl {
    /// Checks if there is one step between the specified two values, or in
    /// other words both symbls are neighbours.
    #[inline]
    pub fn adjoins(self, other: Symbl) -> bool {
        self.steps_between(other) == 1
    }

    #[inline]
    pub fn steps_between(self, other: Self) -> usize {
        self.0.abs_diff(other.0).into()
    }

    pub fn forward(self, count: usize) -> Option<Self> {
        if let Ok(count) = u8::try_from(count) {
            self.0.checked_add(count).map(|v| symbl(v))
        } else {
            None
        }
    }

    pub fn backward(self, count: usize) -> Option<Self> {
        if let Ok(count) = u8::try_from(count) {
            self.0.checked_sub(count).map(|v| symbl(v))
        } else {
            None
        }
    }
}

impl std::default::Default for Symbl {
    #[inline]
    fn default() -> Self {
        Self(0)
    }
}

impl Copy for Symbl {}

impl Clone for Symbl {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl std::convert::From<u8> for Symbl {
    #[inline]
    fn from(value: u8) -> Self {
        Symbl(value)
    }
}

impl std::convert::From<&u8> for Symbl {
    #[inline]
    fn from(value: &u8) -> Self {
        Symbl(*value)
    }
}

impl std::convert::From<Symbl> for u8 {
    #[inline]
    fn from(value: Symbl) -> Self {
        value.0
    }
}

impl std::convert::TryFrom<usize> for Symbl {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Symbl(u8::try_from(value)?))
    }
}

impl std::convert::AsRef<u8> for Symbl {
    #[inline]
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl std::convert::AsMut<u8> for Symbl {
    #[inline]
    fn as_mut(&mut self) -> &mut u8 {
        &mut self.0
    }
}

impl std::cmp::Eq for Symbl {}

impl std::cmp::PartialEq for Symbl {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::cmp::PartialEq::eq(&self.0, &other.0)
    }
}

impl std::cmp::Ord for Symbl {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        std::cmp::Ord::cmp(&self.0, &other.0)
    }
}

impl std::cmp::PartialOrd for Symbl {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        std::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
    }
}

impl std::hash::Hash for Symbl {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl std::fmt::Display for Symbl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_ascii_graphic() {
            std::fmt::Display::fmt(&char::from(self.0), f)
        } else {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
}

impl std::fmt::Debug for Symbl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_ascii_graphic() {
            std::fmt::Debug::fmt(&char::from(self.0), f)
        } else {
            std::fmt::Debug::fmt(&self.0, f)
        }
    }
}

macro_rules! reimpl {
    (std::fmt::$trait:ident for $outer_type:ident) => {
        impl std::fmt::$trait for $outer_type {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::$trait::fmt(&self.0, f)
            }
        }
    };
}

reimpl!(std::fmt::Binary for Symbl);
reimpl!(std::fmt::Octal for Symbl);
reimpl!(std::fmt::LowerHex for Symbl);
reimpl!(std::fmt::UpperHex for Symbl);
