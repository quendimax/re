pub trait ContainOp<T> {
    fn contains(&self, other: T) -> bool;
}

pub trait IntersectOp<T> {
    fn intersects(&self, other: T) -> bool;
}

pub trait MergeOp<T> {
    fn merge(&self, other: T);
}
