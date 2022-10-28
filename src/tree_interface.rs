use crate::bindings as ll_bindings;
use crate::tsk_id_t;
use crate::tsk_size_t;
use crate::NodeId;
use crate::Position;
use crate::SizeType;
use crate::Time;
use crate::TreeFlags;
use crate::TskitError;
use crate::TskitTypeAccess;
use std::ptr::NonNull;

pub struct TreeInterface {
    non_owned_pointer: NonNull<ll_bindings::tsk_tree_t>,
    num_nodes: tsk_size_t,
    array_len: tsk_size_t,
    flags: TreeFlags,
}

impl TskitTypeAccess<ll_bindings::tsk_tree_t> for TreeInterface {
    fn as_ptr(&self) -> *const ll_bindings::tsk_tree_t {
        self.non_owned_pointer.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_tree_t {
        self.non_owned_pointer.as_ptr()
    }
}

impl TreeInterface {
    pub(crate) fn new(
        non_owned_pointer: NonNull<ll_bindings::tsk_tree_t>,
        num_nodes: tsk_size_t,
        array_len: tsk_size_t,
        flags: TreeFlags,
    ) -> Self {
        Self {
            non_owned_pointer,
            num_nodes,
            array_len,
            flags,
        }
    }

    pub fn flags(&self) -> TreeFlags {
        self.flags
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let p = tree.parent_array();
    ///     drop(tree_iter);
    ///     for _ in p {} // ERROR
    /// }
    /// ```
    pub fn parent_array(&self) -> &[NodeId] {
        tree_array_slice!(self, parent, self.array_len)
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let s = tree.samples_array().unwrap();
    ///     for _ in s {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let s = tree.samples_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in s {} // ERROR
    /// }
    /// ```
    pub fn samples_array(&self) -> Result<&[NodeId], TskitError> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        err_if_not_tracking_samples!(self.flags, tree_array_slice!(self, samples, num_samples))
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let n = tree.next_sample_array().unwrap();
    ///     for _ in n {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let n = tree.next_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in n {} // ERROR
    /// }
    /// ```
    pub fn next_sample_array(&self) -> Result<&[NodeId], TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            tree_array_slice!(self, next_sample, (*self.as_ptr()).num_nodes)
        )
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // Error
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_sample_array().unwrap();
    ///     for _ in l {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in l {} // Error
    /// }
    /// ```
    pub fn left_sample_array(&self) -> Result<&[NodeId], TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            tree_array_slice!(self, left_sample, (*self.as_ptr()).num_nodes)
        )
    }

