use redt::RangeU8;

/// HirKind represents a high-level intermediate representation of a regular
/// expression, that contains already encoded into bytes unicode code points,
/// and can be used to build a graph of the corresponding finite automaton.
pub enum Hir {
    Disjunct(DisjunctHir),
    Concat(ConcatHir),
    Repeat(RepeatHir),
    Range(RangeU8),
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

    pub fn new_range(start: u8, last: u8) -> Hir {
        Hir::Range(RangeU8::new(start, last))
    }

    pub fn new_literal(bytes: &[u8]) -> Hir {
        Hir::Literal(bytes.to_vec())
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
            Hir::Range(_) => (1, Some(1)),
            Hir::Literal(bytes) => (bytes.len(), Some(bytes.len())),
        }
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
