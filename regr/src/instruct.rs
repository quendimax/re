/// Instructions represent the actions that can be performed during a transition
/// step.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Store the current position to the specified register
    StorePos(u32),

    /// Invalidate the specified register
    Invalidate(u32),
}
