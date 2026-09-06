use crate::capture::{CaptureGroup, CaptureLabel};
use crate::edge::{Edge, EdgeInner};
use crate::node::{Node, NodeInner, NodePtr};
use bumpish::BumpVec;
use owo_colors::OwoColorize;
use recz_adt::{Legible, Set};
use std::cell::Cell;
use std::fmt::Write;

/// Represents a Tagged NFA graph that holds nodes and edges between them.
///
/// A graph can creates nodes, edges (via [`Node`]'s API) and capturing groups.
///
/// A graph contains only one start node, that you can get via
/// [`Graph::start_node`].
///
/// # Examples
///
/// ```
/// use recz_graph::Graph;
///
/// let graph = Graph::new();
/// assert_eq!(graph.node_count(), 0);
/// assert_eq!(graph.edge_count(), 0);
///
/// graph.node().connect(graph.node());
/// assert_eq!(graph.node_count(), 2);
/// assert_eq!(graph.edge_count(), 1);
/// ```
pub struct Graph(pub(crate) Box<GraphInner>);

pub(crate) struct GraphInner {
    next_nid: Cell<u32>,
    start_node: Cell<Option<NodePtr>>,
    bump_nodes: BumpVec<NodeInner, 0>,
    bump_edges: BumpVec<EdgeInner, 0>,
    cap_groups: Set<CaptureGroup>,
}

pub(crate) type GraphPtr = core::ptr::NonNull<GraphInner>;

/// Public API
impl Graph {
    /// Creates a new graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph = Graph::new();
    /// assert_eq!(graph.node_count(), 0);
    /// assert_eq!(graph.edge_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self(Box::new(GraphInner {
            next_nid: Cell::new(0),
            start_node: Cell::new(None),
            bump_nodes: BumpVec::new(),
            bump_edges: BumpVec::new(),
            cap_groups: Set::default(),
        }))
    }

    /// Returns `true` if both graphs are the same instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph1 = Graph::new();
    /// assert!(graph1.is(&graph1));
    ///
    /// let graph2 = Graph::new();
    /// assert!(!graph1.is(&graph2));
    /// ```
    pub fn is(&self, other: &Graph) -> bool {
        core::ptr::eq(self.0.as_ref(), other.0.as_ref())
    }

    /// Creates a new node.
    ///
    /// If there was no start node, this node will be set as the start node.
    ///
    /// Also every new node gets a new node identifier that is unique within
    /// this graph.
    ///
    /// # Panics
    ///
    /// Panics if the node ID (`u32`) overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph = Graph::new();
    /// let node = graph.node();
    ///
    /// // first node is the start node
    /// assert_eq!(graph.start_node().nid(), node.nid());
    /// assert_ne!(graph.start_node().nid(), graph.node().nid());
    /// ```
    pub fn node(&self) -> Node<'_> {
        let nid = self.0.next_nid.replace(
            self.0
                .next_nid
                .get()
                .checked_add(1)
                .expect("node id overflow"),
        );
        let node_ref = self.0.bump_nodes.push(NodeInner::new(self.0.as_ref(), nid));
        let node_ptr = NodePtr::from(node_ref);

        if self.0.start_node.get().is_none() {
            self.0.start_node.set(Some(node_ptr));
        }
        Node::from_ref(node_ref)
    }

    /// Returns the start node of the graph.
    ///
    /// If the graph is empty, creates the node. That means that the first node
    /// always has NID `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use recz_graph::Graph;
    /// let graph = Graph::new();
    /// assert_eq!(graph.start_node().nid(), 0);
    /// ```
    #[inline]
    pub fn start_node(&self) -> Node<'_> {
        if let Some(ptr) = self.0.start_node.get() {
            Node::from_ref(unsafe { ptr.as_ref() })
        } else {
            self.node()
        }
    }

    /// Creates a new capture group for the given label. If a group with the
    /// same label already exists, returns it.
    ///
    /// `label` is an enum variant of [`CaptureLabel`], i.e. either a `Rc<str>`
    /// or an `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use recz_graph::Graph;
    /// let graph = Graph::new();
    /// let group = graph.group("foo");
    /// let group = graph.group(3u32);
    /// ```
    pub fn group(&self, label: impl Into<CaptureLabel>) -> &CaptureGroup {
        let label = label.into();
        if let Some(group) = self.0.cap_groups.get(&label) {
            group
        } else {
            let index: u32 = self
                .0
                .cap_groups
                .len()
                .try_into()
                .expect("capture group index overflow");
            let new_group = CaptureGroup::new(label, index);
            self.0.cap_groups.insert(new_group).unwrap()
        }
    }

    /// Returns a number of nodes belonging to this graph.
    ///
    /// It doesn't take into account if the nodes are connected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use recz_graph::Graph;
    /// let graph = Graph::new();
    /// assert_eq!(graph.node_count(), 0);
    ///
    /// graph.node();
    /// graph.node();
    /// assert_eq!(graph.node_count(), 2);
    /// ```
    #[inline]
    pub fn node_count(&self) -> usize {
        self.0.bump_nodes.len()
    }

    /// Returns a number of edges between nodes belonging to this graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph = Graph::new();
    ///
    /// graph.node();
    /// graph.node();
    /// assert_eq!(graph.edge_count(), 0);
    ///
    /// graph.node().connect(graph.node());
    /// assert_eq!(graph.edge_count(), 1);
    /// ```
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.0.bump_edges.len()
    }

    /// Returns `true` if the graph is empty (contains no nodes and edges).
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph = Graph::new();
    /// assert!(graph.is_empty());
    ///
    /// graph.node();
    /// assert!(!graph.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.bump_nodes.is_empty() && self.0.bump_edges.is_empty() && self.0.cap_groups.is_empty()
    }

    /// Returns an iterator over the nodes in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::Graph;
    ///
    /// let graph = Graph::new();
    /// assert!(graph.nodes().next().is_none());
    ///
    /// let node = graph.node();
    /// assert!(graph.nodes().next().is_some());
    /// ```
    pub fn nodes(&self) -> impl Iterator<Item = Node<'_>> {
        self.0.bump_nodes.iter().map(Node::from_ref)
    }

    /// Returns an iterator over the capture group labels in the graph. The
    /// order of group labels is the same as the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use recz_graph::{Graph, Tag};
    ///
    /// let graph = Graph::new();
    /// assert!(graph.groups().next().is_none());
    ///
    /// let group = graph.group("foo");
    /// assert!(graph.groups().next().is_some());
    /// ```
    pub fn groups(&self) -> impl Iterator<Item = &CaptureGroup> {
        self.0.cap_groups.iter()
    }
}

