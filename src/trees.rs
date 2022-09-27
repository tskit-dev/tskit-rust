use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeId;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SimplificationOptions;
use crate::SiteTable;
use crate::SizeType;
use crate::TableAccess;
use crate::TableOutputOptions;
use crate::TreeFlags;
use crate::TreeInterface;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::TskitTypeAccess;
use crate::{tsk_id_t, tsk_size_t, TableCollection};
use std::ptr::NonNull;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree {
    pub(crate) inner: ll_bindings::tsk_tree_t,
    api: TreeInterface,
    current_tree: i32,
    advanced: bool,
}

pub struct TreeIterator {
    tree: ll_bindings::tsk_tree_t,
    current_tree: i32,
    advanced: bool,
    num_nodes: tsk_size_t,
    array_len: tsk_size_t,
    flags: TreeFlags,
}

impl TreeIterator {
    // FIXME: init if fallible!
    fn new(treeseq: &TreeSequence) -> Self {
        let mut tree = MaybeUninit::<ll_bindings::tsk_tree_t>::uninit();
        let _rv = unsafe {
            ll_bindings::tsk_tree_init(tree.as_mut_ptr(), treeseq.as_ptr(), 0);
        };
        let tree = unsafe { tree.assume_init() };

        Self {
            tree,
            current_tree: -1,
            advanced: false,
            num_nodes: 0,
            array_len: 0,
            flags: 0.into(),
        }
    }
    fn item(&mut self) -> NonOwningTree {
        NonOwningTree::new(
            NonNull::from(&mut self.tree),
            self.current_tree,
            self.advanced,
            self.num_nodes,
            self.array_len,
            self.flags,
        )
    }
}

#[derive(Debug)]
pub struct NonOwningTree {
    tree: NonNull<ll_bindings::tsk_tree_t>,
    api: TreeInterface,
    current_tree: i32,
    advanced: bool,
    num_nodes: tsk_size_t,
    array_len: tsk_size_t,
    flags: TreeFlags,
}

impl Deref for NonOwningTree {
    type Target = TreeInterface;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl NonOwningTree {
    fn new(
        tree: NonNull<ll_bindings::tsk_tree_t>,
        current_tree: i32,
        advanced: bool,
        num_nodes: tsk_size_t,
        array_len: tsk_size_t,
        flags: TreeFlags,
    ) -> Self {
        let api = TreeInterface::new(tree, num_nodes, array_len, flags);
        Self {
            tree,
            api,
            current_tree,
            advanced,
            num_nodes,
            array_len,
            flags,
        }
    }

    fn as_owned(&self) -> &Self {
        self
    }
}

impl Iterator for TreeIterator {
    type Item = NonOwningTree;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_first(&mut self.tree) }
        } else {
            unsafe { ll_bindings::tsk_tree_next(&mut self.tree) }
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
        if self.advanced {
            Some(self.item())
        } else {
            None
        }
    }
}

impl Drop for TreeIterator {
    fn drop(&mut self) {
        unsafe { ll_bindings::tsk_tree_free(&mut self.tree) };
    }
}

// Trait defining iteration over nodes.
trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<NodeId>;
}

impl Deref for Tree {
    type Target = TreeInterface;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}

impl Tree {
    fn new<F: Into<TreeFlags>>(ts: &TreeSequence, flags: F) -> Result<Self, TskitError> {
        let flags = flags.into();
        let mut tree = MaybeUninit::<ll_bindings::tsk_tree_t>::uninit();
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
                    ts.num_samples().into(),
                    (*tree.as_ptr()).samples,
                )
            };
        }

        let mut tree = unsafe { tree.assume_init() };
        let ptr = &mut tree as *mut ll_bindings::tsk_tree_t;

        let num_nodes = unsafe { (*(*ts.as_ptr()).tables).nodes.num_rows };
        let non_owned_pointer = unsafe { NonNull::new_unchecked(ptr) };
        let api = TreeInterface::new(non_owned_pointer, num_nodes, num_nodes + 1, flags);
        handle_tsk_return_value!(
            rv,
            Tree {
                inner: tree,
                current_tree: 0,
                advanced: false,
                api
            }
        )
    }
}

