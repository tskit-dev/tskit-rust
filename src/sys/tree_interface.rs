use super::bindings::tsk_id_t;
use super::bindings::tsk_tree_t;
use super::bindings::tsk_treeseq_t;

use std::ptr::NonNull;

pub struct TreeInterface {
    tree: NonNull<tsk_tree_t>,
    treeseq: NonNull<tsk_treeseq_t>,
}

impl TreeInterface {
    /// # Safety
    ///
    /// `tree` must be properly initialized
    pub unsafe fn new_from_initialized_tree(tree: &mut tsk_tree_t) -> Option<Self> {
        let mut tree = NonNull::new(tree)?;
        // SAFETY: tree is not NULL
        // If it is also initialized, we can "borrow" the pointer to the
        // tree sequence.
        let treeseq = NonNull::new(unsafe { tree.as_mut() }.tree_sequence as *mut tsk_treeseq_t)?;

        Some(Self { tree, treeseq })
    }

    fn root_iterator(&self) -> impl Iterator<Item = tsk_id_t> + '_ {
        RootIterator {
            current_root: None,
            next_root: -1,
            tree: self,
        }
    }
}

// Trait defining iteration over nodes.
trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<tsk_id_t>;
}

struct RootIterator<'a> {
    current_root: Option<tsk_id_t>,
    next_root: tsk_id_t,
    tree: &'a TreeInterface,
}

impl<'a> Iterator for RootIterator<'a> {
    type Item = tsk_id_t;
    fn next(&mut self) -> Option<Self::Item> {
        Some(-1)
    }
}
