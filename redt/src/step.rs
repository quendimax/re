use num_traits::Num;

/// This trait adds some functionality needed by [`crate::Range`] type.
///
/// It could be based on [`std::iter::Step`] trait, but it is unstable yet.
pub trait Step: Num + Copy {
    /// Returns the number of steps required to get from `self` to `other` or
    /// vice versa.
    fn steps_between(&self, other: Self) -> Self;

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

    /// Checks if there is one step between the two values.
    fn adjoins(&self, other: Self) -> bool {
        self.steps_between(other) == Self::one()
    }
}

macro_rules! impl_step_for {
    ($type:ty) => {
        impl Step for $type {
            fn steps_between(&self, other: Self) -> Self {
                self.abs_diff(other)
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
    };
}

impl_step_for!(u8);
impl_step_for!(u16);
impl_step_for!(u32);
impl_step_for!(u64);
