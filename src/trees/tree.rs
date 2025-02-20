use crate::sys::bindings as ll_bindings;
use crate::sys::{LLTree, TreeSequence};
use crate::NodeId;
use crate::Position;
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
        self.inner.virtual_root().into()
    }

    /// Get the left sib of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn left_sib<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        self.inner.left_sib(u.into()).map(|x| x.into())
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
