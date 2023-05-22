use std::ptr::NonNull;

use mbox::MBox;

use super::bindings::tsk_tree_t;
use super::flags::TreeFlags;
use super::Error;
use super::LLTreeSeq;

pub struct LLTree<'treeseq> {
    inner: MBox<tsk_tree_t>,
    // NOTE: this reference exists becaust tsk_tree_t
    // contains a NON-OWNING pointer to tsk_treeseq_t.
    // Thus, we could theoretically cause UB without
    // tying the rust-side object liftimes together.
    #[allow(dead_code)]
    treeseq: &'treeseq LLTreeSeq,
}

impl<'treeseq> LLTree<'treeseq> {
    pub fn new(treeseq: &'treeseq LLTreeSeq, flags: TreeFlags) -> Result<Self, Error> {
        // SAFETY: this is the type we want :)
        let temp = unsafe {
            libc::malloc(std::mem::size_of::<super::bindings::tsk_tree_t>())
                as *mut super::bindings::tsk_tree_t
        };

        // Get our pointer into MBox ASAP
        let nonnull = NonNull::<super::bindings::tsk_tree_t>::new(temp)
            .ok_or_else(|| Error::Message("failed to malloc tsk_tree_t".to_string()))?;

        // SAFETY: if temp is NULL, we have returned Err already.
        let mut inner = unsafe { mbox::MBox::from_non_null_raw(nonnull) };
        let mut rv = unsafe {
            super::bindings::tsk_tree_init(inner.as_mut(), treeseq.as_ptr(), flags.bits())
        };
        if rv < 0 {
            return Err(Error::Code(rv));
        }
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            rv = unsafe {
                super::bindings::tsk_tree_set_tracked_samples(
                    inner.as_mut(),
                    treeseq.num_samples(),
                    (inner.as_mut()).samples,
                )
            };
            if rv < 0 {
                return Err(Error::Code(rv));
            }
        }
        Ok(Self { inner, treeseq })
    }

    pub fn as_mut_ptr(&mut self) -> *mut tsk_tree_t {
        MBox::<tsk_tree_t>::as_mut_ptr(&mut self.inner)
    }
}

impl<'treeseq> Drop for LLTree<'treeseq> {
    fn drop(&mut self) {
        // SAFETY: Mbox<_> cannot hold a NULL ptr
        let rv = unsafe { super::bindings::tsk_tree_free(self.inner.as_mut()) };
        assert_eq!(rv, 0);
    }
}
