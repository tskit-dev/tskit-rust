use crate::bindings;

use super::bindings::tsk_id_t;
use super::bindings::tsk_size_t;
use super::bindings::tsk_tree_t;
use super::flags::TreeFlags;
use super::newtypes::NodeId;
use super::tskbox::TskBox;
use super::TreeSequence;
use super::TskitError;

pub struct LLTree<'treeseq> {
    inner: TskBox<tsk_tree_t>,
    flags: TreeFlags,
    // NOTE: this reference exists becaust tsk_tree_t
    // contains a NON-OWNING pointer to tsk_treeseq_t.
    // Thus, we could theoretically cause UB without
    // tying the rust-side object liftimes together.
    #[allow(dead_code)]
    treeseq: &'treeseq TreeSequence,
}

impl<'treeseq> LLTree<'treeseq> {
    pub fn new(treeseq: &'treeseq TreeSequence, flags: TreeFlags) -> Result<Self, TskitError> {
        let mut inner = TskBox::new(|x: *mut super::bindings::tsk_tree_t| unsafe {
            super::bindings::tsk_tree_init(x, treeseq.as_ref(), flags.bits())
        })?;
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            let code = unsafe {
                super::bindings::tsk_tree_set_tracked_samples(
                    inner.as_mut(),
                    treeseq.num_samples().into(),
                    (inner.as_mut()).samples,
                )
            };
            if code < 0 {
                return Err(TskitError::ErrorCode { code });
            }
        }
        Ok(Self {
            inner,
            flags,
            treeseq,
        })
    }

    pub fn num_samples(&self) -> tsk_size_t {
        assert!(self.as_ref().tree_sequence.is_null());
        // SAFETY: tree_sequence is not NULL
        // the tree_sequence is also initialized (unless unsafe code was used previously?)
        unsafe { crate::sys::bindings::tsk_treeseq_get_num_samples(self.as_ref().tree_sequence) }
    }

    pub fn samples_array(&self) -> Result<&[super::newtypes::NodeId], TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            super::generate_slice(self.as_ref().samples, self.num_samples())
        )
    }

    /// Return the virtual root of the tree.
    pub fn virtual_root(&self) -> tsk_id_t {
        self.as_ref().virtual_root
    }

    pub fn as_mut_ptr(&mut self) -> *mut tsk_tree_t {
        self.inner.as_mut()
    }

    pub fn as_ptr(&mut self) -> *const tsk_tree_t {
        self.inner.as_ptr()
    }

    pub fn as_ref(&self) -> &tsk_tree_t {
        self.inner.as_ref()
    }

    pub fn left_sib<N: Into<tsk_id_t> + Copy>(&self, u: N) -> Option<tsk_id_t> {
        todo!("doc SAFETY");
        super::tsk_column_access::<tsk_id_t, _, _, _>(
            u.into(),
            self.as_ref().left_sib,
            unsafe {
                self.as_ref()
                    .tree_sequence
                    .as_ref()
                    .unwrap()
                    .tables
                    .as_ref()
            }
            .unwrap()
            .nodes
            .num_rows,
        )
    }

    pub fn right_child<N: Into<tsk_id_t> + Copy>(&self, u: N) -> Option<tsk_id_t> {
        todo!("doc SAFETY");
        super::tsk_column_access::<tsk_id_t, _, _, _>(
            u.into(),
            self.as_ref().right_child,
            unsafe {
                self.as_ref()
                    .tree_sequence
                    .as_ref()
                    .unwrap()
                    .tables
                    .as_ref()
            }
            .unwrap()
            .nodes
            .num_rows,
        )
    }
}

// Trait defining iteration over nodes.
pub trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<tsk_id_t>;
}

struct NodeIteratorAdapter<T>
where
    T: NodeIterator,
{
    ni: T,
}

impl<T> Iterator for NodeIteratorAdapter<T>
where
    T: NodeIterator,
{
    type Item = tsk_id_t;
    fn next(&mut self) -> Option<Self::Item> {
        self.ni.next_node();
        self.ni.current_node()
    }
}

struct PreorderNodeIterator<'a> {
    current_root: tsk_id_t,
    node_stack: Vec<tsk_id_t>,
    tree: &'a LLTree<'a>,
    current_node_: Option<tsk_id_t>,
}

impl<'a> PreorderNodeIterator<'a> {
    fn new(tree: &'a LLTree) -> Self {
        debug_assert!(tree.right_child(tree.virtual_root()).is_some());
        let mut rv = PreorderNodeIterator {
            current_root: tree.right_child(tree.virtual_root()).unwrap_or(-1),
            node_stack: vec![],
            tree,
            current_node_: None,
        };
        let mut c = rv.current_root;
        while c != -1 {
            rv.node_stack.push(c);
            debug_assert!(rv.tree.left_sib(c).is_some());
            c = rv.tree.left_sib(c).unwrap_or(-1);
        }
        rv
    }
}

impl NodeIterator for PreorderNodeIterator<'_> {
    fn next_node(&mut self) {
        self.current_node_ = self.node_stack.pop();
        if let Some(u) = self.current_node_ {
            // NOTE: process children right-to-left
            // because we later pop them from a steck
            // to generate the expected left-to-right ordering.
            debug_assert!(self.tree.right_child(u).is_some());
            let mut c = self.tree.right_child(u).unwrap_or(-1);
            while c != NodeId::NULL {
                self.node_stack.push(c);
                debug_assert!(self.tree.right_child(c).is_some());
                c = self.tree.left_sib(c).unwrap_or(-1);
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node_
    }
}
