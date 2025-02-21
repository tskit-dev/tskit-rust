use crate::bindings;

use super::bindings::tsk_id_t;
use super::bindings::tsk_size_t;
use super::bindings::tsk_tree_t;
use super::flags::TreeFlags;
use super::newtypes::NodeId;
use super::newtypes::SizeType;
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
    pub fn virtual_root(&self) -> NodeId {
        self.as_ref().virtual_root.into()
    }

    pub fn as_mut_ptr(&mut self) -> *mut tsk_tree_t {
        self.inner.as_mut()
    }

    pub fn as_ptr(&self) -> *const tsk_tree_t {
        self.inner.as_ptr()
    }

    pub fn as_ref(&self) -> &tsk_tree_t {
        self.inner.as_ref()
    }

    pub fn left_sib(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
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

    pub fn right_child(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
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

    pub fn num_tracked_samples(&self, u: NodeId) -> Result<SizeType, TskitError> {
        let mut n = tsk_size_t::MAX;
        let np: *mut tsk_size_t = &mut n;
        assert!(!self.as_ptr().is_null());
        // SAFETY: internal pointer not null and is initialized.
        let code = unsafe {
            super::bindings::tsk_tree_get_num_tracked_samples(self.as_ptr(), u.into(), np)
        };
        handle_tsk_return_value!(code, n.into())
    }
}

// Trait defining iteration over nodes.
pub trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<NodeId>;
}

#[repr(transparent)]
struct NodeIteratorAdapter<T: NodeIterator>(T);

impl<T> Iterator for NodeIteratorAdapter<T>
where
    T: NodeIterator,
{
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_node();
        self.0.current_node()
    }
}

struct PreorderNodeIterator<'a> {
    current_root: NodeId,
    node_stack: Vec<NodeId>,
    tree: &'a LLTree<'a>,
    current_node: Option<NodeId>,
}

impl<'a> PreorderNodeIterator<'a> {
    fn new(tree: &'a LLTree) -> Self {
        debug_assert!(tree.right_child(tree.virtual_root()).is_some());
        let mut rv = PreorderNodeIterator {
            current_root: tree
                .right_child(tree.virtual_root())
                .unwrap_or(NodeId::NULL),
            node_stack: vec![],
            tree,
            current_node: None,
        };
        let mut c = rv.current_root;
        while c != -1 {
            rv.node_stack.push(c);
            debug_assert!(rv.tree.left_sib(c).is_some());
            c = rv.tree.left_sib(c).unwrap_or(NodeId::NULL);
        }
        rv
    }
}

impl NodeIterator for PreorderNodeIterator<'_> {
    fn next_node(&mut self) {
        self.current_node = self.node_stack.pop();
        if let Some(u) = self.current_node {
            // NOTE: process children right-to-left
            // because we later pop them from a steck
            // to generate the expected left-to-right ordering.
            debug_assert!(self.tree.right_child(u).is_some());
            let mut c = self.tree.right_child(u).unwrap_or(NodeId::NULL);
            while c != NodeId::NULL {
                self.node_stack.push(c);
                debug_assert!(self.tree.right_child(c).is_some());
                c = self.tree.left_sib(c).unwrap_or(NodeId::NULL);
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_node
    }
}
