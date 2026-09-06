use owo_colors::OwoColorize;
use recz_adt::Legible;

/// Represents tags in tagged NFA/DFA.
///
/// In practice, the tags are converted into actions during NFA/DFA execution,
/// co you cann look at them as instruction of a NFA/DFA virtual machine.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tag {
    /// A tag used to mark the start of a capture group.
    OpenGroup(u32),

    /// A tag used to mark the end of a capture group.
    CloseGroup(u32),

    /// A tag used to mark a group's tags for deletion.
    DeleteGroup(u32),
}

use Tag::*;

impl Tag {
    pub fn delete_group(&self) -> Option<Self> {
        match self {
            OpenGroup(index) => Some(DeleteGroup(*index)),
            CloseGroup(index) => Some(DeleteGroup(*index)),
            DeleteGroup(_) => None,
        }
    }

    pub(crate) fn fmt(&self, f: &mut std::fmt::Formatter<'_>, colored: bool) -> std::fmt::Result {
        if colored {
            match self {
                OpenGroup(group_idx) => {
                    write!(f, "{}{}", "+g".bright_blue(), group_idx.bright_blue())
                }
                CloseGroup(group_idx) => {
                    write!(f, "{}{}", "-g".bright_blue(), group_idx.bright_blue())
                }
                DeleteGroup(group_idx) => {
                    write!(f, "{}{}", "!g".bright_blue(), group_idx.bright_blue())
                }
            }
        } else {
            match self {
                OpenGroup(group_idx) => write!(f, "+g{group_idx}"),
                CloseGroup(group_idx) => write!(f, "-g{group_idx}"),
                DeleteGroup(group_idx) => write!(f, "!g{group_idx}"),
            }
        }
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, false)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, false)
    }
}

impl Legible for Tag {
    fn legible(&self) -> impl core::fmt::Display {
        self
    }

    fn colored(&self) -> impl core::fmt::Display {
        struct ColoredTag(Tag);
        impl core::fmt::Display for ColoredTag {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Tag::fmt(&self.0, f, true)
            }
        }
        ColoredTag(*self)
    }
}
