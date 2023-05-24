use crate::sys;
use crate::NodeId;
use crate::Position;
use crate::SizeType;
use crate::Time;
use crate::TreeFlags;
use crate::TskitError;
use ll_bindings::tsk_id_t;
use ll_bindings::tsk_size_t;
use std::ptr::NonNull;
use sys::bindings as ll_bindings;

pub struct TreeInterface {
    non_owned_pointer: NonNull<ll_bindings::tsk_tree_t>,
    num_nodes: tsk_size_t,
    array_len: tsk_size_t,
    flags: TreeFlags,
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

    fn as_ref(&self) -> &ll_bindings::tsk_tree_t {
        // SAFETY: we have already successfuly constructed
        // the NonNull
        unsafe { self.non_owned_pointer.as_ref() }
    }

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_tree_t {
        self.non_owned_pointer.as_ptr()
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_tree_t {
        self.non_owned_pointer.as_ptr()
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
        sys::generate_slice(self.as_ref().parent, self.array_len)
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
            unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ref().tree_sequence) };
        err_if_not_tracking_samples!(
            self.flags,
            sys::generate_slice(self.as_ref().samples, num_samples)
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
            sys::generate_slice(self.as_ref().next_sample, self.array_len)
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
            sys::generate_slice(self.as_ref().left_sample, self.array_len)
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
            sys::generate_slice(self.as_ref().right_sample, self.array_len)
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
        sys::generate_slice(self.as_ref().left_sib, self.array_len)
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
        sys::generate_slice(self.as_ref().right_sib, self.array_len)
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
        sys::generate_slice(self.as_ref().left_child, self.array_len)
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
        sys::generate_slice(self.as_ref().right_child, self.array_len)
    }

    // error if we are not tracking samples,
    // Ok(None) if u is out of range
    fn left_sample<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(
            u.into(),
            self.as_ref().left_sample,
            self.num_nodes,
        )
    }

    // error if we are not tracking samples,
    // Ok(None) if u is out of range
    fn right_sample<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(
            u.into(),
            self.as_ref().right_sample,
            self.num_nodes,
        )
    }

    /// Return the `[left, right)` coordinates of the tree.
    pub fn interval(&self) -> (Position, Position) {
        (
            self.as_ref().interval.left.into(),
            self.as_ref().interval.right.into(),
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
    /// Returns `None` if `u` is out of range.
    pub fn parent<N: Into<NodeId> + Copy + std::fmt::Debug>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(u.into(), self.as_ref().parent, self.array_len)
    }

    /// Get the left child of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn left_child<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(
            u.into(),
            self.as_ref().left_child,
            self.array_len,
        )
    }

    /// Get the right child of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn right_child<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(
            u.into(),
            self.as_ref().right_child,
            self.array_len,
        )
    }

    /// Get the left sib of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn left_sib<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(u.into(), self.as_ref().left_sib, self.array_len)
    }

    /// Get the right sib of node `u`.
    ///
    /// Returns `None` if `u` is out of range.
    pub fn right_sib<N: Into<NodeId> + Copy>(&self, u: N) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(u.into(), self.as_ref().right_sib, self.array_len)
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
            unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ref().tree_sequence) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = match isize::try_from(i) {
                Ok(o) => unsafe { *(*(self.as_ref()).tree_sequence).samples.offset(o) },
                Err(e) => panic!("{}", e),
            };
            rv.push(u.into());
        }
        rv
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ref().tree_sequence) };
        sys::generate_slice(self.as_ref().samples, num_samples)
    }

    /// Return an [`Iterator`] from the node `u` to the root of the tree,
    /// travering all parent nodes.
    ///
    /// # Returns
    ///
    /// * `Some(iterator)` if `u` is valid
    /// * `None` otherwise
    pub fn parents<N: Into<NodeId> + Copy>(&self, u: N) -> impl Iterator<Item = NodeId> + '_ {
        ParentsIterator::new(self, u.into())
    }

    /// Return an [`Iterator`] over the children of node `u`.
    /// # Returns
    ///
    /// * `Some(iterator)` if `u` is valid
    /// * `None` otherwise
    pub fn children<N: Into<NodeId> + Copy>(&self, u: N) -> impl Iterator<Item = NodeId> + '_ {
        ChildIterator::new(self, u.into())
    }

    /// Return an [`Iterator`] over the sample nodes descending from node `u`.
    ///
    /// # Note
    ///
    /// If `u` is itself a sample, then it is included in the values returned.
    ///
    /// # Returns
    ///
    /// * Some(Ok(iterator)) if [`TreeFlags::SAMPLE_LISTS`] is in [`TreeInterface::flags`]
    /// * Some(Err(_)) if [`TreeFlags::SAMPLE_LISTS`] is not in [`TreeInterface::flags`]
    /// * None if `u` is not valid.
    pub fn samples<N: Into<NodeId> + Copy>(
        &self,
        u: N,
    ) -> Result<impl Iterator<Item = NodeId> + '_, TskitError> {
        SamplesIterator::new(self, u.into())
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
    //    fn node_table(&self) -> &crate::NodeTable {
    //       &self.nodes
    //  }

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
        let time: &[Time] = sys::generate_slice(
            unsafe {
                (*(*(*self.non_owned_pointer.as_ptr()).tree_sequence).tables)
                    .nodes
                    .time
            },
            self.num_nodes,
        );
        let mut b = Time::from(0.);
        for n in self.traverse_nodes(NodeTraversalOrder::Preorder) {
            let p = self.parent(n).ok_or(TskitError::IndexError {})?;
            if p != NodeId::NULL {
                b += time[p.as_usize()] - time[n.as_usize()]
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
    pub fn num_tracked_samples<N: Into<NodeId> + Copy>(
        &self,
        u: N,
    ) -> Result<SizeType, TskitError> {
        let mut n = tsk_size_t::MAX;
        let np: *mut tsk_size_t = &mut n;
        let code = unsafe {
            ll_bindings::tsk_tree_get_num_tracked_samples(self.as_ptr(), u.into().into(), np)
        };
        handle_tsk_return_value!(code, n.into())
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
        self.as_ref().virtual_root.into()
    }
}

/// Specify the traversal order used by
/// [`TreeInterface::traverse_nodes`].
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
        debug_assert!(tree.right_child(tree.virtual_root()).is_some());
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
            debug_assert!(rv.tree.left_sib(c).is_some());
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
        debug_assert!(tree.left_child(tree.virtual_root()).is_some());
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
                debug_assert!(self.tree.right_sib(r).is_some());
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
    fn new(tree: &'a TreeInterface, u: NodeId) -> Self {
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

iterator_for_nodeiterator!(ChildIterator<'_>);

struct ParentsIterator<'a> {
    current_node: Option<NodeId>,
    next_node: NodeId,
    tree: &'a TreeInterface,
}

impl<'a> ParentsIterator<'a> {
    fn new(tree: &'a TreeInterface, u: NodeId) -> Self {
        let u = match tsk_id_t::try_from(tree.num_nodes) {
            Ok(num_nodes) if u < num_nodes => u,
            _ => NodeId::NULL,
        };
        ParentsIterator {
            current_node: None,
            next_node: u,
            tree,
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

iterator_for_nodeiterator!(SamplesIterator<'_>);
