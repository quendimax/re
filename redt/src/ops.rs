//! Collection of traits for set operations.

pub trait Containable<Rhs = Self> {
    fn contains(&self, rhs: Rhs) -> bool;
}

pub trait Intersectable<Rhs = Self> {
    fn intersects(&self, rhs: Rhs) -> bool;
}

pub trait Includable<Rhs = Self> {
    fn include(&mut self, rhs: Rhs) -> &mut Self;
}

pub trait Excludable<Rhs = Self> {
    fn exclude(&mut self, rhs: Rhs) -> &mut Self;
}
