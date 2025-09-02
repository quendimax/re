use redt::{Legible, SetU8};
use std::fmt::Write;

/// Hir represents a high-level intermediate representation of a regular
/// expression, that contains bytes already encoded from unicode code points,
/// and can be used to build a graph of the corresponding finite automaton.
#[derive(Debug, Clone, PartialEq)]
pub enum Hir {
    Disjunct(DisjunctHir),
    Concat(ConcatHir),
    Repeat(RepeatHir),
    Group(GroupHir),
    Class(SetU8),
    Literal(Vec<u8>),
}

impl Hir {
    /// Creates a new disjunciton hir instance. If there is only one item, it
    /// returns that item.
    pub fn disjunct(items: impl Into<Vec<Hir>>) -> Hir {
        let alters = items.into();
        if alters.len() == 1 {
            return alters.into_iter().next().unwrap();
        }
        let mut min_len = usize::MAX;
        let mut max_len = Some(0);
        for alter in &alters {
            let (alter_min_len, alter_max_len) = alter.len_hint();
            min_len = min_len.min(alter_min_len);
            max_len = if let Some(max_len) = max_len
                && let Some(alter_max_len) = alter_max_len
            {
                Some(max_len.max(alter_max_len))
            } else {
                None
            };
        }
        if alters.is_empty() {
            min_len = 0;
        }
        Hir::Disjunct(DisjunctHir {
            alters,
            min_len,
            max_len,
        })
    }

    /// Creates a new concatenation hir instance. If there is only one item, it
    /// returns that item.
    pub fn concat(items: impl Into<Vec<Hir>>) -> Hir {
        let items = items.into();
        if items.len() == 1 {
            return items.into_iter().next().unwrap();
        }
        let mut min_len = 0;
        let mut max_len = Some(0);
        for item in &items {
            let (item_min, item_max) = item.len_hint();
            min_len += item_min;
            if let Some(max) = max_len
                && let Some(item_max) = item_max
            {
                max_len = Some(max + item_max);
            } else {
                max_len = None;
            }
        }
        Hir::Concat(ConcatHir {
            items,
            min_len,
            max_len,
        })
    }

    /// Creates a new repeat hir instance.
    pub fn repeat(item: Hir, lower: usize, upper: Option<usize>) -> Hir {
        if let Some(upper) = upper {
            assert!(lower <= upper);
        }
        Hir::Repeat(RepeatHir {
            lower,
            upper,
            item: Box::new(item),
        })
    }

    pub fn group(label: u32, item: Hir) -> Hir {
        Hir::Group(GroupHir {
            label,
            item: Box::new(item),
        })
    }

    /// Creates a new class hir instance, i.e. a choice between possible single bytes.
    #[inline]
    pub fn class(set: SetU8) -> Hir {
        Hir::Class(set)
    }

    /// Creates a new literal hir instance, i.e. a sequence of bytes
    #[inline]
    pub fn literal(bytes: impl Into<Vec<u8>>) -> Hir {
        Hir::Literal(bytes.into())
    }

    #[inline]
    pub fn is_disjunct(&self) -> bool {
        matches!(self, Hir::Disjunct(..))
    }

    #[inline]
    pub fn is_concat(&self) -> bool {
        matches!(self, Hir::Concat(..))
    }

    #[inline]
    pub fn is_repeat(&self) -> bool {
        matches!(self, Hir::Repeat(..))
    }

    #[inline]
    pub fn is_group(&self) -> bool {
        matches!(self, Hir::Group(..))
    }

    #[inline]
    pub fn is_class(&self) -> bool {
        matches!(self, Hir::Class(..))
    }

    #[inline]
    pub fn is_literal(&self) -> bool {
        matches!(self, Hir::Literal(..))
    }

    /// Returns the bounds of the Hir's length. `None` means infinite.
    pub fn len_hint(&self) -> (usize, Option<usize>) {
        match self {
            Hir::Disjunct(hir) => hir.len_hint(),
            Hir::Concat(hir) => hir.len_hint(),
            Hir::Repeat(hir) => hir.len_hint(),
            Hir::Group(hir) => hir.len_hint(),
            Hir::Class(_) => (1, Some(1)),
            Hir::Literal(bytes) => (bytes.len(), Some(bytes.len())),
        }
    }

    /// Returns `Some(len)` if this hir instance has the exact length, otherwise
    /// returns `None`.
    pub fn exact_len(&self) -> Option<usize> {
        let (lower, upper) = self.len_hint();
        if Some(lower) == upper { upper } else { None }
    }

    /// Unwraps the Hir into a DisjunctHir, panicking if it is not a repeat.
    pub fn unwrap_disjunct(self) -> DisjunctHir {
        match self {
            Hir::Disjunct(hir) => hir,
            _ => panic!("unwrap_disjunct called on non-disjunct hir"),
        }
    }

    /// Unwraps the Hir into a ConcatHir, panicking if it is not a repeat.
    pub fn unwrap_concat(self) -> ConcatHir {
        match self {
            Hir::Concat(hir) => hir,
            _ => panic!("unwrap_concat called on non-concat hir"),
        }
    }

