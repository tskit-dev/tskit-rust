use std::ops::Deref;
use std::ops::DerefMut;

use crate::sys::bindings as ll_bindings;
use crate::sys::{LLTree, LLTreeSeq};
use crate::TreeFlags;
use crate::TreeInterface;
use crate::TskitError;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree<'treeseq> {
    pub(crate) inner: LLTree<'treeseq>,
    api: TreeInterface,
    current_tree: i32,
    advanced: bool,
}

impl<'treeseq> Deref for Tree<'treeseq> {
    type Target = TreeInterface;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl<'treeseq> DerefMut for Tree<'treeseq> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}

impl<'treeseq> Tree<'treeseq> {
    pub(crate) fn new<F: Into<TreeFlags>>(
        ts: &'treeseq LLTreeSeq,
        flags: F,
    ) -> Result<Self, TskitError> {
        let flags = flags.into();
        let mut inner = LLTree::new(ts, flags)?;
        let nonnull = std::ptr::NonNull::new(inner.as_mut_ptr()).unwrap();
        let num_nodes = ts.num_nodes_raw();
        let api = TreeInterface::new(nonnull, num_nodes, num_nodes + 1, flags);
        Ok(Self {
            inner,
            current_tree: 0,
            advanced: false,
            api,
        })
    }
}

impl<'ts> streaming_iterator::StreamingIterator for Tree<'ts> {
    type Item = Tree<'ts>;
    fn advance(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_first(self.inner.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_next(self.inner.as_mut_ptr()) }
        };
        if rv == 0 {
            self.advanced = false;
            self.current_tree += 1;
        } else if rv == 1 {
            self.advanced = true;
            self.current_tree += 1;
        } else if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match self.advanced {
            true => Some(self),
            false => None,
        }
    }
}

impl<'ts> streaming_iterator::DoubleEndedStreamingIterator for Tree<'ts> {
    fn advance_back(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_last(self.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_prev(self.as_mut_ptr()) }
        };
        if rv == 0 {
            self.advanced = false;
            self.current_tree -= 1;
        } else if rv == 1 {
            self.advanced = true;
            self.current_tree -= 1;
        } else if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }
}
