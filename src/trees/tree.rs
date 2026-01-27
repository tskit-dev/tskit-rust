use crate::sys::bindings as ll_bindings;
use crate::sys::{LLTree, TreeSequence};
use crate::NodeId;
use crate::Position;
use crate::SizeType;
use crate::Time;
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
        self.inner.interval()
    }

    /// Calculate the total length of the tree via a preorder traversal.
    ///
    /// # Parameters
    ///
    /// * `by_span`: if `true`, multiply the return value by [`Tree::span`].
    ///
    /// # Errors
    ///
    /// [`TskitError`] may be returned if a node index is out of range.
    pub fn total_branch_length(&self, by_span: bool) -> Result<Time, TskitError> {
        self.inner.total_branch_length(by_span)
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use tskit::StreamingIterator;
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
    /// use tskit::StreamingIterator;
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

    /// Return the length of the genome for which this
    /// tree is the ancestry.
    pub fn span(&self) -> Position {
        let i = self.interval();
        i.1 - i.0
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
    /// use tskit::StreamingIterator;
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
    /// use tskit::StreamingIterator;
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
    /// use tskit::StreamingIterator;
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
    /// use tskit::StreamingIterator;
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
    /// use tskit::StreamingIterator;
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
    /// * `order`: A value from [`crate::NodeTraversalOrder`] specifying the
    ///   iteration order.
    // Return value is dyn for later addition of other traversal orders
    pub fn traverse_nodes(
        &self,
        order: crate::NodeTraversalOrder,
    ) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.inner.traverse_nodes(order)
    }

    pub fn nodes(
        &self,
        order: crate::NodeTraversalOrder,
    ) -> Result<Box<[NodeId]>, TskitError> {
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

    pub fn as_ll_ref(&self) -> &ll_bindings::tsk_tree_t {
        self.inner.as_ll_ref()
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let n = tree.next_sample_array().unwrap();
    ///     for _ in n {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let n = tree.next_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in n {} // ERROR
    /// }
    /// ```
    pub fn next_sample_array(&self) -> Result<&[NodeId], TskitError> {
        self.inner.next_sample_array()
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // Error
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_sample_array().unwrap();
    ///     for _ in l {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in l {} // Error
    /// }
    /// ```
    pub fn left_sample_array(&self) -> Result<&[NodeId], TskitError> {
        self.inner.left_sample_array()
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sample_array().unwrap();
    ///     for _ in r {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use tskit::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_sample_array(&self) -> Result<&[NodeId], TskitError> {
        self.inner.right_sample_array()
    }

    /// Calculate the average Kendall-Colijn (`K-C`) distance between
    /// pairs of trees whose intervals overlap.
    ///
    /// # Note
    ///
    /// * [Citation](https://doi.org/10.1093/molbev/msw124)
    ///
    /// # Parameters
    ///
    /// * `lambda` specifies the relative weight of topology and branch length.
    ///   If `lambda` is 0, we only consider topology.
    ///   If `lambda` is 1, we only consider branch lengths.
    pub fn kc_distance(&self, other: &Self, lambda: f64) -> Result<f64, TskitError> {
        self.inner.kc_distance(&other.inner, lambda)
    }
}

impl<'ts> crate::StreamingIterator for Tree<'ts> {
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

impl crate::DoubleEndedStreamingIterator for Tree<'_> {
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
