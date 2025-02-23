use crate::sys::bindings as ll_bindings;
use crate::sys::{LLTree, TreeSequence};
use crate::NodeId;
use crate::Position;
use crate::SizeType;
use crate::TreeFlags;
use crate::TskitError;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree<'treeseq> {
    pub(crate) inner: LLTree<'treeseq>,
    advanced: i32,
}

impl<'treeseq> Tree<'treeseq> {
    pub(crate) fn new<F: Into<TreeFlags>>(
        ts: &'treeseq crate::sys::TreeSequence,
        flags: F,
    ) -> Result<Self, TskitError> {
        let flags = flags.into();
        let inner = LLTree::new(ts, flags)?;
        Ok(Self { inner, advanced: 0 })
    }

    pub(crate) fn new_at_position<F: Into<TreeFlags>, P: Into<Position>>(
        ts: &'treeseq crate::sys::TreeSequence,
        flags: F,
        at: P,
    ) -> Result<Self, TskitError> {
        let mut tree = Self::new(ts, flags)?;
        assert!(!tree.inner.as_ptr().is_null());
        assert_eq!(unsafe { (*tree.inner.as_ptr()).index }, -1);
        // SAFETY: tree is initialized and the pointer is not NULL
        match unsafe { ll_bindings::tsk_tree_seek(tree.inner.as_mut_ptr(), at.into().into(), 0) } {
            code if code < 0 => return Err(TskitError::ErrorCode { code }),
            _ => (),
        };
        Ok(tree)
    }

    pub(crate) fn new_at_index<F: Into<TreeFlags>>(
        ts: &'treeseq TreeSequence,
        flags: F,
        at: i32,
    ) -> Result<Self, TskitError> {
        let mut tree = Self::new(ts, flags)?;
        assert!(!tree.inner.as_ptr().is_null());
        assert_eq!(unsafe { (*tree.inner.as_ptr()).index }, -1);
        // SAFETY: tree is initialized and the pointer is not NULL
        match unsafe { ll_bindings::tsk_tree_seek_index(tree.inner.as_mut_ptr(), at, 0) } {
            code if code < 0 => return Err(TskitError::ErrorCode { code }),
            _ => (),
        };
        Ok(tree)
    }

    /// Return the `[left, right)` coordinates of the tree.
    pub fn interval(&self) -> (Position, Position) {
        (
            self.inner.as_ref().interval.left.into(),
            self.inner.as_ref().interval.right.into(),
        )
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let s = tree.samples_array().unwrap();
    ///     for _ in s {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let s = tree.samples_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in s {} // ERROR
    /// }
    /// ```
    pub fn samples_array(&self) -> Result<&[NodeId], TskitError> {
        self.inner.samples_array()
    }

    /// Return the virtual root of the tree.
    pub fn virtual_root(&self) -> NodeId {
        self.inner.virtual_root()
    }

    /// Get the left sib of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn left_sib<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        self.inner.left_sib(u.into())
    }

    /// Get the right sib of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn right_sib<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        self.inner.right_sib(u.into())
    }

    /// Get the right child of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn right_child<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        self.inner.right_child(u.into())
    }

    /// Get the left child of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn left_child<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        self.inner.left_child(u.into())
    }

    /// Get the number of samples below node `u`.
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if [`TreeFlags::NO_SAMPLE_COUNTS`].
    pub fn num_tracked_samples<N: Into<NodeId> + Copy>(
        &self,
        u: N,
    ) -> Result<SizeType, TskitError> {
        self.inner.num_tracked_samples(u.into())
    }

    pub fn samples<N: Into<NodeId> + Copy>(
        &self,
        u: N,
    ) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        self.inner.samples(u.into())
    }

    pub fn flags(&self) -> TreeFlags {
        self.inner.flags()
    }

    /// Get the parent of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn parent<N: Into<NodeId> + Copy + std::fmt::Debug>(&self, u: N) -> Option<NodeId> {
        self.inner.parent(u.into())
    }

    /// Return an [`Iterator`] over the roots of the tree.
    ///
    /// # Note
    ///
    /// For a tree with multiple roots, the iteration starts
    /// at the left root.
    pub fn roots(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.inner.roots()
    }

    /// Return all roots as a vector.
    pub fn roots_to_vec(&self) -> Vec<NodeId> {
        self.roots().collect::<Vec<_>>()
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        self.inner.sample_nodes()
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.left_sib_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn left_sib_array(&self) -> &[NodeId] {
        self.inner.left_sib_array()
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sib_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_sib_array(&self) -> &[NodeId] {
        self.inner.right_sib_array()
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_child_array();
    ///     drop(tree_iter);
    ///     for _ in l {} // ERROR
    /// }
    /// ```
    pub fn left_child_array(&self) -> &[NodeId] {
        self.inner.left_child_array()
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_child_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_child_array(&self) -> &[NodeId] {
        self.inner.right_child_array()
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let p = tree.parent_array();
    ///     drop(tree_iter);
    ///     for _ in p {} // ERROR
    /// }
    /// ```
    pub fn parent_array(&self) -> &[NodeId] {
        self.inner.parent_array()
    }

    /// Return an [`Iterator`] over all nodes in the tree.
    ///
    /// # Parameters
    ///
    /// * `order`: A value from [`NodeTraversalOrder`] specifying the
    ///   iteration order.
    // Return value is dyn for later addition of other traversal orders
    pub fn traverse_nodes(
        &self,
        order: crate::NodeTraversalOrder,
    ) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.inner.traverse_nodes(order)
    }

    /// Return an [`Iterator`] over the children of node `u`.
    /// # Returns
    ///
    /// * `Some(iterator)` if `u` is valid
    /// * `None` otherwise
    pub fn children<N: Into<NodeId> + Copy>(&self, u: N) -> impl Iterator<Item = NodeId> + '_ {
        self.inner.children(u.into())
    }

    /// Return an [`Iterator`] over the parents of node `u`.
    /// # Returns
    ///
    /// * `Some(iterator)` if `u` is valid
    /// * `None` otherwise
    pub fn parents<N: Into<NodeId> + Copy>(&self, u: N) -> impl Iterator<Item = NodeId> + '_ {
        self.inner.parents(u.into())
    }
}

impl<'ts> streaming_iterator::StreamingIterator for Tree<'ts> {
    type Item = Tree<'ts>;
    fn advance(&mut self) {
        assert!(!self.inner.as_ptr().is_null());
        // SAFETY: pointer is not null.
        // We also know it is initialized b/c
        // it comes from LLTree
        let rv = if unsafe { *self.inner.as_ptr() }.index == -1 {
            unsafe { ll_bindings::tsk_tree_first(self.inner.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_next(self.inner.as_mut_ptr()) }
        };
        self.advanced = rv;
        if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match self.advanced == (ll_bindings::TSK_TREE_OK as i32) {
            true => Some(self),
            false => None,
        }
    }
}

impl streaming_iterator::DoubleEndedStreamingIterator for Tree<'_> {
    fn advance_back(&mut self) {
        assert!(!self.inner.as_ptr().is_null());
        // SAFETY: pointer is not null.
        // We also know it is initialized b/c
        // it comes from LLTree
        let rv = if unsafe { *self.inner.as_ptr() }.index == -1 {
            unsafe { ll_bindings::tsk_tree_last(self.inner.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_prev(self.inner.as_mut_ptr()) }
        };
        self.advanced = rv;
        if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }
}
