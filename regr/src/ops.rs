pub trait Mergeable<T> {
    fn merge(&self, other: T) -> &Self;
}

pub trait Rejectable<T> {
    fn reject(&self, other: T) -> &Self;
}

pub use redt::ops::{Containable, Excludable, Includable, Intersectable};