    /// # Failing examples
    ///
    /// An error will be returned if ['crate::TreeFlags::SAMPLE_LISTS`] is not used:
    ///
    /// ```should_panic
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap(); // ERROR
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sample_array().unwrap();
    ///     for _ in r {}
    /// }
    /// ```
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::SAMPLE_LISTS).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sample_array().unwrap();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_sample_array(&self) -> Result<&[NodeId], TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            tree_array_slice!(self, right_sample, (*self.as_ptr()).num_nodes)
        )
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.left_sib_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn left_sib_array(&self) -> &[NodeId] {
        tree_array_slice!(self, left_sib, self.array_len)
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_sib_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_sib_array(&self) -> &[NodeId] {
        tree_array_slice!(self, right_sib, self.array_len)
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let l = tree.left_child_array();
    ///     drop(tree_iter);
    ///     for _ in l {} // ERROR
    /// }
    /// ```
    pub fn left_child_array(&self) -> &[NodeId] {
        tree_array_slice!(self, left_child, self.array_len)
    }

    /// # Failing examples
    ///
    /// The lifetime of the slice is tied to the parent object:
    ///
    /// ```compile_fail
    /// use streaming_iterator::StreamingIterator;
    /// let tables = tskit::TableCollection::new(1.).unwrap();
    /// let treeseq =
    /// tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// let mut tree_iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iter.next() {
    ///     let r = tree.right_child_array();
    ///     drop(tree_iter);
    ///     for _ in r {} // ERROR
    /// }
    /// ```
    pub fn right_child_array(&self) -> &[NodeId] {
        tree_array_slice!(self, right_child, self.array_len)
    }

    fn left_sample(&self, u: NodeId) -> Result<NodeId, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(
                u.0,
                0,
                self.num_nodes,
                (*self.as_ptr()).left_sample,
                NodeId
            )?
        )
    }

    fn right_sample(&self, u: NodeId) -> Result<NodeId, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(
                u.0,
                0,
                self.num_nodes,
                (*self.as_ptr()).right_sample,
                NodeId
            )?
        )
    }

    /// Return the `[left, right)` coordinates of the tree.
    pub fn interval(&self) -> (Position, Position) {
        (
            unsafe { (*self.as_ptr()).interval }.left.into(),
            unsafe { (*self.as_ptr()).interval }.right.into(),
        )
    }

    /// Return the length of the genome for which this
    /// tree is the ancestry.
    pub fn span(&self) -> Position {
        let i = self.interval();
        i.1 - i.0
    }

    /// Get the parent of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn parent(&self, u: NodeId) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(u.0, 0, self.array_len, (*self.as_ptr()).parent, NodeId)
    }

    /// Get the left child of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn left_child(&self, u: NodeId) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(u.0, 0, self.array_len, (*self.as_ptr()).left_child, NodeId)
    }

    /// Get the right child of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn right_child(&self, u: NodeId) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(u.0, 0, self.array_len, (*self.as_ptr()).right_child, NodeId)
    }

    /// Get the left sib of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn left_sib(&self, u: NodeId) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(u.0, 0, self.array_len, (*self.as_ptr()).left_sib, NodeId)
    }

    /// Get the right sib of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn right_sib(&self, u: NodeId) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(u.0, 0, self.array_len, (*self.as_ptr()).right_sib, NodeId)
    }

    /// Obtain the list of samples for the current tree/tree sequence
    /// as a vector.
    ///
    /// # Panics
    ///
    /// Will panic if the number of samples is too large to cast to a valid id.
    #[deprecated(since = "0.2.3", note = "Please use Tree::sample_nodes instead")]
    pub fn samples_to_vec(&self) -> Vec<NodeId> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = match isize::try_from(i) {
                Ok(o) => unsafe { *(*(*self.as_ptr()).tree_sequence).samples.offset(o) },
                Err(e) => panic!("{}", e),
            };
            rv.push(u.into());
        }
        rv
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        tree_array_slice!(self, samples, num_samples)
    }

    /// Return an [`Iterator`] from the node `u` to the root of the tree.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    #[deprecated(since = "0.2.3", note = "Please use Tree::parents instead")]
    pub fn path_to_root(&self, u: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        self.parents(u)
    }

    /// Return an [`Iterator`] from the node `u` to the root of the tree,
    /// travering all parent nodes.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn parents(&self, u: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        ParentsIterator::new(self, u)
    }

    /// Return an [`Iterator`] over the children of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn children(&self, u: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        ChildIterator::new(self, u)
    }
    /// Return an [`Iterator`] over the sample nodes descending from node `u`.
    ///
    /// # Note
    ///
    /// If `u` is itself a sample, then it is included in the values returned.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    ///
    /// [`TskitError::NotTrackingSamples`] if [`TreeFlags::SAMPLE_LISTS`] was not used
    /// to initialize `self`.
    pub fn samples(&self, u: NodeId) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        SamplesIterator::new(self, u)
    }

    /// Return an [`Iterator`] over the roots of the tree.
    ///
    /// # Note
    ///
    /// For a tree with multiple roots, the iteration starts
    /// at the left root.
    pub fn roots(&self) -> impl Iterator<Item = NodeId> + '_ {
        RootIterator::new(self)
    }

    /// Return all roots as a vector.
    pub fn roots_to_vec(&self) -> Vec<NodeId> {
        let mut v = vec![];

        for r in self.roots() {
            v.push(r);
        }

        v
    }

    /// Return an [`Iterator`] over all nodes in the tree.
    ///
    /// # Parameters
    ///
    /// * `order`: A value from [`NodeTraversalOrder`] specifying the
    ///   iteration order.
    // Return value is dyn for later addition of other traversal orders
    pub fn traverse_nodes(
        &self,
        order: NodeTraversalOrder,
    ) -> Box<dyn Iterator<Item = NodeId> + '_> {
        match order {
            NodeTraversalOrder::Preorder => Box::new(PreorderNodeIterator::new(self)),
            NodeTraversalOrder::Postorder => Box::new(PostorderNodeIterator::new(self)),
        }
    }

    /// Return the [`crate::NodeTable`] for this current tree
    /// (and the tree sequence from which it came).
    ///
    /// This is a convenience function for accessing node times, etc..
    pub fn node_table<'a>(&'a self) -> crate::NodeTable<'a> {
        crate::NodeTable::<'a>::new_from_table(unsafe {
            &(*(*(*self.as_ptr()).tree_sequence).tables).nodes
        })
    }

    /// Calculate the total length of the tree via a preorder traversal.
    ///
    /// # Parameters
    ///
    /// * `by_span`: if `true`, multiply the return value by [`TreeInterface::span`].
    ///
    /// # Errors
    ///
    /// [`TskitError`] may be returned if a node index is out of range.
    pub fn total_branch_length(&self, by_span: bool) -> Result<Time, TskitError> {
        let nt = self.node_table();
        let mut b = Time::from(0.);
        for n in self.traverse_nodes(NodeTraversalOrder::Preorder) {
            let p = self.parent(n)?;
            if p != NodeId::NULL {
                b += nt.time(p)? - nt.time(n)?;
            }
        }

        match by_span {
            true => Ok(b * self.span()),
            false => Ok(b),
        }
    }

    /// Get the number of samples below node `u`.
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if [`TreeFlags::NO_SAMPLE_COUNTS`].
    pub fn num_tracked_samples(&self, u: NodeId) -> Result<SizeType, TskitError> {
        let mut n = SizeType(tsk_size_t::MAX);
        let np: *mut tsk_size_t = &mut n.0;
        let code = unsafe { ll_bindings::tsk_tree_get_num_tracked_samples(self.as_ptr(), u.0, np) };
        handle_tsk_return_value!(code, n)
    }

    /// Calculate the average Kendall-Colijn (`K-C`) distance between
    /// pairs of trees whose intervals overlap.
    ///
    /// # Note
    ///
    /// * [Citation](https://doi.org/10.1093/molbev/msw124)
    ///
    /// # Parameters
    ///
    /// * `lambda` specifies the relative weight of topology and branch length.
    ///   If `lambda` is 0, we only consider topology.
    ///   If `lambda` is 1, we only consider branch lengths.
    pub fn kc_distance(&self, other: &TreeInterface, lambda: f64) -> Result<f64, TskitError> {
        let mut kc = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        let code = unsafe {
            ll_bindings::tsk_tree_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        };
        handle_tsk_return_value!(code, kc)
    }

    /// Return the virtual root of the tree.
    pub fn virtual_root(&self) -> NodeId {
        unsafe { (*self.as_ptr()).virtual_root }.into()
    }
}