    /// Unwraps the Hir into a RepeatHir, panicking if it is not a repeat.
    pub fn unwrap_repeat(self) -> RepeatHir {
        match self {
            Hir::Repeat(hir) => hir,
            _ => panic!("unwrap_repeat called on non-repeat hir"),
        }
    }

    /// Unwraps the Hir into a GroupHir, panicking if it is not a repeat.
    pub fn unwrap_group(self) -> GroupHir {
        match self {
            Hir::Group(hir) => hir,
            _ => panic!("unwrap_group called on non-group hir"),
        }
    }

    /// Unwraps the Hir into a byte set, panicking if it is not a literal.
    pub fn unwrap_class(self) -> SetU8 {
        match self {
            Hir::Class(hir) => hir,
            _ => panic!("unwrap_class called on non-class hir"),
        }
    }

    /// Unwraps the Hir into a byte array, panicking if it is not a literal.
    pub fn unwrap_literal(self) -> Vec<u8> {
        match self {
            Hir::Literal(hir) => hir,
            _ => panic!("unwrap_literal called on non-literal hir"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisjunctHir {
    alters: Vec<Hir>,
    min_len: usize,
    max_len: Option<usize>,
}

impl DisjunctHir {
    #[inline]
    pub fn alternatives(&self) -> &[Hir] {
        &self.alters
    }

    #[inline]
    pub fn len_hint(&self) -> (usize, Option<usize>) {
        (self.min_len, self.max_len)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcatHir {
    items: Vec<Hir>,
    min_len: usize,
    max_len: Option<usize>,
}

impl ConcatHir {
    #[inline]
    pub fn items(&self) -> &[Hir] {
        &self.items
    }

    #[inline]
    pub fn len_hint(&self) -> (usize, Option<usize>) {
        (self.min_len, self.max_len)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatHir {
    lower: usize,
    upper: Option<usize>,
    item: Box<Hir>,
}

impl RepeatHir {
    #[inline]
    pub fn inner(&self) -> &Hir {
        &self.item
    }

    pub fn len_hint(&self) -> (usize, Option<usize>) {
        let (min_len, max_len) = self.item.len_hint();
        if let Some(max) = self.upper
            && let Some(max_len) = max_len
        {
            (self.lower * min_len, Some(max * max_len))
        } else {
            (self.lower * min_len, None)
        }
    }

    /// Lower and upper bounds of possible number of iterations. `None` means infinite.
    #[inline]
    pub fn iter_hint(&self) -> (usize, Option<usize>) {
        (self.lower, self.upper)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupHir {
    label: u32,
    item: Box<Hir>,
}

impl GroupHir {
    #[inline]
    pub fn inner(&self) -> &Hir {
        &self.item
    }

    #[inline]
    pub fn len_hint(&self) -> (usize, Option<usize>) {
        self.item.len_hint()
    }
}

impl std::fmt::Display for Hir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hir::Literal(bytes) => {
                std::fmt::Display::fmt(&bytes.display(), f)?;
            }
            Hir::Class(set) => {
                std::fmt::Display::fmt(&set, f)?;
            }
            Hir::Group(group) => {
                write!(f, "(?<{}> {} )", group.label, group.item)?;
            }
            Hir::Repeat(repeat) => {
                let item = &repeat.item;
                let needs_parens = item.is_concat() || item.is_disjunct();
                if needs_parens {
                    f.write_char('(')?;
                }
                std::fmt::Display::fmt(&item, f)?;
                if needs_parens {
                    f.write_char(')')?;
                }
                match (repeat.lower, repeat.upper) {
                    (0, None) => f.write_char('*')?,
                    (1, None) => f.write_char('+')?,
                    (0, Some(1)) => f.write_char('?')?,
                    (lower, None) => write!(f, "{{{lower},}}")?,
                    (lower, Some(upper)) if lower == upper => write!(f, "{{{lower}}}")?,
                    (lower, Some(upper)) => write!(f, "{{{lower},{upper}}}")?,
                }
            }
            Hir::Concat(concat) => {
                let items = &concat.items;
                for i in 0..items.len() {
                    if items[i].is_disjunct() {
                        f.write_char('(')?;
                    }
                    std::fmt::Display::fmt(&items[i], f)?;
                    if items[i].is_disjunct() {
                        f.write_char(')')?;
                    }
                    if i + 1 < items.len() {
                        f.write_str(" & ")?;
                    }
                }
            }
            Hir::Disjunct(disjunct) => {
                let alters = &disjunct.alters;
                for i in 0..alters.len() {
                    if alters[i].is_concat() {
                        f.write_char('(')?;
                    }
                    std::fmt::Display::fmt(&alters[i], f)?;
                    if alters[i].is_concat() {
                        f.write_char(')')?;
                    }
                    if i + 1 < alters.len() {
                        f.write_str(" | ")?;
                    }
                }
            }
        }
        Ok(())
    }
}