impl streaming_iterator::StreamingIterator for Tree {
    type Item = Tree;
    fn advance(&mut self) {
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

    fn get(&self) -> Option<&Tree> {
        match self.advanced {
            true => Some(self),
            false => None,
        }
    }
}

impl streaming_iterator::DoubleEndedStreamingIterator for Tree {
    fn advance_back(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_last(self.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_prev(self.as_mut_ptr()) }
        };
        if rv == 0 {
            self.advanced = false;
            self.current_tree -= 1;
        } else if rv == 1 {
            self.advanced = true;
            self.current_tree -= 1;
        } else if rv < 0 {
            panic_on_tskit_error!(rv);
        }
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
/// tables.add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
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
    pub(crate) inner: ll_bindings::tsk_treeseq_t,
}

unsafe impl Send for TreeSequence {}
unsafe impl Sync for TreeSequence {}

impl TskitTypeAccess<ll_bindings::tsk_treeseq_t> for TreeSequence {
    fn as_ptr(&self) -> *const ll_bindings::tsk_treeseq_t {
        &self.inner
    }

    fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_treeseq_t {
        &mut self.inner
    }
}

impl Drop for TreeSequence {
    fn drop(&mut self) {
        let rv = unsafe { ll_bindings::tsk_treeseq_free(&mut self.inner) };
        assert_eq!(rv, 0);
    }
}

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
    /// * [`TskitError`] if the tables are not properly sorted.
    ///   See [`TableCollection::full_sort`](crate::TableCollection::full_sort).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tskit::TreeSequence::try_from(tables).unwrap();
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
    /// let tree_sequence = tskit::TreeSequence::try_from(tables).unwrap();
    /// ```
    ///
    /// ## Note
    ///
    /// This function makes *no extra copies* of the tables.
    /// There is, however, a temporary allocation of an empty table collection
    /// in order to convince rust that we are safely handling all memory.
    pub fn new<F: Into<TreeSequenceFlags>>(
        tables: TableCollection,
        flags: F,
    ) -> Result<Self, TskitError> {
        let mut inner = std::mem::MaybeUninit::<ll_bindings::tsk_treeseq_t>::uninit();
        let mut flags: u32 = flags.into().bits();
        flags |= ll_bindings::TSK_TAKE_OWNERSHIP;
        let raw_tables_ptr = tables.into_raw()?;
        let rv =
            unsafe { ll_bindings::tsk_treeseq_init(inner.as_mut_ptr(), raw_tables_ptr, flags) };
        handle_tsk_return_value!(rv, {
            let inner = unsafe { inner.assume_init() };
            Self { inner }
        })
    }

    /// Dump the tree sequence to file.
    ///
    /// # Note
    ///
    /// * `options` is currently not used.  Set to default value.
    ///   This behavior may change in a future release, which could
    ///   break `API`.
    ///
    /// # Panics
    ///
    /// This function allocates a `CString` to pass the file name to the C API.
    /// A panic will occur if the system runs out of memory.
    pub fn dump<O: Into<TableOutputOptions>>(&self, filename: &str, options: O) -> TskReturnValue {
        let c_str = std::ffi::CString::new(filename).unwrap();
        let rv = unsafe {
            ll_bindings::tsk_treeseq_dump(self.as_ptr(), c_str.as_ptr(), options.into().bits())
        };

        handle_tsk_return_value!(rv)
    }

    /// Load from a file.
    ///
    /// This function calls [`TableCollection::new_from_file`] with
    /// [`TreeSequenceFlags::default`].
    pub fn load(filename: impl AsRef<str>) -> Result<Self, TskitError> {
        let tables = TableCollection::new_from_file(filename.as_ref())?;

        Self::new(tables, TreeSequenceFlags::default())
    }

