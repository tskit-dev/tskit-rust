use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::WrapTskitType;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeIterator;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SimplificationOptions;
use crate::SiteTable;
use crate::TableAccess;
use crate::TableOutputOptions;
use crate::TreeFlags;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::TskitTypeAccess;
use crate::{tsk_id_t, tsk_size_t, TableCollection, TSK_NULL};
use ll_bindings::{tsk_tree_free, tsk_treeseq_free};

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree {
    inner: Box<ll_bindings::tsk_tree_t>,
    current_tree: i32,
    advanced: bool,
    num_nodes: tsk_size_t,
    flags: TreeFlags,
}

pub type BoxedNodeIterator = Box<dyn NodeIterator>;

drop_for_tskit_type!(Tree, tsk_tree_free);
tskit_type_access!(Tree, ll_bindings::tsk_tree_t);

impl Tree {
    fn wrap(num_nodes: tsk_size_t, flags: TreeFlags) -> Self {
        let temp: std::mem::MaybeUninit<ll_bindings::tsk_tree_t> = std::mem::MaybeUninit::uninit();
        Self {
            inner: unsafe { Box::<ll_bindings::tsk_tree_t>::new(temp.assume_init()) },
            current_tree: 0,
            advanced: false,
            num_nodes,
            flags,
        }
    }

    fn new(ts: &TreeSequence, flags: TreeFlags) -> Result<Self, TskitError> {
        let mut tree = Self::wrap(unsafe { (*(*ts.inner).tables).nodes.num_rows }, flags);
        let mut rv =
            unsafe { ll_bindings::tsk_tree_init(tree.as_mut_ptr(), ts.as_ptr(), flags.bits()) };
        if rv < 0 {
            return Err(TskitError::ErrorCode { code: rv });
        }
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            rv = unsafe {
                ll_bindings::tsk_tree_set_tracked_samples(
                    tree.as_mut_ptr(),
                    ts.num_samples() as u64,
                    tree.inner.samples,
                )
            };
        }

