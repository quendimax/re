#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Absolute {
        /// the tag's id
        id: u32,

        /// the tag's register id
        reg: u32,
    },
    PseudoAbsolute {
        /// the tag's id
        id: u32,
        /// starting primary tag id
        starting_tag: u32,
        /// offset from the starting tag's value
        offset: usize,
    },
    Relative {
        /// the tag's id
        id: u32,
        /// starting secondary tag id
        starting_tag: u32,
        /// offset from the starting tag's value
        offset: usize,
    },
}

impl Tag {
    pub fn id(&self) -> u32 {
        match self {
            Self::Absolute { id, .. } => *id,
            Self::PseudoAbsolute { id, .. } => *id,
            Self::Relative { id, .. } => *id,
        }
    }

    pub fn offset(&self) -> usize {
        match self {
            Self::Absolute { .. } => 0,
            Self::PseudoAbsolute { offset, .. } => *offset,
            Self::Relative { offset, .. } => *offset,
        }
    }

    pub fn add_offset(&mut self, offset: usize) {
        match self {
            Self::Absolute { .. } => panic!("Cannot add offset to absolute tag"),
            Self::PseudoAbsolute {
                offset: self_offset,
                ..
            } => {
                *self_offset += offset;
            }
            Self::Relative {
                offset: self_offset,
                ..
            } => {
                *self_offset += offset;
            }
        };
    }

    pub fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute { .. })
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Absolute { id, reg } => write!(f, "a-tag(id={id}, reg={reg})"),
            Tag::PseudoAbsolute {
                id,
                starting_tag,
                offset,
            } => {
                write!(
                    f,
                    "p-tag(id={id}, start_tag={starting_tag}, offset={offset})"
                )
            }
            Tag::Relative {
                id,
                starting_tag,
                offset,
            } => {
                write!(
                    f,
                    "r-tag(id={id}, start_tag={starting_tag}, offset={offset})"
                )
            }
        }
    }
}

pub struct TagBank {
    next_id: u32,
    next_reg: u32,
}

impl TagBank {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            next_reg: 0,
        }
    }

    pub fn absolute(&mut self) -> Tag {
        let id = self.next_id();
        let reg = self.next_reg();
        Tag::Absolute { id, reg }
    }

    pub fn pseudo_absolute(&mut self, tag: Tag, offset: usize) -> Tag {
        assert!(matches!(tag, Tag::Absolute { .. }));
        let id = self.next_id();
        Tag::PseudoAbsolute {
            id,
            starting_tag: tag.id(),
            offset,
        }
    }

    pub fn relative(&mut self, base: Tag, offset: usize) -> Tag {
        let other_offset = offset;
        match base {
            Tag::Absolute { id, .. } => Tag::Relative {
                id: self.next_id(),
                starting_tag: id,
                offset,
            },
            Tag::PseudoAbsolute { id, .. } => Tag::Relative {
                id: self.next_id(),
                starting_tag: id,
                offset,
            },
            Tag::Relative {
                starting_tag,
                offset,
                ..
            } => Tag::Relative {
                id: self.next_id(),
                starting_tag,
                offset: other_offset + offset,
            },
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = id.checked_add(1).expect("tag id overflow");
        id
    }

    fn next_reg(&mut self) -> u32 {
        let reg = self.next_reg;
        self.next_reg = reg.checked_add(1).expect("register id overflow");
        reg
    }
}

impl std::default::Default for TagBank {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
