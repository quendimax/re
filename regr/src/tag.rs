#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    /// register index in the register file
    register: Reg,
    /// offset from the register value
    offset: usize,
}

impl Tag {
    pub fn new(register: Reg) -> Self {
        Self {
            register,
            offset: 0,
        }
    }

    pub fn with_offset(register: Reg, offset: usize) -> Self {
        Self { register, offset }
    }

    pub fn register(&self) -> Reg {
        self.register
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reg(u32);

impl Reg {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.id())
    }
}

/// Instruction represents the actions that can be performed during a transition
/// step.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Store the current position to the specified register
    StorePos(Reg),

    /// Invalidate the specified register
    Invalidate(Reg),
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
