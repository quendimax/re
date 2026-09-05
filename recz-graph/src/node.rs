use crate::edge::{Edge, EdgePtr};
use crate::graph::{Graph, GraphInner, GraphPtr};
use core::cell::Cell;
use core::fmt;
use core::iter::Iterator;
use owo_colors::OwoColorize;
use recz_adt::{Legible, Map};
use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Normal,
    Final,
    Epilogue,
}

/// Node for an NFA graph.
///
/// It has an ID (unique within its graph owner). Also it can be connected to
/// another node via [`Transition`]'s.
pub struct Node<'a>(&'a NodeInner);

pub(crate) struct NodeInner {
    nid: u32,
    kind: Cell<NodeKind>,
    sources: Map<NodePtr, EdgePtr>,
    targets: Map<NodePtr, EdgePtr>,
    graph_ptr: GraphPtr,
}

pub(crate) type NodePtr = core::ptr::NonNull<NodeInner>;

/// Public API
impl<'a> Node<'a> {
    /// Returns the node's identifier unique within the running process.
    #[inline]
    pub fn nid(&self) -> u32 {
        self.0.nid
    }

    #[inline]
    pub fn kind(&self) -> NodeKind {
        self.0.kind.get()
    }

    #[inline]
    pub fn set_kind(&self, kind: NodeKind) {
        self.0.kind.set(kind);
    }

    /// Checks if the node is a final NFA state.
    ///
    /// To change the marker, use [`Node::finalize`] or [`Node::definalize`].
    #[inline]
    pub fn is_final(&self) -> bool {
        self.0.kind.get() == NodeKind::Final
    }

    /// Checks if the node is an epilogue DFA state.
    #[inline]
    pub fn is_epilogue(&self) -> bool {
        self.0.kind.get() == NodeKind::Epilogue
    }

    /// Make the node normal.
    #[inline]
    pub fn normalize(&self) -> Self {
        self.set_kind(NodeKind::Normal);
        *self
    }

    /// Make the node final.
    #[inline]
    pub fn finalize(&self) -> Self {
        self.set_kind(NodeKind::Final);
        *self
    }

    /// Make the node epilogized.
    #[inline]
    pub fn epilogize(&self) -> Self {
        self.set_kind(NodeKind::Epilogue);
        *self
    }

    /// Returns `true` if the node belongs to the given graph, `false`
    /// otherwise.
    pub fn belongs_to(&self, graph: &Graph) -> bool {
        ptr::eq(self.0.graph_inner(), graph.0.as_ref())
    }

    /// Creates a new empty edge between two nodes. You can fill the edge with
    /// symbols and tags later.
    pub fn connect(&self, to: Node<'a>) -> Edge<'a> {
        assert_eq!(
            self.0.graph_ptr, to.0.graph_ptr,
            "only nodes belonging to the same graph can be joined"
        );
        if let Some(edge) = self.0.targets.get(&to.as_ptr()) {
            Edge::from_ref(unsafe { edge.as_ref() })
        } else {
            let edge = self.0.graph_inner().edge();
            self.0.targets.insert(to.as_ptr(), edge.as_ptr());
            to.0.sources.insert(self.as_ptr(), edge.as_ptr());
            edge
        }
    }

    #[inline]
    pub fn target_count(&self) -> usize {
        self.0.targets.len()
    }

    #[inline]
    pub fn source_count(&self) -> usize {
        self.0.sources.len()
    }

    /// Returns an iterator over target nodes, i.e. nodes that this node has
    /// transitions to.
    ///
    /// This iterator walks over pairs `(Edge<'a>, Node<'a>)`.
    #[inline]
    pub fn targets(&self) -> impl Iterator<Item = (Edge<'a>, Node<'a>)> {
        self.0.targets.iter().map(|(to, tr)| {
            let node = Node::from_ref(unsafe { to.as_ref() });
            let edge = Edge::from_ref(unsafe { tr.as_ref() });
            (edge, node)
        })
    }

    /// Returns an iterator over target nodes, i.e. nodes that this node has
    /// transitions from.
    ///
    /// This iterator walks over pairs `(Node<'a>, Edge<'a>)`.
    #[inline]
    pub fn sources(&self) -> impl Iterator<Item = (Node<'a>, Edge<'a>)> {
        self.0.sources.iter().map(|(to, edge)| {
            let node = Node::from_ref(unsafe { to.as_ref() });
            let edge = Edge::from_ref(unsafe { edge.as_ref() });
            (node, edge)
        })
    }
}

/// Private API
impl<'a> Node<'a> {
    pub(crate) fn from_ref(inner: &'a NodeInner) -> Self {
        Self(inner)
    }

    pub(crate) fn as_ptr(&self) -> NodePtr {
        NodePtr::from(self.0)
    }

    pub(crate) fn fmt(&self, f: &mut std::fmt::Formatter<'_>, colored: bool) -> std::fmt::Result {
        match self.kind() {
            NodeKind::Normal => {
                if colored {
                    write!(f, "{}", "no_".bright_yellow())?;
                    write!(f, "{}", self.nid().bright_yellow())
                } else {
                    write!(f, "no_")?;
                    write!(f, "{}", self.nid())
                }
            }
            NodeKind::Final => {
                if colored {
                    write!(f, "{}", "fi_".bold().yellow())?;
                    write!(f, "{}", self.nid().bold().yellow())
                } else {
                    write!(f, "fi_")?;
                    write!(f, "{}", self.nid())
                }
            }
            NodeKind::Epilogue => {
                if colored {
                    write!(f, "{}", "eg_".bold().bright_yellow())?;
                    write!(f, "{}", self.nid().bold().bright_yellow())
                } else {
                    write!(f, "eg_")?;
                    write!(f, "{}", self.nid())
                }
            }
        }
    }
}

/// Crate API
impl NodeInner {
    #[inline(always)]
    pub(crate) fn new(graph: &GraphInner, nid: u32) -> NodeInner {
        NodeInner {
            nid,
            kind: Cell::new(NodeKind::Normal),
            sources: Default::default(),
            targets: Default::default(),
            graph_ptr: GraphPtr::from(graph),
        }
    }

    /// Graph owner of this node.
    #[inline]
    fn graph_inner<'a>(&self) -> &'a GraphInner {
        unsafe { self.graph_ptr.as_ref() }
    }
}

impl Copy for Node<'_> {}

impl Clone for Node<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::cmp::Eq for Node<'_> {}

impl std::cmp::PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.0, other.0)
    }
}

impl std::cmp::Ord for Node<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.nid().cmp(&other.nid())
    }
}

impl std::cmp::PartialOrd for Node<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Node<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const NodeInner).hash(state)
    }
}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::fmt(self, f, false)
    }
}

impl core::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::fmt(self, f, false)
    }
}

impl Legible for Node<'_> {
    fn legible(&self) -> impl core::fmt::Display {
        self
    }

    fn colored(&self) -> impl fmt::Display {
        struct Colored<'a, 'b>(&'a Node<'b>);
        impl core::fmt::Display for Colored<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Node::fmt(self.0, f, false)
            }
        }
        Colored(self)
    }
}