    /// Obtain a copy of the [`TableCollection`].
    /// The result is a "deep" copy of the tables.
    ///
    /// # Errors
    ///
    /// [`TskitError`] will be raised if the underlying C library returns an error code.
    pub fn dump_tables(&self) -> Result<TableCollection, TskitError> {
        let mut inner = crate::table_collection::uninit_table_collection();

        let rv = unsafe {
            ll_bindings::tsk_table_collection_copy((*self.as_ptr()).tables, &mut *inner, 0)
        };

        // SAFETY: we just initialized it.
        // The C API doesn't free NULL pointers.
        handle_tsk_return_value!(rv, unsafe { TableCollection::new_from_mbox(inner) })
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
    /// // Import this to allow .next_back() for reverse
    /// // iteration over trees.
    /// use streaming_iterator::DoubleEndedStreamingIterator;
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
    pub fn tree_iterator<F: Into<TreeFlags>>(&self, flags: F) -> Result<Tree, TskitError> {
        let tree = Tree::new(self, flags)?;

        Ok(tree)
    }

    /// Return an iterator over the trees.
    pub fn trees(&self) -> impl Iterator<Item = impl Deref<Target = TreeInterface>> + '_ {
        TreeIterator::new(self)
    }

    /// Get the list of samples as a vector.
    /// # Panics
    ///
    /// Will panic if the number of samples is too large to cast to a valid id.
    #[deprecated(
        since = "0.2.3",
        note = "Please use TreeSequence::sample_nodes instead"
    )]
    pub fn samples_to_vec(&self) -> Vec<NodeId> {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = match isize::try_from(i) {
                Ok(o) => NodeId::from(unsafe { *(*self.as_ptr()).samples.offset(o) }),
                Err(e) => panic!("{}", e),
            };
            rv.push(u);
        }
        rv
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        tree_array_slice!(self, samples, num_samples)
    }

    /// Get the number of trees.
    pub fn num_trees(&self) -> SizeType {
        unsafe { ll_bindings::tsk_treeseq_get_num_trees(self.as_ptr()) }.into()
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
    ///    See [`TreeInterface::kc_distance`] for more details.
    pub fn kc_distance(&self, other: &TreeSequence, lambda: f64) -> Result<f64, TskitError> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        let code = unsafe {
            ll_bindings::tsk_treeseq_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        };
        handle_tsk_return_value!(code, kc)
    }

    // FIXME: document
    pub fn num_samples(&self) -> SizeType {
        unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) }.into()
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
    ///   this vector either contains the node's new index or [`NodeId::NULL`]
    ///   if the input node is not part of the simplified history.
    pub fn simplify<O: Into<SimplificationOptions>>(
        &self,
        samples: &[NodeId],
        options: O,
        idmap: bool,
    ) -> Result<(Self, Option<Vec<NodeId>>), TskitError> {
        // The output is an UNINITIALIZED treeseq,
        // else we leak memory.
        let mut ts = MaybeUninit::<ll_bindings::tsk_treeseq_t>::uninit();
        let mut output_node_map: Vec<NodeId> = vec![];
        if idmap {
            output_node_map.resize(usize::try_from(self.nodes().num_rows())?, NodeId::NULL);
        }
        let rv = unsafe {
            ll_bindings::tsk_treeseq_simplify(
                self.as_ptr(),
                // NOTE: casting away const-ness:
                samples.as_ptr().cast::<tsk_id_t>(),
                samples.len() as tsk_size_t,
                options.into().bits(),
                ts.as_mut_ptr(),
                match idmap {
                    true => output_node_map.as_mut_ptr().cast::<tsk_id_t>(),
                    false => std::ptr::null_mut(),
                },
            )
        };
        handle_tsk_return_value!(
            rv,
            (
                {
                    let inner = unsafe { ts.assume_init() };
                    Self { inner }
                },
                match idmap {
                    true => Some(output_node_map),
                    false => None,
                }
            )
        )
    }

    #[cfg(any(feature = "provenance", doc))]
    /// Add provenance record with a time stamp.
    ///
    /// All implementation of this trait provided by `tskit` use
    /// an `ISO 8601` format time stamp
    /// written using the [RFC 3339](https://tools.ietf.org/html/rfc3339)
    /// specification.
    /// This formatting approach has been the most straightforward method
    /// for supporting round trips to/from a [`crate::provenance::ProvenanceTable`].
    /// The implementations used here use the [`humantime`](https://docs.rs/humantime/latest/humantime/) crate.
    ///
    /// # Parameters
    ///
    /// * `record`: the provenance record
    /// ```
    /// use tskit::TableAccess;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// let mut treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// # #[cfg(feature = "provenance")] {
    /// treeseq.add_provenance(&String::from("All your provenance r belong 2 us.")).unwrap();
    ///
    /// let prov_ref = treeseq.provenances();
    /// let row_0 = prov_ref.row(0).unwrap();
    /// assert_eq!(row_0.record, "All your provenance r belong 2 us.");
    /// let record_0 = prov_ref.record(0).unwrap();
    /// assert_eq!(record_0, row_0.record);
    /// let timestamp = prov_ref.timestamp(0).unwrap();
    /// assert_eq!(timestamp, row_0.timestamp);
    /// use core::str::FromStr;
    /// let dt_utc = humantime::Timestamp::from_str(&timestamp).unwrap();
    /// println!("utc = {}", dt_utc);
    /// # }
    /// ```
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_add_row(
                &mut (*self.inner.tables).provenances,
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, crate::ProvenanceId::from(rv))
    }
}

