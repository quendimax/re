/// This trait adds some functionality needed by [`regr::Range`] type.
///
/// It could be based on [`std::iter::Step`] trait, but it is experimental and
/// is not available within Rust's stable standard library.
pub trait Symbol: Sized + Copy {
    /// Returns the number of steps required to get from `self` to `other` or vice versa.
    fn steps_between(&self, other: Self) -> usize;

    /// Checks if there is one step between the specified two values.
    fn adjoins(&self, other: Self) -> bool {
        self.steps_between(other) == 1
    }
}

impl Symbol for u8 {
    fn steps_between(&self, other: Self) -> usize {
        self.abs_diff(other).into()
    }
}

impl Symbol for u32 {
    fn steps_between(&self, other: Self) -> usize {
        self.abs_diff(other) as usize
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
}
