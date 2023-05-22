use std::ops::Deref;
use std::ops::DerefMut;

use super::TreeSequence;
use crate::sys::bindings as ll_bindings;
use crate::TreeFlags;
use crate::TreeInterface;
use crate::TskitError;
use ll_bindings::tsk_tree_free;
use std::ptr::NonNull;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree<'treeseq> {
    pub(crate) inner: mbox::MBox<ll_bindings::tsk_tree_t>,
    // NOTE: this reference exists becaust tsk_tree_t
    // contains a NON-OWNING pointer to tsk_treeseq_t.
    // Thus, we could theoretically cause UB without
    // tying the rust-side object liftimes together.
    #[allow(dead_code)]
    treeseq: &'treeseq TreeSequence,
    api: TreeInterface,
    current_tree: i32,
    advanced: bool,
}

impl<'treeseq> Drop for Tree<'treeseq> {
    fn drop(&mut self) {
        // SAFETY: Mbox<_> cannot hold a NULL ptr
        let rv = unsafe { tsk_tree_free(self.inner.as_mut()) };
        assert_eq!(rv, 0);
    }
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
        ts: &'treeseq TreeSequence,
        flags: F,
    ) -> Result<Self, TskitError> {
        let flags = flags.into();

        // SAFETY: this is the type we want :)
        let temp = unsafe {
            libc::malloc(std::mem::size_of::<ll_bindings::tsk_tree_t>())
                as *mut ll_bindings::tsk_tree_t
        };

        // Get our pointer into MBox ASAP
        let nonnull = NonNull::<ll_bindings::tsk_tree_t>::new(temp)
            .ok_or_else(|| TskitError::LibraryError("failed to malloc tsk_tree_t".to_string()))?;

        // SAFETY: if temp is NULL, we have returned Err already.
        let mut tree = unsafe { mbox::MBox::from_non_null_raw(nonnull) };
        let mut rv =
            unsafe { ll_bindings::tsk_tree_init(tree.as_mut(), ts.as_ptr(), flags.bits()) };
        if rv < 0 {
            return Err(TskitError::ErrorCode { code: rv });
        }
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            rv = unsafe {
                ll_bindings::tsk_tree_set_tracked_samples(
                    tree.as_mut(),
                    ts.num_samples().into(),
                    (tree.as_mut()).samples,
                )
            };
        }

        let num_nodes = unsafe { (*(*ts.as_ptr()).tables).nodes.num_rows };
        let api = TreeInterface::new(nonnull, num_nodes, num_nodes + 1, flags);
        handle_tsk_return_value!(
            rv,
            Tree {
                inner: tree,
                treeseq: ts,
                current_tree: 0,
                advanced: false,
                api
            }
        )
    }
}

impl<'ts> streaming_iterator::StreamingIterator for Tree<'ts> {
    type Item = Tree<'ts>;
    fn advance(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_first(self.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_next(self.as_mut_ptr()) }
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