        handle_tsk_return_value!(rv, tree)
    }

    fn advance_details(&mut self) {
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

    pub fn parent_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.parent, self.inner.num_nodes)
    }

    pub fn samples_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.samples, num_samples)
        )
    }

    pub fn next_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.next_sample, self.inner.num_nodes)
        )
    }

    pub fn left_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.left_sample, self.inner.num_nodes)
        )
    }

    pub fn right_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.right_sample, self.inner.num_nodes)
        )
    }

    pub fn left_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_sib, self.inner.num_nodes)
    }

    pub fn right_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_sib, self.inner.num_nodes)
    }

    pub fn left_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_child, self.inner.num_nodes)
    }

    pub fn right_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_child, self.inner.num_nodes)
    }

    fn left_sample(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_sample).unwrap()
        )
    }

    fn right_sample(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_sample).unwrap()
        )
    }

    /// Return the `[left, right)` coordinates of the tree.
    pub fn interval(&self) -> (f64, f64) {
        unsafe { ((*self.as_ptr()).left, (*self.as_ptr()).right) }
    }

    /// Return the length of the genome for which this
    /// tree is the ancestry.
    pub fn span(&self) -> f64 {
        let i = self.interval();
        i.1 - i.0
    }

    /// Get the parent of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn parent(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.parent)
    }

    /// Get the left child of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn left_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_child)
    }

    /// Get the right child of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn right_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_child)
    }

    /// Get the left sib of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError`] if `u` is out of range.
    pub fn left_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_sib)
    }

    /// Get the right sib of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn right_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_sib)
    }

    /// Obtain the list of samples for the current tree/tree sequence
    /// as a vector.
    pub fn samples_to_vec(&self) -> Vec<tsk_id_t> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = unsafe { *(*(*self.as_ptr()).tree_sequence).samples.offset(i as isize) };
            rv.push(u);
        }
        rv
    }

    /// Return a [`NodeIterator`] from the node `u` to the root of the tree.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn path_to_root(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = PathToRootIterator::new(self, u)?;
        Ok(Box::new(iter))
    }

    /// Return a [`NodeIterator`] over the children of node `u`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `u` is out of range.
    pub fn children(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = ChildIterator::new(self, u)?;
        Ok(Box::new(iter))
    }
    /// Return a [`NodeIterator`] over the sample nodes descending from node `u`.
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
    pub fn samples(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = SamplesIterator::new(self, u)?;
        Ok(Box::new(iter))
    }

    /// Return a [`NodeIterator`] over the roots of the tree.
    ///
    /// # Note
    ///
    /// For a tree with multiple roots, the iteration starts
    /// at the left root.
    pub fn roots(&self) -> BoxedNodeIterator {
        Box::new(RootIterator::new(self))
    }

    /// Return all roots as a vector.
    pub fn roots_to_vec(&self) -> Vec<tsk_id_t> {
        let mut v = vec![];

        for r in self.roots() {
            v.push(r);
        }

        v
    }

    /// Return a [`NodeIterator`] over all nodes in the tree.
    ///
    /// # Parameters
    ///
    /// * `order`: A value from [`NodeTraversalOrder`] specifying the
    ///   iteration order.
    pub fn traverse_nodes(&self, order: NodeTraversalOrder) -> BoxedNodeIterator {
        match order {
            NodeTraversalOrder::Preorder => Box::new(PreorderNodeIterator::new(&self)),
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
    /// * `by_span`: if `true`, multiply the return value by [`Tree::span`].
    ///
    /// # Errors
    ///
    /// [`TskitError`] may be returned via [`Tree::nodes`].
    pub fn total_branch_length(&self, by_span: bool) -> Result<f64, TskitError> {
        let nt = self.node_table();
        let mut b = 0.;
        for n in self.traverse_nodes(NodeTraversalOrder::Preorder) {
            let p = self.parent(n)?;
            if p != TSK_NULL {
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
    pub fn num_tracked_samples(&self, u: tsk_id_t) -> Result<u64, TskitError> {
        let mut n = u64::MAX;
        let np: *mut u64 = &mut n;
        let code = unsafe { ll_bindings::tsk_tree_get_num_tracked_samples(self.as_ptr(), u, np) };
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
    pub fn kc_distance(&self, other: &Tree, lambda: f64) -> Result<f64, TskitError> {
        let mut kc = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        let code = unsafe {
            ll_bindings::tsk_tree_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        };
        handle_tsk_return_value!(code, kc)
    }
}

impl streaming_iterator::StreamingIterator for Tree {
    type Item = Tree;
    fn advance(&mut self) {
        self.advance_details();
    }

    fn get(&self) -> Option<&Tree> {
        match self.advanced {
            true => Some(&self),
            false => None,
        }
    }
}

/// Specify the traversal order used by
/// [`Tree::nodes`].
pub enum NodeTraversalOrder {
    ///Preorder traversal, starting at the root(s) of a [`Tree`].
    ///For trees with multiple roots, start at the left root,
    ///traverse to tips, proceeed to the next root, etc..
    Preorder,
}

struct PreorderNodeIterator {
    root_stack: Vec<i32>,
    node_stack: Vec<i32>,
    left_child: crate::ffi::TskIdArray,
    right_sib: crate::ffi::TskIdArray,
    current_node_: Option<tsk_id_t>,
}

impl PreorderNodeIterator {
    fn new(tree: &Tree) -> Self {
        let mut rv = PreorderNodeIterator {
            root_stack: tree.roots_to_vec(),
            node_stack: vec![],
            left_child: tree.left_child_array(),
            right_sib: tree.right_sib_array(),
            current_node_: None,
        };
        rv.root_stack.reverse();
        if let Some(root) = rv.root_stack.pop() {
            rv.node_stack.push(root);
        }
        rv
    }
}

impl NodeIterator for PreorderNodeIterator {
    fn next_node(&mut self) {
        self.current_node_ = self.node_stack.pop();
        match self.current_node_ {
            Some(u) => {
                let mut c = self.left_child[u];
                while c != TSK_NULL {
                    self.node_stack.push(c);
                    c = self.right_sib[c];
                }
            }
            None => {
                if let Some(r) = self.root_stack.pop() {
                    self.current_node_ = Some(r);
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node_
    }
}

struct RootIterator {
    current_root: Option<tsk_id_t>,
    next_root: tsk_id_t,
    right_sib: crate::ffi::TskIdArray,
}

impl RootIterator {
    fn new(tree: &Tree) -> Self {
        RootIterator {
            current_root: None,
            next_root: tree.inner.left_root,
            right_sib: tree.right_sib_array(),
        }
    }
}

impl NodeIterator for RootIterator {
    fn next_node(&mut self) {
        self.current_root = match self.next_root {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_root = self.right_sib[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_root
    }
}

struct ChildIterator {
    current_child: Option<tsk_id_t>,
    next_child: tsk_id_t,
    right_sib: crate::ffi::TskIdArray,
}

impl ChildIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        let c = tree.left_child(u)?;

        Ok(ChildIterator {
            current_child: None,
            next_child: c,
            right_sib: tree.right_sib_array(),
        })
    }
}

impl NodeIterator for ChildIterator {
    fn next_node(&mut self) {
        self.current_child = match self.next_child {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_child = self.right_sib[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_child
    }
}

struct PathToRootIterator {
    current_node: Option<tsk_id_t>,
    next_node: tsk_id_t,
    parent: crate::ffi::TskIdArray,
}

impl PathToRootIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        match u >= tree.num_nodes as tsk_id_t {
            true => Err(TskitError::IndexError),
            false => Ok(PathToRootIterator {
                current_node: None,
                next_node: u,
                parent: tree.parent_array(),
            }),
        }
    }
}

impl NodeIterator for PathToRootIterator {
    fn next_node(&mut self) {
        self.current_node = match self.next_node {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_node = self.parent[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node
    }
}

struct SamplesIterator {
    current_node: Option<tsk_id_t>,
    next_sample_index: tsk_id_t,
    last_sample_index: tsk_id_t,
    next_sample: crate::ffi::TskIdArray,
    samples: crate::ffi::TskIdArray,
}

impl SamplesIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        let rv = SamplesIterator {
            current_node: None,
            next_sample_index: tree.left_sample(u)?,
            last_sample_index: tree.right_sample(u)?,
            next_sample: tree.next_sample_array()?,
            samples: tree.samples_array()?,
        };

        Ok(rv)
    }
}

impl NodeIterator for SamplesIterator {
    fn next_node(&mut self) {
        self.current_node = match self.next_sample_index {
            TSK_NULL => None,
            r => {
                if r == self.last_sample_index {
                    let cr = Some(self.samples[r]);
                    self.next_sample_index = TSK_NULL;
                    cr
                } else {
                    assert!(r >= 0);
                    let cr = Some(self.samples[r]);
                    self.next_sample_index = self.next_sample[r];
                    cr
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node
    }
}

/// A tree sequence.
///
/// This is a thin wrapper around the C type `tsk_treeseq_t`.
///
/// When created from a [`TableCollection`], the input tables are
/// moved into the `TreeSequence` object.
///
/// # Examples
///
/// ```
/// let mut tables = tskit::TableCollection::new(1000.).unwrap();
/// tables.add_node(0, 1.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_edge(0., 1000., 0, 1).unwrap();
/// tables.add_edge(0., 1000., 0, 2).unwrap();
///
/// // index
/// tables.build_index();
///
/// // tables gets moved into our treeseq variable:
/// let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
/// ```
pub struct TreeSequence {
    inner: Box<ll_bindings::tsk_treeseq_t>,
}

build_tskit_type!(TreeSequence, ll_bindings::tsk_treeseq_t, tsk_treeseq_free);

impl TreeSequence {
    /// Create a tree sequence from a [`TableCollection`].
    /// In general, [`TableCollection::tree_sequence`] may be preferred.
    /// The table collection is moved/consumed.
    ///
    /// # Parameters
    ///
    /// * `tables`, a [`TableCollection`]
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if the tables are not indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tskit::TreeSequence::new(tables, tskit::TreeSequenceFlags::default()).unwrap();
    /// ```
    ///
    /// The following may be preferred to the previous example, and more closely
    /// mimics the Python `tskit` interface:
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// ```
    ///
    /// The following raises an error because the tables are not indexed:
    ///
    /// ```should_panic
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// let tree_sequence = tskit::TreeSequence::new(tables,
    /// tskit::TreeSequenceFlags::default()).unwrap();
    /// ```
    pub fn new(tables: TableCollection, flags: TreeSequenceFlags) -> Result<Self, TskitError> {
        let mut treeseq = Self::wrap();
        let rv = unsafe {
            ll_bindings::tsk_treeseq_init(treeseq.as_mut_ptr(), tables.as_ptr(), flags.bits())
        };
        handle_tsk_return_value!(rv, treeseq)
    }

    /// Dump the tree sequence to file.
    ///
    /// # Note
    ///
    /// * `options` is currently not used.  Set to default value.
    ///   This behavior may change in a future release, which could
    ///   break `API`.
    ///
    /// # Note
    ///
    /// This function always sets the flag TSK_NO_BUILD_INDEXES.
    pub fn dump(&self, filename: &str, options: TableOutputOptions) -> TskReturnValue {
        let c_str = std::ffi::CString::new(filename).unwrap();
        let rv = unsafe {
            ll_bindings::tsk_treeseq_dump(
                self.as_ptr() as *mut ll_bindings::tsk_treeseq_t,
                c_str.as_ptr(),
                options.bits() | ll_bindings::TSK_NO_BUILD_INDEXES,
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Load from a file.
    pub fn load(filename: &str) -> Result<Self, TskitError> {
        let tables = TableCollection::new_from_file(filename)?;

        Self::new(tables, TreeSequenceFlags::default())
    }

    /// Obtain a copy of the [`TableCollection`].
    /// The result is a "deep" copy of the tables.
    ///
    /// # Errors
    ///
    /// [`TskitError`] will be raised if the underlying C library returns an error code.
    pub fn dump_tables(&self) -> Result<TableCollection, TskitError> {
        let mut copy = TableCollection::new(1.)?;

        let rv = unsafe {
            ll_bindings::tsk_table_collection_copy(self.inner.tables, copy.as_mut_ptr(), 0)
        };

        handle_tsk_return_value!(rv, copy)
    }

    /// Create an iterator over trees.
    ///
    /// # Parameters
    ///
    /// * `flags` A [`TreeFlags`] bit field.
    ///
    /// # Errors
    ///
    /// # Examples
    ///
    /// ```
    /// // You must include streaming_iterator as a dependency
    /// // and import this type.
    /// use streaming_iterator::StreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// let mut tree_iterator = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iterator.next() {
    /// }
    /// ```
    ///
    /// # Warning
    ///
    /// The following code results in an infinite loop.
    /// Be sure to note the difference from the previous example.
    ///
    /// ```no_run
    /// use streaming_iterator::StreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// while let Some(tree) = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap().next() {
    /// }
    /// ```
    pub fn tree_iterator(&self, flags: TreeFlags) -> Result<Tree, TskitError> {
        let tree = Tree::new(self, flags)?;

        Ok(tree)
    }

    /// Get the list of samples as a vector.
    pub fn samples_to_vec(&self) -> Vec<tsk_id_t> {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = unsafe { *(*self.as_ptr()).samples.offset(i as isize) };
            rv.push(u);
        }
        rv
    }

    /// Get the number of trees.
    pub fn num_trees(&self) -> tsk_size_t {
        unsafe { ll_bindings::tsk_treeseq_get_num_trees(self.as_ptr()) }
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
    ///    See [`Tree::kc_distance`] for more details.
    pub fn kc_distance(&self, other: &TreeSequence, lambda: f64) -> Result<f64, TskitError> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        let code = unsafe {
            ll_bindings::tsk_treeseq_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        };
        handle_tsk_return_value!(code, kc)
    }

    // FIXME: document
    pub fn num_samples(&self) -> tsk_size_t {
        unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) }
    }

    /// Simplify tables and return a new tree sequence.
    ///
    /// # Parameters
    ///
    /// * `samples`: a slice containing non-null node ids.
    ///   The tables are simplified with respect to the ancestry
    ///   of these nodes.
    /// * `options`: A [`SimplificationOptions`] bit field controlling
    ///   the behavior of simplification.
    /// * `idmap`: if `true`, the return value contains a vector equal
    ///   in length to the input node table.  For each input node,
    ///   this vector either contains the node's new index or [`TSK_NULL`]
    ///   if the input node is not part of the simplified history.
    pub fn simplify(
        &self,
        samples: &[tsk_id_t],
        options: SimplificationOptions,
        idmap: bool,
    ) -> Result<(Self, Option<Vec<tsk_id_t>>), TskitError> {
        let mut tables = TableCollection::new(unsafe { (*self.inner.tables).sequence_length })?;
        tables.build_index().unwrap();
        let mut ts = tables.tree_sequence(TreeSequenceFlags::default())?;
        let mut output_node_map: Vec<tsk_id_t> = vec![];
        if idmap {
            output_node_map.resize(self.nodes().num_rows() as usize, TSK_NULL);
        }
        let rv = unsafe {
            ll_bindings::tsk_treeseq_simplify(
                self.as_ptr(),
                samples.as_ptr(),
                samples.len() as tsk_size_t,
                options.bits(),
                ts.as_mut_ptr(),
                match idmap {
                    true => output_node_map.as_mut_ptr(),
                    false => std::ptr::null_mut(),
                },
            )
        };
        handle_tsk_return_value!(
            rv,
            (
                ts,
                match idmap {
                    true => Some(output_node_map),
                    false => None,
                }
            )
        )
    }
}

impl TableAccess for TreeSequence {
    fn edges(&self) -> EdgeTable {
        EdgeTable::new_from_table(unsafe { &(*self.inner.tables).edges })
    }

    fn individuals(&self) -> IndividualTable {
        IndividualTable::new_from_table(unsafe { &(*self.inner.tables).individuals })
    }

    fn migrations(&self) -> MigrationTable {
        MigrationTable::new_from_table(unsafe { &(*self.inner.tables).migrations })
    }

    fn nodes(&self) -> NodeTable {
        NodeTable::new_from_table(unsafe { &(*self.inner.tables).nodes })
    }

    fn sites(&self) -> SiteTable {
        SiteTable::new_from_table(unsafe { &(*self.inner.tables).sites })
    }

    fn mutations(&self) -> MutationTable {
        MutationTable::new_from_table(unsafe { &(*self.inner.tables).mutations })
    }

    fn populations(&self) -> PopulationTable {
        PopulationTable::new_from_table(unsafe { &(*self.inner.tables).populations })
    }
}

impl crate::traits::NodeListGenerator for TreeSequence {}

#[cfg(test)]
pub(crate) mod test_trees {
    use super::*;
    use crate::test_fixtures::{
        make_small_table_collection, make_small_table_collection_two_trees,
        treeseq_from_small_table_collection, treeseq_from_small_table_collection_two_trees,
    };
    use streaming_iterator::StreamingIterator;

    #[test]
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let treeseq = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
        let samples = treeseq.samples_to_vec();
        assert_eq!(samples.len(), 2);
        for i in 1..3 {
            assert_eq!(samples[i - 1], i as tsk_id_t);
        }
    }

    #[test]
    fn test_create_treeseq_from_tables() {
        let tables = make_small_table_collection();
        let _treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    }

    #[test]
    fn test_iterate_tree_seq_with_one_tree() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let mut ntrees = 0;
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            ntrees += 1;
            assert_eq!(tree.current_tree, ntrees);
            let samples = tree.samples_to_vec();
            assert_eq!(samples.len(), 2);
            for i in 1..3 {
                assert_eq!(samples[i - 1], i as tsk_id_t);

                let mut nsteps = 0;
                for _ in tree.path_to_root(samples[i - 1]).unwrap() {
                    nsteps += 1;
                }
                assert_eq!(nsteps, 2);
            }
            let roots = tree.roots_to_vec();
            for r in roots.iter() {
                let mut num_children = 0;
                for _ in tree.children(*r).unwrap() {
                    num_children += 1;
                }
                assert_eq!(num_children, 2);
            }
        }
        assert_eq!(ntrees, 1);
    }

    #[test]
    fn test_iterate_no_roots() {
        let mut tables = TableCollection::new(100.).unwrap();
        tables.build_index().unwrap();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            let mut num_roots = 0;
            for _ in tree.roots() {
                num_roots += 1;
            }
            assert_eq!(num_roots, 0);
        }
    }

    #[should_panic]
    #[test]
    fn test_samples_iterator_error_when_not_tracking_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                for _ in tree.samples(n).unwrap() {}
            }
        }
    }

    #[test]
    fn test_num_tracked_samples() {
        let treeseq = treeseq_from_small_table_collection();
        assert_eq!(treeseq.inner.num_samples, 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(1).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(0).unwrap(), 2);
        }
    }

    #[should_panic]
    #[test]
    fn test_num_tracked_samples_not_tracking_samples() {
        let treeseq = treeseq_from_small_table_collection();
        assert_eq!(treeseq.inner.num_samples, 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::NO_SAMPLE_COUNTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(1).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(0).unwrap(), 0);
        }
    }

    #[test]
    fn test_iterate_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert!(!tree.flags.contains(TreeFlags::NO_SAMPLE_COUNTS));
            assert!(tree.flags.contains(TreeFlags::SAMPLE_LISTS));
            let mut s = vec![];
            for i in tree.samples(0).unwrap() {
                s.push(i);
            }
            assert_eq!(s.len(), 2);
            assert_eq!(s.len(), tree.num_tracked_samples(0).unwrap() as usize);
            assert_eq!(s[0], 1);
            assert_eq!(s[1], 2);

            for u in 1..3 {
                let mut s = vec![];
                for i in tree.samples(u).unwrap() {
                    s.push(i);
                }
                assert_eq!(s.len(), 1);
                assert_eq!(s[0], u);
                assert_eq!(s.len(), tree.num_tracked_samples(u).unwrap() as usize);
            }
        } else {
            panic!("Expected a tree");
        }
    }

    #[test]
    fn test_iterate_samples_two_trees() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        assert_eq!(treeseq.inner.num_trees, 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        while let Some(tree) = tree_iter.next() {
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                let mut nsamples = 0;
                for _ in tree.samples(n).unwrap() {
                    nsamples += 1;
                }
                assert!(nsamples > 0);
                assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
            }
        }
    }

    #[test]
    fn test_kc_distance_naive_test() {
        let ts1 = treeseq_from_small_table_collection();
        let ts2 = treeseq_from_small_table_collection();

        let kc = ts1.kc_distance(&ts2, 0.0).unwrap();
        assert!(kc.is_finite());
        assert!((kc - 0.).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dump_tables() {
        let tables = make_small_table_collection_two_trees();
        // Have to make b/c tables will no longer exist after making the treeseq
        let tables_copy = tables.deepcopy().unwrap();
        let ts = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let dumped = ts.dump_tables().unwrap();
        assert!(tables_copy.equals(&dumped, crate::TableEqualityOptions::default()));
    }
}
