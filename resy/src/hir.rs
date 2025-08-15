use redt::{Legible, SetU8};
use std::fmt::Write;

/// HirKind represents a high-level intermediate representation of a regular
/// expression, that contains already encoded into bytes unicode code points,
/// and can be used to build a graph of the corresponding finite automaton.
pub enum Hir {
    Disjunct(DisjunctHir),
    Concat(ConcatHir),
    Repeat(RepeatHir),
    Class(SetU8),
    Literal(Vec<u8>),
}

impl Hir {
    pub fn new_disjunct(alters: Vec<Hir>) -> Hir {
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

    pub fn new_concat(items: Vec<Hir>) -> Hir {
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

    pub fn new_repeat(item: Hir, min: usize, max: Option<usize>) -> Hir {
        Hir::Repeat(RepeatHir {
            min,
            max,
            item: Box::new(item),
        })
    }

    pub fn new_class(set: SetU8) -> Hir {
        Hir::Class(set)
    }

    pub fn new_literal(bytes: &[u8]) -> Hir {
        Hir::Literal(bytes.to_vec())
    }

    pub fn is_disjunct(&self) -> bool {
        matches!(self, Hir::Disjunct(..))
    }

    pub fn is_concat(&self) -> bool {
        matches!(self, Hir::Concat(..))
    }

    pub fn is_repeat(&self) -> bool {
        matches!(self, Hir::Repeat(..))
    }

    pub fn is_class(&self) -> bool {
        matches!(self, Hir::Class(..))
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Hir::Literal(..))
    }

    /// Returns the bounds of the Hir's length. `None` means infinite.
    pub fn len_hint(&self) -> (usize, Option<usize>) {
        match self {
            Hir::Disjunct(hir) => (hir.min_len, hir.max_len),
            Hir::Concat(hir) => (hir.min_len, hir.max_len),
            Hir::Repeat(hir) => {
                let (min_len, max_len) = hir.item.len_hint();
                if let Some(max) = hir.max
                    && let Some(max_len) = max_len
                {
                    (hir.min * min_len, Some(max * max_len))
                } else {
                    (hir.min * min_len, None)
                }
            }
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
}

pub struct DisjunctHir {
    pub alters: Vec<Hir>,
    min_len: usize,
    max_len: Option<usize>,
}

pub struct ConcatHir {
    pub items: Vec<Hir>,
    min_len: usize,
    max_len: Option<usize>,
}

pub struct RepeatHir {
    pub min: usize,
    pub max: Option<usize>,
    pub item: Box<Hir>,
}

impl std::fmt::Display for Hir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hir::Literal(bytes) => {
                for byte in bytes {
                    std::fmt::Display::fmt(&byte.display(), f)?;
                }
            }
            Hir::Class(set) => {
                f.write_char('[')?;
                for range in set.ranges() {
                    std::fmt::Display::fmt(&range.display(), f)?;
                }
                f.write_char(']')?;
            }
            Hir::Repeat(repeat) => {
                let item = &repeat.item;
                let needs_parens =
                    !item.is_class() && !item.is_repeat() && item.exact_len() != Some(1);
                if needs_parens {
                    f.write_char('(')?;
                }
                std::fmt::Display::fmt(&item, f)?;
                if needs_parens {
                    f.write_char(')')?;
                }
                match (repeat.min, repeat.max) {
                    (0, None) => f.write_char('*')?,
                    (1, None) => f.write_char('+')?,
                    (0, Some(1)) => f.write_char('?')?,
                    (lower, None) => write!(f, "{{{lower},}}")?,
                    (lower, Some(upper)) => write!(f, "{{{lower},{upper}}}")?,
                }
            }
            Hir::Concat(concat) => {
                for item in &concat.items {
                    std::fmt::Display::fmt(item, f)?;
                }
            }
            Hir::Disjunct(disjunct) => {
                let alters = &disjunct.alters;
                for i in 0..alters.len() {
                    if i + 1 < alters.len() {
                        f.write_char('|')?;
                    }
                    std::fmt::Display::fmt(&alters[i], f)?;
                }
            }
        }
        Ok(())
    }
}
