//! Iterators.

use crate::Node;

macro_rules! impl_node_iterator {
    ($name: ident, $next: expr) => {
        impl<T> Iterator for $name<T> {
            type Item = Node<T>;

            /// # Panics
            ///
            /// Panics if the node about to be yielded is currently mutably borrowed.
            fn next(&mut self) -> Option<Self::Item> {
                match self.0.take() {
                    Some(node) => {
                        self.0 = $next(&node);
                        Some(node)
                    }
                    None => None
                }
            }
        }
    }
}

/// An iterator of nodes to the ancestors a given node.
pub struct Ancestors<T>(Option<Node<T>>);
impl_node_iterator!(Ancestors, |node: &Node<T>| node.parent());

impl<T> Ancestors<T> {
    pub(crate) fn new(node: Node<T>) -> Self {
        Self(Some(node))
    }
}

/// An iterator of nodes to the siblings before a given node.
pub struct PrecedingSiblings<T>(Option<Node<T>>);
impl_node_iterator!(PrecedingSiblings, |node: &Node<T>| node.previous_sibling());

impl<T> PrecedingSiblings<T> {
    pub(crate) fn new(node: Node<T>) -> Self {
        Self(Some(node))
    }
}

/// An iterator of nodes to the siblings after a given node.
pub struct FollowingSiblings<T>(Option<Node<T>>);
impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling());

impl<T> FollowingSiblings<T> {
    pub(crate) fn new(node: Node<T>) -> Self {
        Self(Some(node))
    }
}

/// A double ended iterator of nodes to the children of a given node.
pub struct Children<T> {
    next: Option<Node<T>>,
    next_back: Option<Node<T>>,
}

impl<T> Children<T> {
    pub(crate) fn new(node: &Node<T>) -> Self {
        Self {
            next: node.first_child(),
            next_back: node.last_child(),
        }
    }

    // true if self.next_back's next sibling is self.next
    fn finished(&self) -> bool {
        match self.next_back {
            Some(ref next_back) => next_back.next_sibling() == self.next,
            _ => true,
        }
    }
}

impl<T> Iterator for Children<T> {
    type Item = Node<T>;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutably borrowed.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished() {
            return None;
        }

        match self.next.take() {
            Some(node) => {
                self.next = node.next_sibling();
                Some(node)
            }
            None => None
        }
    }
}

impl<T> DoubleEndedIterator for Children<T> {
    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutably borrowed.
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.finished() {
            return None;
        }

        match self.next_back.take() {
            Some(node) => {
                self.next_back = node.previous_sibling();
                Some(node)
            }
            None => None
        }
    }
}

/// An iterator of nodes to a given node and its descendants, in tree order.
pub struct Descendants<T>(Traverse<T>);

impl<T> Descendants<T> {
    pub(crate) fn new(node: Node<T>) -> Self {
        Self(Traverse::new(node))
    }
}

impl<T> Iterator for Descendants<T> {
    type Item = Node<T>;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutably borrowed.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
}


/// A node type during traverse.
#[derive(Clone, Debug)]
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node's descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(Node<T>),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node's descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(Node<T>),
}

// Implement PartialEq manually, because we do not need to require T: PartialEq
impl<T> PartialEq for NodeEdge<T> {
    fn eq(&self, other: &NodeEdge<T>) -> bool {
        match (&*self, &*other) {
            (&NodeEdge::Start(ref n1), &NodeEdge::Start(ref n2)) => *n1 == *n2,
            (&NodeEdge::End(ref n1), &NodeEdge::End(ref n2)) => *n1 == *n2,
            _ => false,
        }
    }
}

impl<T> NodeEdge<T> {
    fn next_item(&self, root: &Node<T>) -> Option<NodeEdge<T>> {
        match *self {
            NodeEdge::Start(ref node) => match node.first_child() {
                Some(first_child) => Some(NodeEdge::Start(first_child)),
                None => Some(NodeEdge::End(node.clone())),
            },
            NodeEdge::End(ref node) => {
                if *node == *root {
                    None
                } else {
                    match node.next_sibling() {
                        Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                        None => match node.parent() {
                            Some(parent) => Some(NodeEdge::End(parent)),

                            // `node.parent()` here can only be `None`
                            // if the tree has been modified during iteration,
                            // but silently stoping iteration
                            // seems a more sensible behavior than panicking.
                            None => None,
                        },
                    }
                }
            }
        }
    }

    fn previous_item(&self, root: &Node<T>) -> Option<NodeEdge<T>> {
        match *self {
            NodeEdge::End(ref node) => match node.last_child() {
                Some(last_child) => Some(NodeEdge::End(last_child)),
                None => Some(NodeEdge::Start(node.clone())),
            },
            NodeEdge::Start(ref node) => {
                if *node == *root {
                    None
                } else {
                    match node.previous_sibling() {
                        Some(previous_sibling) => Some(NodeEdge::End(previous_sibling)),
                        None => match node.parent() {
                            Some(parent) => Some(NodeEdge::Start(parent)),

                            // `node.parent()` here can only be `None`
                            // if the tree has been modified during iteration,
                            // but silently stoping iteration
                            // seems a more sensible behavior than panicking.
                            None => None
                        }
                    }
                }
            }
        }
    }
}

/// A double ended iterator of nodes to a given node and its descendants,
/// in tree order.
pub struct Traverse<T> {
    root: Node<T>,
    next: Option<NodeEdge<T>>,
    next_back: Option<NodeEdge<T>>,
}

impl<T> Traverse<T> {
    pub(crate) fn new(root: Node<T>) -> Self {
        let next = Some(NodeEdge::Start(root.clone()));
        let next_back = Some(NodeEdge::End(root.clone()));
        Self {
            root,
            next,
            next_back,
        }
    }

    // true if self.next_back's next item is self.next
    fn finished(&self) -> bool {
        match self.next_back {
            Some(ref next_back) => next_back.next_item(&self.root) == self.next,
            _ => true,
        }
    }
}

impl<T> Iterator for Traverse<T> {
    type Item = NodeEdge<T>;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutably borrowed.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished() {
            return None;
        }

        match self.next.take() {
            Some(item) => {
                self.next = item.next_item(&self.root);
                Some(item)
            }
            None => None
        }
    }
}

impl<T> DoubleEndedIterator for Traverse<T> {
    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutably borrowed.
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.finished() {
            return None;
        }

        match self.next_back.take() {
            Some(item) => {
                self.next_back = item.previous_item(&self.root);
                Some(item)
            }
            None => None
        }
    }
}
