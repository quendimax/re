use crate::edge::Edge;
use crate::node::NodeId;
use crate::range::{Range, range};
use std::cell::{Ref, RefCell};
use std::ptr::NonNull;

pub struct Node<'a>(&'a NodeInner);

const SYMBOLS_COUNT: usize = 1 << u8::BITS;

pub(super) struct NodeInner {
    id: NodeId,
    targets: RefCell<[Option<NonNull<NodeInner>>; SYMBOLS_COUNT]>,
}

impl NodeInner {
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: RefCell::new([None; SYMBOLS_COUNT]),
        }
    }
}

impl Node<'_> {
    pub fn id(&self) -> NodeId {
        self.0.id
    }

    pub(super) fn as_ptr(&self) -> NonNull<NodeInner> {
        unsafe { NonNull::<NodeInner>::new_unchecked(self.0 as *const NodeInner as *mut NodeInner) }
    }

    pub(super) unsafe fn from_ptr(ptr: NonNull<NodeInner>) -> Self {
        Self(unsafe { ptr.as_ref() })
    }

    /// Connects this node to another node with a specified edge of symbols.
    /// If a connection to the target node already exists, it replaces
    /// the old target node with the new one.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge of sybols describing valid transitions to the target
    pub fn connect(&self, to: Node<'_>, with: impl Into<Edge>) {
        let to = to.as_ptr();
        let with: Edge = with.into();
        let mut targets = self.0.targets.borrow_mut();
        for range in with.ranges() {
            for sym in range.start()..=range.end() {
                targets[sym as usize] = Some(to);
            }
        }
    }

    #[inline]
    pub fn symbol_target_pairs(&self) -> SymbolTargetIter<'_> {
        SymbolTargetIter::new(*self)
    }
}

impl Copy for Node<'_> {}

impl Clone for Node<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> std::convert::From<&'a NodeInner> for Node<'a> {
    fn from(value: &'a NodeInner) -> Self {
        Self(value)
    }
}

impl<'a> std::convert::From<&'a mut NodeInner> for Node<'a> {
    fn from(value: &'a mut NodeInner) -> Self {
        Self(value)
    }
}

impl std::cmp::PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl std::cmp::Eq for Node<'_> {}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node({})", self.id())
    }
}

pub struct SymbolTargetIter<'a> {
    index: usize,
    targets: Ref<'a, [Option<NonNull<NodeInner>>; SYMBOLS_COUNT]>,
}

impl<'a> SymbolTargetIter<'a> {
    pub fn new(node: Node<'a>) -> Self {
        Self {
            index: 0,
            targets: node.0.targets.borrow(),
        }
    }
}

impl<'a> std::iter::Iterator for SymbolTargetIter<'a> {
    type Item = (Range, Node<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < SYMBOLS_COUNT {
            // TODO: replace with `get_unchecked` after some testing
            if let Some(target_ptr) = self.targets[self.index] {
                let first_target_ptr = target_ptr;
                let first_index = self.index;
                self.index += 1;
                while self.index < SYMBOLS_COUNT {
                    if let Some(target_ptr) = self.targets[self.index] {
                        if target_ptr == first_target_ptr {
                            self.index += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                println!("first {first_index}, index {}", self.index);
                return Some((range(first_index as u8..=(self.index - 1) as u8), unsafe {
                    Node::from_ptr(target_ptr)
                }));
            } else {
                self.index += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod utest {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn node_from_ptr_and_from_ref() {
        let inner = NodeInner::new(42);
        let inner_ref = &inner;
        let inner_ptr = NonNull::from(&inner);
        let node_one = Node::from(inner_ref);
        let node_two = unsafe { Node::from_ptr(inner_ptr) };
        assert_eq!(inner.id, node_one.id());
        assert_eq!(inner.id, node_two.id());
    }
}