impl Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, colored: bool) -> std::fmt::Result {
        let white = |f: &mut std::fmt::Formatter<'_>, s: &str| {
            if colored {
                write!(f, "{}", s.white())
            } else {
                write!(f, "{}", s)
            }
        };
        if colored {
            write!(f, "{}", "graph".bold().bright_yellow())?;
        } else {
            write!(f, "graph")?;
        }
        white(f, " {")?;
        if !self.is_empty() {
            for node in self.nodes() {
                f.write_str("\n  ")?;
                node.fmt(f, colored)?;
                write!(f, " ")?;
                match node.target_count() {
                    0 => white(f, "{}")?,
                    1 => {
                        white(f, "{ ")?;
                        let (edge, target) = node.targets().next().unwrap();
                        edge.fmt(f, colored)?;
                        white(f, " -> ")?;
                        if node == target {
                            if colored {
                                write!(f, "{}", "self".bright_yellow())?;
                            } else {
                                write!(f, "self")?;
                            }
                        } else {
                            target.fmt(f, colored)?;
                        }
                        white(f, " }")?;
                    }
                    _ => {
                        white(f, "{")?;
                        for (tr, target) in node.targets() {
                            f.write_str("\n    ")?;
                            tr.fmt(f, colored)?;
                            white(f, " -> ")?;
                            if node == target {
                                if colored {
                                    write!(f, "{}", "self".bright_yellow())?;
                                } else {
                                    write!(f, "self")?;
                                }
                            } else {
                                target.fmt(f, colored)?;
                            }
                        }
                        white(f, "\n  }")?;
                    }
                }
            }
            f.write_char('\n')?;
        }
        white(f, "}")
    }
}

/// Private API
impl GraphInner {
    /// Creates a new edge.
    ///
    /// This method is not available for external use. Use [`Node::connect`] instead.
    pub(crate) fn edge(&self) -> Edge<'_> {
        Edge::from_ref(self.bump_edges.push(EdgeInner::new()))
    }
}

impl std::default::Default for Graph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Graph::fmt(self, f, false)
    }
}

impl core::fmt::Display for Graph {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Graph::fmt(self, f, false)
    }
}

impl Legible for Graph {
    #[inline]
    fn legible(&self) -> impl std::fmt::Display {
        self
    }

    #[inline]
    fn colored(&self) -> impl std::fmt::Display {
        struct ColoredGraph<'a>(&'a Graph);
        impl<'a> core::fmt::Display for ColoredGraph<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Graph::fmt(self.0, f, true)
            }
        }
        ColoredGraph(self)
    }
}
