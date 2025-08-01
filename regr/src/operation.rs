/// Operations represent the actions that can be performed during a transition.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    /// Store the current position to the specified register
    StorePos(u32),

    /// Invalidate the specified register
    Invalidate(u32),
}
