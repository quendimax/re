//! Collection of traits for set operations.

pub trait ContainOp<Rhs = Self> {
    fn contains(&self, rhs: Rhs) -> bool;
}

pub trait IntersectOp<Rhs = Self> {
    fn intersects(&self, rhs: Rhs) -> bool;
}

pub trait IncludeOp<Rhs = Self> {
    fn include(&mut self, rhs: Rhs);
}

pub trait ExcludeOp<Rhs = Self> {
    fn exclude(&mut self, rhs: Rhs);
}
