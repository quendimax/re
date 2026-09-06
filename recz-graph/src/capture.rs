use crate::tag::{Tag, Tag::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CaptureLabel {
    Num(u32),
    Str(Box<str>),
}

impl core::convert::From<u8> for CaptureLabel {
    fn from(value: u8) -> Self {
        CaptureLabel::Num(value as u32)
    }
}

impl core::convert::From<u16> for CaptureLabel {
    fn from(value: u16) -> Self {
        CaptureLabel::Num(value as u32)
    }
}

impl core::convert::From<u32> for CaptureLabel {
    fn from(value: u32) -> Self {
        CaptureLabel::Num(value)
    }
}

impl core::convert::From<Box<str>> for CaptureLabel {
    fn from(value: Box<str>) -> Self {
        CaptureLabel::Str(value)
    }
}

impl core::convert::From<String> for CaptureLabel {
    fn from(value: String) -> Self {
        CaptureLabel::Str(value.into())
    }
}

impl<'a> core::convert::From<&'a str> for CaptureLabel {
    fn from(value: &'a str) -> Self {
        CaptureLabel::Str(value.into())
    }
}

impl equivalent::Equivalent<CaptureGroup> for CaptureLabel {
    fn equivalent(&self, other: &CaptureGroup) -> bool {
        PartialEq::eq(self, &other.label)
    }
}

impl std::fmt::Display for CaptureLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureLabel::Num(n) => write!(f, "{}", n),
            CaptureLabel::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureGroup {
    label: CaptureLabel,
    tag_index: u32,
}

impl CaptureGroup {
    pub fn label(&self) -> CaptureLabel {
        self.label.clone()
    }

    pub fn index(&self) -> usize {
        self.tag_index as usize
    }

    pub fn open_tag(&self) -> Tag {
        OpenGroup(self.tag_index)
    }

    pub fn close_tag(&self) -> Tag {
        CloseGroup(self.tag_index)
    }

    pub fn delete_tag(&self) -> Tag {
        DeleteGroup(self.tag_index)
    }
}

impl CaptureGroup {
    pub(crate) fn new(label: CaptureLabel, tag_index: u32) -> Self {
        Self { label, tag_index }
    }
}

impl core::hash::Hash for CaptureGroup {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}
