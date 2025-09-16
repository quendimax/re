#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    /// register index in the register file
    register: u32,
    /// offset from the register value
    offset: usize,
}

impl Tag {
    pub fn new(register: u32) -> Self {
        Self {
            register,
            offset: 0,
        }
    }

    pub fn with_offset(register: u32, offset: usize) -> Self {
        Self { register, offset }
    }

    pub fn register(&self) -> u32 {
        self.register
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// Instruction represents the actions that can be performed during a transition
/// step.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Store the current position to the specified register
    StorePos(u32),

    /// Invalidate the specified register
    Invalidate(u32),
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Inst {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Inst::StorePos(reg) => write!(f, "strpos {reg}")?,
                    Inst::Invalidate(reg) => write!(f, "invld {reg}")?,
                }
                Ok(())
            }
        }
    };
}

impl_fmt!(std::fmt::Display);
impl_fmt!(std::fmt::Debug);
