#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Primary {
        /// the tag's id
        id: u32,
    },
    Secondary {
        /// the tag's id
        id: u32,
        /// starting primary tag id
        start_id: u32,
        /// offset from the starting tag's value
        offset: usize,
    },
    Tertiary {
        /// the tag's id
        id: u32,
        /// starting secondary tag id
        start_id: u32,
        /// offset from the starting tag's value
        offset: usize,
    },
}

impl Tag {
    pub fn primary(id: u32) -> Self {
        Self::Primary { id }
    }

    pub fn secondary(id: u32, start_tag_id: u32, offset: usize) -> Self {
        Self::Secondary {
            id,
            start_id: start_tag_id,
            offset,
        }
    }

    pub fn tertiary(id: u32, start_tag_id: u32, offset: usize) -> Self {
        Self::Tertiary {
            id,
            start_id: start_tag_id,
            offset,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Self::Primary { id, .. } => *id,
            Self::Secondary { id, .. } => *id,
            Self::Tertiary { id, .. } => *id,
        }
    }

    pub fn starting_tag(&self) -> u32 {
        match self {
            Self::Primary { id, .. } => *id,
            Self::Secondary { start_id, .. } => *start_id,
            Self::Tertiary { start_id, .. } => *start_id,
        }
    }

    pub fn offset(&self) -> usize {
        match self {
            Self::Primary { .. } => 0,
            Self::Secondary { offset, .. } => *offset,
            Self::Tertiary { offset, .. } => *offset,
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Primary { .. })
    }

    pub fn is_secondary(&self) -> bool {
        matches!(self, Self::Secondary { .. })
    }

    pub fn is_tertiary(&self) -> bool {
        matches!(self, Self::Tertiary { .. })
    }
}