/// Specify the traversal order used by
/// [`TreeInterface::traverse_nodes`].
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

// Trait defining iteration over nodes.
trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<NodeId>;
}

struct PreorderNodeIterator<'a> {
    current_root: NodeId,
    node_stack: Vec<NodeId>,
    tree: &'a TreeInterface,
    current_node_: Option<NodeId>,
}

impl<'a> PreorderNodeIterator<'a> {
    fn new(tree: &'a TreeInterface) -> Self {
        debug_assert!(tree.right_child(tree.virtual_root()).is_ok());
        let mut rv = PreorderNodeIterator {
            current_root: tree
                .right_child(tree.virtual_root())
                .unwrap_or(NodeId::NULL),
            node_stack: vec![],
            tree,
            current_node_: None,
        };
        let mut c = rv.current_root;
        while !c.is_null() {
            rv.node_stack.push(c);
            debug_assert!(rv.tree.left_sib(c).is_ok());
            c = rv.tree.left_sib(c).unwrap_or(NodeId::NULL);
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
            debug_assert!(self.tree.right_child(u).is_ok());
            let mut c = self.tree.right_child(u).unwrap_or(NodeId::NULL);
            while c != NodeId::NULL {
                self.node_stack.push(c);
                debug_assert!(self.tree.right_child(c).is_ok());
                c = self.tree.left_sib(c).unwrap_or(NodeId::NULL);
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_node_
    }
}

iterator_for_nodeiterator!(PreorderNodeIterator<'_>);

struct PostorderNodeIterator<'a> {
    nodes: Vec<NodeId>,
    current_node_index: usize,
    num_nodes_current_tree: usize,
    // Make the lifetime checker happy.
    tree: std::marker::PhantomData<&'a TreeInterface>,
}

impl<'a> PostorderNodeIterator<'a> {
    fn new(tree: &'a TreeInterface) -> Self {
        let mut num_nodes_current_tree: tsk_size_t = 0;
        let ptr = std::ptr::addr_of_mut!(num_nodes_current_tree);
        let mut nodes = vec![
            NodeId::NULL;
            // NOTE: this fn does not return error codes
            usize::try_from(unsafe {
                ll_bindings::tsk_tree_get_size_bound(tree.as_ptr())
            }).unwrap_or(usize::MAX)
        ];

        let rv = unsafe {
            ll_bindings::tsk_tree_postorder(
                tree.as_ptr(),
                nodes.as_mut_ptr().cast::<tsk_id_t>(),
                ptr,
            )
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
            num_nodes_current_tree: usize::try_from(num_nodes_current_tree).unwrap_or(0),
            tree: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for PostorderNodeIterator<'a> {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current_node_index < self.num_nodes_current_tree {
            true => {
                let rv = self.nodes[self.current_node_index];
                self.current_node_index += 1;
                Some(rv)
            }
            false => None,
        }
    }
}

struct RootIterator<'a> {
    current_root: Option<NodeId>,
    next_root: NodeId,
    tree: &'a TreeInterface,
}

impl<'a> RootIterator<'a> {
    fn new(tree: &'a TreeInterface) -> Self {
        debug_assert!(tree.left_child(tree.virtual_root()).is_ok());
        RootIterator {
            current_root: None,
            next_root: tree.left_child(tree.virtual_root()).unwrap_or(NodeId::NULL),
            tree,
        }
    }
}

impl NodeIterator for RootIterator<'_> {
    fn next_node(&mut self) {
        self.current_root = match self.next_root {
            NodeId::NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                debug_assert!(self.tree.right_sib(r).is_ok());
                self.next_root = self.tree.right_sib(r).unwrap_or(NodeId::NULL);
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_root
    }
}

iterator_for_nodeiterator!(RootIterator<'_>);

struct ChildIterator<'a> {
    current_child: Option<NodeId>,
    next_child: NodeId,
    tree: &'a TreeInterface,
}

impl<'a> ChildIterator<'a> {
    fn new(tree: &'a TreeInterface, u: NodeId) -> Result<Self, TskitError> {
        let c = tree.left_child(u)?;

        Ok(ChildIterator {
            current_child: None,
            next_child: c,
            tree,
        })
    }
}

impl NodeIterator for ChildIterator<'_> {
    fn next_node(&mut self) {
        self.current_child = match self.next_child {
            NodeId::NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                debug_assert!(self.tree.right_sib(r).is_ok());
                self.next_child = self.tree.right_sib(r).unwrap_or(NodeId::NULL);
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_child
    }
}

iterator_for_nodeiterator!(ChildIterator<'_>);

struct ParentsIterator<'a> {
    current_node: Option<NodeId>,
    next_node: NodeId,
    tree: &'a TreeInterface,
}

impl<'a> ParentsIterator<'a> {
    fn new(tree: &'a TreeInterface, u: NodeId) -> Result<Self, TskitError> {
        let num_nodes = match tsk_id_t::try_from(tree.num_nodes) {
            Ok(n) => n,
            Err(_) => {
                return Err(TskitError::RangeError(format!(
                    "could not convert {} into tsk_id_t",
                    stringify!(num_nodes)
                )))
            }
        };
        match u.0 >= num_nodes {
            true => Err(TskitError::IndexError),
            false => Ok(ParentsIterator {
                current_node: None,
                next_node: u,
                tree,
            }),
        }
    }
}

impl NodeIterator for ParentsIterator<'_> {
    fn next_node(&mut self) {
        self.current_node = match self.next_node {
            NodeId::NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                debug_assert!(self.tree.parent(r).is_ok());
                self.next_node = self.tree.parent(r).unwrap_or(NodeId::NULL);
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_node
    }
}

iterator_for_nodeiterator!(ParentsIterator<'_>);

struct SamplesIterator<'a> {
    current_node: Option<NodeId>,
    next_sample_index: NodeId,
    last_sample_index: NodeId,
    tree: &'a TreeInterface,
    //next_sample: crate::ffi::TskIdArray,
    //samples: crate::ffi::TskIdArray,
}

impl<'a> SamplesIterator<'a> {
    fn new(tree: &'a TreeInterface, u: NodeId) -> Result<Self, TskitError> {
        let rv = SamplesIterator {
            current_node: None,
            next_sample_index: tree.left_sample(u)?,
            last_sample_index: tree.right_sample(u)?,
            tree,
        };

        Ok(rv)
    }
}

impl NodeIterator for SamplesIterator<'_> {
    fn next_node(&mut self) {
        self.current_node = match self.next_sample_index {
            NodeId::NULL => None,
            r => {
                if r == self.last_sample_index {
                    let cr =
                        Some(unsafe { *(*self.tree.as_ptr()).samples.offset(r.0 as isize) }.into());
                    self.next_sample_index = NodeId::NULL;
                    cr
                } else {
                    assert!(r >= 0);
                    let cr =
                        Some(unsafe { *(*self.tree.as_ptr()).samples.offset(r.0 as isize) }.into());
                    //self.next_sample_index = self.next_sample[r];
                    self.next_sample_index =
                        unsafe { *(*self.tree.as_ptr()).next_sample.offset(r.0 as isize) }.into();
                    cr
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<NodeId> {
        self.current_node
    }
}

iterator_for_nodeiterator!(SamplesIterator<'_>);
