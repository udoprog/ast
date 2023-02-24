use core::iter::FusedIterator;

use crate::links::Links;
use crate::node::{Node, SkipTokens};
use crate::pointer::Pointer;
use crate::tree::Kind;

/// An iterator that iterates over the [`Node::next`] elements of a node. This is
/// typically used for iterating over the children of a tree.
///
/// Note that this iterator also implements [Default], allowing it to
/// effectively create an empty iterator in case a particular sibling is not
/// available:
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2" => {}
///         },
///         "child3" => {}
///     }
/// };
///
/// let mut it = tree.first().and_then(|n| n.next()).map(|n| n.siblings()).unwrap_or_default();
/// assert_eq!(it.next().map(|n| *n.value()), None);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// See [`Node::siblings`].
///
/// # Examples
///
/// ```
/// let mut tree = syntree::tree! {
///     "root" => {
///         "child1" => {
///             "child2" => {}
///         },
///         "child3" => {}
///     },
///     "root2" => {
///         "child4" => {}
///     }
/// };
///
/// let root = tree.first().ok_or("missing root")?;
///
/// assert_eq!(
///     root.siblings().map(|n| *n.value()).collect::<Vec<_>>(),
///     ["root", "root2"]
/// );
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct Siblings<'a, T, I, P> {
    tree: &'a [Links<T, I, P>],
    links: Option<&'a Links<T, I, P>>,
}

impl<'a, T, I, P> Siblings<'a, T, I, P> {
    /// Construct a new child iterator.
    #[inline]
    pub(crate) const fn new(tree: &'a [Links<T, I, P>], links: &'a Links<T, I, P>) -> Self {
        Self {
            tree,
            links: Some(links),
        }
    }

    /// Construct a [`SkipTokens`] iterator from the remainder of this
    /// iterator. This filters out [`Kind::Token`] elements.
    ///
    /// See [`SkipTokens`] for documentation.
    #[must_use]
    pub const fn skip_tokens(self) -> SkipTokens<Self> {
        SkipTokens::new(self)
    }
}

impl<T, I, P> Siblings<'_, T, I, P>
where
    P: Pointer,
{
    /// Get the next node from the iterator. This advances past all non-node
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// let tree = syntree::tree! {
    ///     ("t1", 1),
    ///     "child1" => {},
    ///     ("t2", 1),
    ///     "child2" => {},
    ///     ("t3", 1),
    ///     "child3" => {},
    ///     ("t4", 1)
    /// };
    ///
    /// let first = tree.first().ok_or("missing first")?;
    ///
    /// let mut it = first.siblings();
    /// let mut out = Vec::new();
    ///
    /// while let Some(n) = it.next_node() {
    ///     out.push(*n.value());
    /// }
    ///
    /// assert_eq!(out, ["child1", "child2", "child3"]);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn next_node(&mut self) -> Option<Node<'_, T, I, P>> {
        loop {
            let node = self.next()?;

            if matches!(node.kind(), Kind::Node) {
                return Some(node);
            }
        }
    }
}

impl<'a, T, I, P> Iterator for Siblings<'a, T, I, P>
where
    P: Pointer,
{
    type Item = Node<'a, T, I, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let links = self.links.take()?;
        self.links = links.next.and_then(|id| self.tree.get(id.get()));
        Some(Node::new(links, self.tree))
    }
}

impl<T, I, P> FusedIterator for Siblings<'_, T, I, P> where P: Pointer {}

impl<T, I, P> Clone for Siblings<'_, T, I, P> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            links: self.links,
        }
    }
}

impl<T, I, P> Default for Siblings<'_, T, I, P> {
    #[inline]
    fn default() -> Self {
        Self {
            tree: &[],
            links: None,
        }
    }
}
