use crate::sys::bindings as ll_bindings;
use crate::sys::{LLTree, TreeSequence};
use crate::Position;
use crate::TreeFlags;
use crate::TreeInterface;
use crate::TskitError;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree<'treeseq> {
    pub(crate) inner: LLTree<'treeseq>,
    api: TreeInterface,
    advanced: i32,
}

impl<'treeseq> Tree<'treeseq> {
    pub(crate) fn new<F: Into<TreeFlags>>(
        ts: &'treeseq crate::sys::TreeSequence,
        flags: F,
    ) -> Result<Self, TskitError> {
        let flags = flags.into();
        let mut inner = LLTree::new(ts, flags)?;
        let nonnull = std::ptr::NonNull::new(inner.as_mut_ptr()).unwrap();
        let num_nodes = ts.num_nodes_raw();
        let api = TreeInterface::new(nonnull, num_nodes, num_nodes + 1, flags);
        Ok(Self {
            inner,
            advanced: 0,
            api,
        })
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
