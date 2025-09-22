/// Instruction represents the actions that can be performed during a transition
/// step.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Store the current position to the specified register
    WritePos(/*tag id*/ u32),

    /// Invalidate the specified register
    InvalidateTag(/*tag id*/ u32),
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Inst {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Inst::WritePos(tag) => write!(f, "wrpos t{tag}")?,
                    Inst::InvalidateTag(tag) => write!(f, "invd t{tag}")?,
                }
                Ok(())
            }
        }
    };
}

impl_fmt!(std::fmt::Display);
impl_fmt!(std::fmt::Debug);
