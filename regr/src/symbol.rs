/// This trait adds some functionality needed by [`regr::Range`] type.
///
/// It could be based on [`std::iter::Step`] trait, but it is experimental and
/// is not available within Rust's stable standard library.
pub trait Symbol: Sized + Copy {
    /// Returns the number of steps required to get from `self` to `other` or
    /// vice versa.
    fn steps_between(&self, other: Self) -> usize;

    /// Returns the value that would be obtained by taking the _successor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn forward(&self, count: usize) -> Option<Self>;

    /// Returns the value that would be obtained by taking the _predecessor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn backward(&self, count: usize) -> Option<Self>;

    /// Checks if there is one step between the specified two values.
    fn adjoins(&self, other: Self) -> bool {
        self.steps_between(other) == 1
    }
}

impl Symbol for u8 {
    fn steps_between(&self, other: Self) -> usize {
        self.abs_diff(other).into()
    }

    fn forward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_add(count)
        } else {
            None
        }
    }

    fn backward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_sub(count)
        } else {
            None
        }
    }
}

impl Symbol for u32 {
    fn steps_between(&self, other: Self) -> usize {
        self.abs_diff(other) as usize
    }

    fn forward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_add(count)
        } else {
            None
        }
    }

    fn backward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_sub(count)
        } else {
            None
        }
    }
}

impl Symbol for char {
    fn steps_between(&self, other: Self) -> usize {
        let start = *self.min(&other) as u32;
        let end = *self.max(&other) as u32;
        let mut steps = start.abs_diff(end);
        if start < 0xD800 && 0xDFFF < end {
            steps -= 0xDFFF - 0xD800 + 1;
        }
        steps as usize
    }

    fn forward(&self, count: usize) -> Option<Self> {
        let start = *self as u32;
        let mut res = start.forward(count)?;
        if start < 0xD800 && 0xD800 <= res {
            res = res.forward(0x800)?;
        }
        if res <= char::MAX as u32 {
            // SAFETY: res is a valid unicode scalar
            // (below 0x110000 and not in 0xD800..0xE000)
            Some(unsafe { char::from_u32_unchecked(res) })
        } else {
            None
        }
    }

    fn backward(&self, count: usize) -> Option<Self> {
        let start = *self as u32;
        let mut res = start.backward(count)?;
        if start >= 0xE000 && 0xE000 > res {
            res = res.backward(0x800)?;
        }
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        Some(unsafe { char::from_u32_unchecked(res) })
    }
}
