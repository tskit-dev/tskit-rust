use super::bindings::tsk_id_t;
use super::bindings::tsk_tree_t;
use super::bindings::tsk_treeseq_t;

use std::ptr::NonNull;

// NOTE: this module should be all standalone functions.

struct RootIterator {
    current_root: Option<tsk_id_t>,
    next_root: tsk_id_t,
    tree: NonNull<tsk_tree_t>
}
