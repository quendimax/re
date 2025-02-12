use crate::edge::Edge;
use crate::node::NodeId;
use crate::range::Range;
use crate::symbol::Symbol;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::ptr::NonNull;

#[derive(Clone, Copy)]
pub struct Node<'a, T>(&'a NodeInner<T>);

impl<'a, T> std::convert::From<&'a NodeInner<T>> for Node<'a, T> {
    fn from(value: &'a NodeInner<T>) -> Self {
        Self(value)
    }
}

impl<'a, T> std::convert::From<&'a mut NodeInner<T>> for Node<'a, T> {
    fn from(value: &'a mut NodeInner<T>) -> Self {
        Self(value)
    }
}

impl<T> std::convert::From<NodePtr<T>> for Node<'_, T> {
    fn from(value: NodePtr<T>) -> Self {
        Self(unsafe { value.0.as_ref() })
    }
}

impl<T> Node<'_, T> {
    pub fn id(&self) -> NodeId {
        self.0.id
    }
}

impl<T> std::cmp::PartialEq for Node<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl<T> std::cmp::Eq for Node<'_, T> {}

impl<T> std::cmp::PartialOrd for Node<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> std::cmp::Ord for Node<'_, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl<T> std::hash::Hash for Node<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state)
    }
}

impl<T> std::fmt::Debug for Node<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node({})", self.id())
    }
}

impl<T> std::fmt::Display for Node<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<'a, T: std::fmt::Debug + PartialEq + PartialOrd + Ord + Symbol> Node<'a, T> {
    /// Connects this node to another node with the specified range.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The range to use for the connection
    ///
    /// # Panics
    ///
    /// Panics if there is already a node connected with a symbol from the given range
    pub fn connect_with_range(&self, to: Node<'a, T>, with: impl Into<Range<T>>) {
        let to = NodePtr::from(to);
        let with = with.into();
        let mut index = 0;
        let mut is_adjoint = false;
        for (range, _) in self.0.targets.borrow().iter() {
            if range.intersects(&with) {
                panic!(
                    "there is a node connected with a symbol from the range {:?}",
                    with
                );
            }
            if range.adjoins(&with) {
                is_adjoint = true;
                break;
            }
            index += 1;
        }
        let mut ranges = self.0.targets.borrow_mut();
        if is_adjoint {
            ranges[index].0.merge(&with);
        } else {
            ranges.insert(index, (with, to));
        }
    }

    /// Connects this node to another node with the specified edge.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge containing ranges to use for the connections
    ///
    /// # Panics
    ///
    /// Panics if there is already a node connected with a symbol from any of the ranges
    pub fn connect(&self, to: Node<'a, T>, with: impl Into<Edge<T>>) {
        let with = with.into();
        for range in with.ranges() {
            self.connect_with_range(to, range.clone());
        }
    }
}

pub(super) struct NodePtr<T>(NonNull<NodeInner<T>>);

impl<T> NodePtr<T> {
    // pub(super) unsafe fn from_ptr(ptr: *mut NodeInner<T>) -> Self {
    //     Self(NonNull::new(ptr).unwrap())
    // }

    pub(super) fn from_ref(refer: &NodeInner<T>) -> Self {
        Self(NonNull::from(refer))
    }
}

impl<T> std::marker::Copy for NodePtr<T> {}

impl<T> std::clone::Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::convert::From<Node<'_, T>> for NodePtr<T> {
    fn from(value: Node<'_, T>) -> Self {
        Self(NonNull::from(value.0))
    }
}

impl<T: Clone> std::cmp::PartialEq for NodePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        let this = unsafe { self.0.as_ref() };
        let other = unsafe { other.0.as_ref() };
        std::cmp::PartialEq::eq(&Node::from(this), &Node::from(other))
    }
}

impl<T: Clone> std::cmp::Eq for NodePtr<T> {}

impl<T: Clone> std::cmp::PartialOrd for NodePtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone> std::cmp::Ord for NodePtr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let this = unsafe { self.0.as_ref() };
        let other = unsafe { other.0.as_ref() };
        std::cmp::Ord::cmp(&Node::from(this), &Node::from(other))
    }
}

impl<T> std::hash::Hash for NodePtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Node::from(unsafe { self.0.as_ref() }).hash(state)
    }
}

impl<T> std::fmt::Debug for NodePtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Node::from(unsafe { self.0.as_ref() }).fmt(f)
    }
}

impl<T> std::fmt::Display for NodePtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub(super) struct NodeInner<T> {
    id: NodeId,
    targets: RefCell<SmallVec<[(Range<T>, NodePtr<T>); 2]>>,
}

impl<T> NodeInner<T> {
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: Default::default(),
        }
    }
}