impl TryFrom<TableCollection> for TreeSequence {
    type Error = TskitError;

    fn try_from(value: TableCollection) -> Result<Self, Self::Error> {
        Self::new(value, TreeSequenceFlags::default())
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

    #[cfg(any(feature = "provenance", doc))]
    fn provenances(&self) -> crate::provenance::ProvenanceTable {
        crate::provenance::ProvenanceTable::new_from_table(unsafe {
            &(*self.inner.tables).provenances
        })
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
    use crate::NodeTraversalOrder;
    use streaming_iterator::DoubleEndedStreamingIterator;
    use streaming_iterator::StreamingIterator;

    #[test]
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let treeseq = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
        let samples = treeseq.sample_nodes();
        assert_eq!(samples.len(), 2);
        for i in 1..3 {
            assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));
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
            let samples = tree.sample_nodes();
            assert_eq!(samples.len(), 2);
            for i in 1..3 {
                assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));

                let mut nsteps = 0;
                for _ in tree.parents(samples[i - 1]).unwrap() {
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
        assert_eq!(treeseq.num_samples(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2.into()).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(1.into()).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(0.into()).unwrap(), 2);
        }
    }

    #[test]
    fn test_new_trees_iterator() {
        let treeseq = treeseq_from_small_table_collection();
        for tree in treeseq.trees() {
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                for p in tree.parents(n).unwrap() {
                    println!("{:?}", p);
                }
            }
        }

        // This is a safety sticking point:
        // we cannot collect the iterable itself b/c
        // the underlying tree memory is re-used.
        // let i = treeseq.trees();
        // let v = Vec::<Tree>::from_iter(i);
        // assert_eq!(v.len(), 2);
    }

    #[should_panic]
    #[test]
    fn test_num_tracked_samples_not_tracking_samples() {
        let treeseq = treeseq_from_small_table_collection();
        assert_eq!(treeseq.num_samples(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::NO_SAMPLE_COUNTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2.into()).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(1.into()).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(0.into()).unwrap(), 0);
        }
    }

    #[test]
    fn test_iterate_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert!(!tree.flags().contains(TreeFlags::NO_SAMPLE_COUNTS));
            assert!(tree.flags().contains(TreeFlags::SAMPLE_LISTS));
            let mut s = vec![];
            for i in tree.samples(0.into()).unwrap() {
                s.push(i);
            }
            assert_eq!(s.len(), 2);
            assert_eq!(
                s.len(),
                usize::try_from(tree.num_tracked_samples(0.into()).unwrap()).unwrap()
            );
            assert_eq!(s[0], 1);
            assert_eq!(s[1], 2);

            for u in 1..3 {
                let mut s = vec![];
                for i in tree.samples(u.into()).unwrap() {
                    s.push(i);
                }
                assert_eq!(s.len(), 1);
                assert_eq!(s[0], u);
                assert_eq!(
                    s.len(),
                    usize::try_from(tree.num_tracked_samples(u.into()).unwrap()).unwrap()
                );
            }
        } else {
            panic!("Expected a tree");
        }
    }

    #[test]
    fn test_iterate_samples_two_trees() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        assert_eq!(treeseq.num_trees(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        let expected_number_of_roots = vec![2, 1];
        let mut expected_root_ids = vec![
            vec![NodeId::from(0)],
            vec![NodeId::from(1), NodeId::from(0)],
        ];
        while let Some(tree) = tree_iter.next() {
            let mut num_roots = 0;
            let eroot_ids = expected_root_ids.pop().unwrap();
            for (i, r) in tree.roots().enumerate() {
                num_roots += 1;
                assert_eq!(r, eroot_ids[i]);
            }
            assert_eq!(
                expected_number_of_roots[(tree.current_tree - 1) as usize],
                num_roots
            );
            assert_eq!(tree.roots().count(), eroot_ids.len());
            let mut preoder_nodes = vec![];
            let mut postoder_nodes = vec![];
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                let mut nsamples = 0;
                preoder_nodes.push(n);
                for _ in tree.samples(n).unwrap() {
                    nsamples += 1;
                }
                assert!(nsamples > 0);
                assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
            }
            for n in tree.traverse_nodes(NodeTraversalOrder::Postorder) {
                let mut nsamples = 0;
                postoder_nodes.push(n);
                for _ in tree.samples(n).unwrap() {
                    nsamples += 1;
                }
                assert!(nsamples > 0);
                assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
            }
            assert_eq!(preoder_nodes.len(), postoder_nodes.len());

            // Test our preorder against the tskit functions in 0.99.15
            {
                let mut nodes: Vec<NodeId> = vec![
                    NodeId::NULL;
                    unsafe { ll_bindings::tsk_tree_get_size_bound(tree.as_ptr()) }
                        as usize
                ];
                let mut num_nodes: tsk_size_t = 0;
                let ptr = std::ptr::addr_of_mut!(num_nodes);
                unsafe {
                    ll_bindings::tsk_tree_preorder(
                        tree.as_ptr(),
                        nodes.as_mut_ptr() as *mut tsk_id_t,
                        ptr,
                    );
                }
                assert_eq!(num_nodes as usize, preoder_nodes.len());
                for i in 0..num_nodes as usize {
                    assert_eq!(preoder_nodes[i], nodes[i]);
                }
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

    #[test]
    fn test_reverse_tree_iteration() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        let mut starts_fwd = vec![];
        let mut stops_fwd = vec![];
        let mut starts_rev = vec![];
        let mut stops_rev = vec![];
        while let Some(tree) = tree_iter.next() {
            let interval = tree.interval();
            starts_fwd.push(interval.0);
            stops_fwd.push(interval.1);
        }
        assert_eq!(stops_fwd.len(), 2);
        assert_eq!(stops_fwd.len(), 2);

        // NOTE: we do NOT need to create a new iterator.
        while let Some(tree) = tree_iter.next_back() {
            let interval = tree.interval();
            starts_rev.push(interval.0);
            stops_rev.push(interval.1);
        }
        assert_eq!(starts_fwd.len(), starts_rev.len());
        assert_eq!(stops_fwd.len(), stops_rev.len());

        starts_rev.reverse();
        assert!(starts_fwd == starts_rev);
        stops_rev.reverse();
        assert!(stops_fwd == stops_rev);
    }

    // FIXME: remove later
    #[test]
    fn test_array_lifetime() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            let pa = tree.parent_array();
            let mut pc = vec![];
            for i in pa.iter() {
                pc.push(*i);
            }
            for (i, p) in pc.iter().enumerate() {
                assert_eq!(pa[i], *p);
            }
        } else {
            panic!("Expected a tree.");
        }
    }
}

#[cfg(test)]
mod test_treeeseq_send_sync {
    use crate::test_fixtures::treeseq_from_small_table_collection_two_trees;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn build_arc() {
        let t = treeseq_from_small_table_collection_two_trees();
        let a = Arc::new(t);
        let join_handle = thread::spawn(move || a.num_trees());
        let ntrees = join_handle.join().unwrap();
        assert_eq!(ntrees, 2);
    }
}
