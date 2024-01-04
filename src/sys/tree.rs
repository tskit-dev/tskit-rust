use super::bindings::tsk_tree_t;
use super::flags::TreeFlags;
use super::tskbox::TskBox;
use super::Error;
use super::LLTreeSeq;

pub struct LLTree<'treeseq> {
    inner: TskBox<tsk_tree_t>,
    // NOTE: this reference exists becaust tsk_tree_t
    // contains a NON-OWNING pointer to tsk_treeseq_t.
    // Thus, we could theoretically cause UB without
    // tying the rust-side object liftimes together.
    #[allow(dead_code)]
    treeseq: &'treeseq LLTreeSeq,
}

impl<'treeseq> LLTree<'treeseq> {
    pub fn new(treeseq: &'treeseq LLTreeSeq, flags: TreeFlags) -> Result<Self, Error> {
        let mut inner = TskBox::new(|x: *mut super::bindings::tsk_tree_t| unsafe {
            super::bindings::tsk_tree_init(x, treeseq.as_ref(), flags.bits())
        })?;
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            let rv = unsafe {
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
        self.inner.as_mut()
    }
}
