use super::bindings;

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
        let mut inner = TskBox::new(|x: *mut bindings::tsk_tree_t| unsafe {
            bindings::tsk_tree_init(x, treeseq.as_ref(), flags.bits())
        })?;
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            let code = unsafe {
                bindings::tsk_tree_set_tracked_samples(
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

    pub fn right_sib(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
            self.as_ref().right_sib,
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

    pub fn left_child(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
            self.as_ref().left_child,
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
        let code =
            unsafe { bindings::tsk_tree_get_num_tracked_samples(self.as_ptr(), u.into(), np) };
        handle_tsk_return_value!(code, n.into())
    }

    pub fn left_sample(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
            self.as_ref().left_sample,
            self.treeseq.num_nodes_raw(),
        )
    }

    pub fn right_sample(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
            self.as_ref().right_sample,
            self.treeseq.num_nodes_raw(),
        )
    }

    pub fn samples(&self, u: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        Ok(NodeIteratorAdapter(SamplesIterator::new(self, u)?))
    }

    pub fn parent(&self, u: NodeId) -> Option<NodeId> {
        super::tsk_column_access::<NodeId, _, _, _>(
            u,
            self.as_ref().parent,
            self.treeseq.num_nodes_raw() + 1,
        )
    }

    pub fn flags(&self) -> TreeFlags {
        self.flags
    }

    pub fn traverse_nodes(
        &self,
        order: NodeTraversalOrder,
    ) -> Box<dyn Iterator<Item = NodeId> + '_> {
        match order {
            NodeTraversalOrder::Preorder => {
                Box::new(NodeIteratorAdapter(PreorderNodeIterator::new(self)))
            }
            NodeTraversalOrder::Postorder => {
                Box::new(NodeIteratorAdapter(PostorderNodeIterator::new(self)))
            }
        }
    }

    pub fn children(&self, u: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        NodeIteratorAdapter(ChildIterator::new(self, u))
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

struct PostorderNodeIterator<'a> {
    nodes: Vec<NodeId>,
    current_node_index: usize,
    current_node: Option<NodeId>,
    num_nodes_current_tree: usize,
    tree: &'a LLTree<'a>,
}

impl<'a> PostorderNodeIterator<'a> {
    fn new(tree: &'a LLTree<'a>) -> Self {
        let mut num_nodes_current_tree: tsk_size_t = 0;
        let ptr = std::ptr::addr_of_mut!(num_nodes_current_tree);
        let mut nodes = vec![
            NodeId::NULL;
            // NOTE: this fn does not return error codes
            usize::try_from(unsafe {
                bindings::tsk_tree_get_size_bound(tree.as_ptr())
            }).unwrap_or(usize::MAX)
        ];

        let rv = unsafe {
            bindings::tsk_tree_postorder(tree.as_ptr(), nodes.as_mut_ptr().cast::<tsk_id_t>(), ptr)
        };

        // This is either out of memory
        // or node out of range.
        // The former is fatal, and the latter
        // not relevant (for now), as we start at roots.
        if rv < 0 {
            panic!("fatal error calculating postoder node list");
        }

        Self {
            nodes,
            current_node_index: 0,
            current_node: None,
            num_nodes_current_tree: usize::try_from(num_nodes_current_tree).unwrap_or(0),
            tree,
        }
    }
}

impl NodeIterator for PostorderNodeIterator<'_> {
    fn next_node(&mut self) {
        match self.current_node_index < self.num_nodes_current_tree {
            true => {
                self.current_node = Some(self.nodes[self.current_node_index]);
                self.current_node_index += 1;
            }
            false => self.current_node = None,
        }
    }

    fn current_node(&mut self) -> Option<NodeId> {
        todo!()
    }
}

struct SamplesIterator<'a> {
    current_node: Option<NodeId>,
    next_sample_index: NodeId,
    last_sample_index: NodeId,
    tree: &'a LLTree<'a>,
}

impl<'a> SamplesIterator<'a> {
    fn new(tree: &'a LLTree<'a>, u: NodeId) -> Result<Self, TskitError> {
        match tree.flags.contains(TreeFlags::SAMPLE_LISTS) {
            false => Err(TskitError::NotTrackingSamples {}),
            true => {
                let next_sample_index = match tree.left_sample(u) {
                    Some(x) => x,
                    None => NodeId::NULL,
                };
                let last_sample_index = match tree.right_sample(u) {
                    Some(x) => x,
                    None => NodeId::NULL,
                };
                Ok(SamplesIterator {
                    current_node: None,
                    next_sample_index,
                    last_sample_index,
                    tree,
                })
            }
        }
    }
}

impl NodeIterator for SamplesIterator<'_> {
    fn next_node(&mut self) {
        self.current_node = match self.next_sample_index {
            NodeId::NULL => None,
            r => {
                let raw = crate::sys::bindings::tsk_id_t::from(r);
                if r == self.last_sample_index {
                    let cr =
                        Some(unsafe { *(*self.tree.as_ptr()).samples.offset(raw as isize) }.into());
                    self.next_sample_index = NodeId::NULL;
                    cr
                } else {
                    assert!(r >= 0);
                    let cr =
                        Some(unsafe { *(*self.tree.as_ptr()).samples.offset(raw as isize) }.into());
                    //self.next_sample_index = self.next_sample[r];
                    self.next_sample_index =
                        unsafe { *(*self.tree.as_ptr()).next_sample.offset(raw as isize) }.into();
                    cr
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_node
    }
}

#[non_exhaustive]
pub enum NodeTraversalOrder {
    ///Preorder traversal, starting at the root(s) of a [`TreeInterface`].
    ///For trees with multiple roots, start at the left root,
    ///traverse to tips, proceeed to the next root, etc..
    Preorder,
    ///Postorder traversal, starting at the root(s) of a [`TreeInterface`].
    ///For trees with multiple roots, start at the left root,
    ///traverse to tips, proceeed to the next root, etc..
    Postorder,
}

struct ChildIterator<'a> {
    current_child: Option<NodeId>,
    next_child: NodeId,
    tree: &'a LLTree<'a>,
}

impl<'a> ChildIterator<'a> {
    fn new(tree: &'a LLTree<'a>, u: NodeId) -> Self {
        let c = match tree.left_child(u) {
            Some(x) => x,
            None => NodeId::NULL,
        };

        ChildIterator {
            current_child: None,
            next_child: c,
            tree,
        }
    }
}

impl NodeIterator for ChildIterator<'_> {
    fn next_node(&mut self) {
        self.current_child = match self.next_child {
            NodeId::NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                debug_assert!(self.tree.right_sib(r).is_some());
                self.next_child = self.tree.right_sib(r).unwrap_or(NodeId::NULL);
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_child
    }
}
