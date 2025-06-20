use crate::error::TskitError;
use crate::sys;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeId;
use crate::NodeTable;
use crate::PopulationTable;
use crate::Position;
use crate::SimplificationOptions;
use crate::SiteTable;
use crate::SizeType;
use crate::TableCollection;
use crate::TableOutputOptions;
use crate::TreeFlags;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use sys::bindings as ll_bindings;

use super::Tree;

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
/// assert_eq!(treeseq.nodes().num_rows(), 3);
/// assert_eq!(treeseq.edges().num_rows(), 2);
/// ```
///
/// This type does not provide access to mutable tables.
///
/// ```compile_fail
/// # let mut tables = tskit::TableCollection::new(1000.).unwrap();
/// # tables.add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_edge(0., 1000., 0, 1).unwrap();
/// # tables.add_edge(0., 1000., 0, 2).unwrap();
///
/// # // index
/// # tables.build_index();
///
/// # // tables gets moved into our treeseq variable:
/// # let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
/// assert_eq!(treeseq.nodes_mut().num_rows(), 3);
/// ```
pub struct TreeSequence {
    pub(crate) inner: sys::TreeSequence,
    tables: crate::TableCollection,
}

unsafe impl Send for TreeSequence {}
unsafe impl Sync for TreeSequence {}

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
        let raw_tables_ptr = tables.into_inner();
        let mut inner = sys::TreeSequence::new(raw_tables_ptr, flags.into())?;
        let tables = unsafe {
            TableCollection::new_from_ll(sys::TableCollection::new_borrowed(
                std::ptr::NonNull::new(inner.as_mut().tables).unwrap(),
            ))
        }?;
        Ok(Self { inner, tables })
    }

    fn as_ref(&self) -> &ll_bindings::tsk_treeseq_t {
        self.inner.as_ref()
    }

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_treeseq_t {
        self.inner.as_ref()
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_treeseq_t {
        self.inner.as_mut()
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
        let c_str = std::ffi::CString::new(filename).map_err(|_| {
            TskitError::LibraryError("call to ffi::Cstring::new failed".to_string())
        })?;
        self.inner.dump(c_str, options.into().bits())
    }

    /// Load from a file.
    ///
    /// This function calls [`TableCollection::new_from_file`] with
    /// [`TreeSequenceFlags::default`].
    pub fn load(filename: impl AsRef<str>) -> Result<Self, TskitError> {
        let tables = TableCollection::new_from_file(filename.as_ref())?;

        Self::new(tables, TreeSequenceFlags::default())
    }

    /// Obtain the underlying [`TableCollection`].
    ///
    ///
    /// # Errors
    ///
    /// [`TskitError`] will be raised if the underlying C library returns an error code.
    pub fn dump_tables(self) -> Result<TableCollection, TskitError> {
        assert!(!self.as_ptr().is_null());
        let mut treeseq = self;
        // SAFETY: the above assert passed
        let tables = std::ptr::NonNull::new(unsafe { (*treeseq.as_ptr()).tables }).unwrap();
        // SAFETY: the above assert passed
        unsafe { (*treeseq.as_mut_ptr()).tables = std::ptr::null_mut() };
        // SAFETY: the table collection points to data that has passed
        // tsk_table_collection_check_integrity, meaning that it must be initialized.
        let tables = unsafe { crate::sys::TableCollection::new_owning_from_nonnull(tables) };
        crate::TableCollection::new_from_ll(tables)
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
    /// use tskit::StreamingIterator;
    /// // Import this to allow .next_back() for reverse
    /// // iteration over trees.
    /// use tskit::DoubleEndedStreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// let mut tree_iterator = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iterator.next() {
    /// }
    /// ```
    ///
    /// ## Coupled liftimes
    ///
    /// A `Tree`'s lifetime is tied to that of its tree sequence:
    ///
    /// ```compile_fail
    /// # use tskit::StreamingIterator;
    /// # use tskit::DoubleEndedStreamingIterator;
    /// # let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// # tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// let mut tree_iterator = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// drop(tree_sequence);
    /// while let Some(tree) = tree_iterator.next() { // compile fail.
    /// }
    /// ```
    /// # Warning
    ///
    /// The following code results in an infinite loop.
    /// Be sure to note the difference from the previous example.
    ///
    /// ```no_run
    /// use tskit::StreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// while let Some(tree) = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap().next() {
    /// }
    /// ```
    pub fn tree_iterator<F: Into<TreeFlags>>(&self, flags: F) -> Result<Tree, TskitError> {
        let tree = Tree::new(&self.inner, flags)?;

        Ok(tree)
    }

    /// Create an iterator over trees starting at a specific position.
    ///
    /// See [`TreeSequence::tree_iterator`] for details
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if `at` is not valid
    pub fn tree_iterator_at_position<F: Into<TreeFlags>, P: Into<Position>>(
        &self,
        flags: F,
        at: P,
    ) -> Result<Tree, TskitError> {
        Tree::new_at_position(&self.inner, flags, at)
    }

    /// Create an iterator over trees starting at a specific tree index.
    ///
    /// See [`TreeSequence::tree_iterator`] for details
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if `at` is not valid
    pub fn tree_iterator_at_index<F: Into<TreeFlags>>(
        &self,
        flags: F,
        at: i32,
    ) -> Result<Tree, TskitError> {
        Tree::new_at_index(&self.inner, flags, at)
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        unsafe {
            let num_samples = ll_bindings::tsk_treeseq_get_num_samples(self.as_ref());
            sys::generate_slice(self.as_ref().samples, num_samples)
        }
    }

    /// Get the number of trees.
    pub fn num_trees(&self) -> SizeType {
        self.inner.num_trees()
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
    pub fn kc_distance(&self, other: &TreeSequence, lambda: f64) -> Result<f64, TskitError> {
        self.inner.kc_distance(&other.inner, lambda)
    }

    // FIXME: document
    pub fn num_samples(&self) -> SizeType {
        self.inner.num_samples()
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
        let mut output_node_map: Vec<NodeId> = vec![];
        if idmap {
            output_node_map.resize(usize::try_from(self.nodes().num_rows())?, NodeId::NULL);
        }
        let mut inner = self.inner.simplify(
            samples,
            options.into(),
            match idmap {
                true => Some(&mut output_node_map),
                false => None,
            },
        )?;
        let tables = unsafe {
            TableCollection::new_from_ll(sys::TableCollection::new_borrowed(
                std::ptr::NonNull::new(inner.as_mut().tables).unwrap(),
            ))
        }?;
        Ok((
            Self { inner, tables },
            match idmap {
                true => Some(output_node_map),
                false => None,
            },
        ))
    }

    /// Truncate the [TreeSequence] to specified genome intervals.
    ///
    /// # Return value
    /// - `Ok(None)`: when truncation leads to empty edge table.
    /// - `Ok(Some(TableCollection))`: when trunction is successfully performed
    ///   and results in non-empty edge table. The tables are sorted.
    /// - `Error(TskitError)`: Any errors from the C API propagate. An
    ///   [TskitError::RangeError] will occur when `intervals` are not
    ///   sorted.
    ///
    /// # Notes
    ///
    /// - There is no option to simplify the output value.
    ///   Do so manually if desired.
    ///   Encapsulate the procedure if necessary.
    ///
    /// # Example
    /// ```rust
    ///  # use tskit::*;
    ///  # let snode = NodeFlags::new_sample();
    ///  # let anode = NodeFlags::default();
    ///  # let pop = PopulationId::NULL;
    ///  # let ind = IndividualId::NULL;
    ///  # let seqlen = 100.0;
    ///  # let (t0, t10) = (0.0, 10.0);
    ///  # let (left, right) = (0.0, 100.0);
    ///  # let sim_opts = SimplificationOptions::default();
    ///  #
    ///  # let mut tables = TableCollection::new(seqlen).unwrap();
    ///  # let child1 = tables.add_node(snode, t0, pop, ind).unwrap();
    ///  # let child2 = tables.add_node(snode, t0, pop, ind).unwrap();
    ///  # let parent = tables.add_node(anode, t10, pop, ind).unwrap();
    ///  #
    ///  # tables.add_edge(left, right, parent, child1).unwrap();
    ///  # tables.add_edge(left, right, parent, child2).unwrap();
    ///  # tables.full_sort(TableSortOptions::all()).unwrap();
    ///  # tables.simplify(&[child1, child2], sim_opts, false).unwrap();
    ///  # tables.build_index().unwrap();
    ///  #
    ///  # let trees = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
    ///  #
    ///  let intervals = [(0.0, 10.0), (90.0, 100.0)].into_iter();
    ///  let mut tables = trees.keep_intervals(intervals).unwrap().unwrap();
    ///  // Conversion back to tree sequence requires the usual steps
    ///  tables.simplify(&tables.samples_as_vector(), tskit::SimplificationOptions::default(), false).unwrap();
    ///  tables.build_index().unwrap();
    ///  let trees = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// ```
    ///
    /// Note that no new provenance will be appended.
    pub fn keep_intervals<P>(
        self,
        intervals: impl Iterator<Item = (P, P)>,
    ) -> Result<Option<TableCollection>, TskitError>
    where
        P: Into<Position>,
    {
        self.dump_tables()?.keep_intervals(intervals)
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Add provenance record with a time stamp.
    ///
    /// All implementation of this trait provided by `tskit` use
    /// an `ISO 8601` format time stamp
    /// written using the [RFC 3339](https://tools.ietf.org/html/rfc3339)
    /// specification.
    /// This formatting approach has been the most straightforward method
    /// for supporting round trips to/from a [`crate::provenance::ProvenanceTable`].
    /// The implementations used here use the [`chrono`](https://docs.rs/chrono/latest/chrono/) crate.
    ///
    /// # Parameters
    ///
    /// * `record`: the provenance record
    ///
    /// # Examples
    ///
    /// ```
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
    /// let dt_utc = chrono::DateTime::<chrono::Utc>::from_str(&timestamp).unwrap();
    /// println!("utc = {}", dt_utc);
    /// # }
    /// ```
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        if record.is_empty() {
            return Err(TskitError::ValueError {
                got: "empty string".to_string(),
                expected: "provenance record".to_string(),
            });
        }
        let timestamp = chrono::Utc::now().to_string();
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_add_row(
                &mut (*self.inner.as_ref().tables).provenances,
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as ll_bindings::tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as ll_bindings::tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, crate::ProvenanceId::from(rv))
    }

    /// Build a lending iterator over edge differences.
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if the `C` back end is unable to allocate
    ///   needed memory
    pub fn edge_differences_iter(
        &self,
    ) -> Result<crate::edge_differences::EdgeDifferencesIterator, TskitError> {
        crate::edge_differences::EdgeDifferencesIterator::new_from_treeseq(self, 0)
    }

    /// Reference to the underlying table collection.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.add_node(tskit::NodeFlags::default(),0.0, -1, -1).unwrap();
    /// tables.build_index();
    /// let tcopy = tables.deepcopy().unwrap();
    /// let tree_sequence = tskit::TreeSequence::try_from(tcopy).unwrap();
    /// assert_eq!(tables.equals(tree_sequence.tables(), 0), true);
    /// ```
    pub fn tables(&self) -> &TableCollection {
        &self.tables
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    pub fn edges(&self) -> &EdgeTable {
        self.tables.edges()
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    pub fn nodes(&self) -> &NodeTable {
        self.tables.nodes()
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    pub fn sites(&self) -> &SiteTable {
        self.tables.sites()
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    pub fn migrations(&self) -> &MigrationTable {
        self.tables.migrations()
    }

    pub fn mutations(&self) -> &MutationTable {
        self.tables.mutations()
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    pub fn individuals(&self) -> &IndividualTable {
        self.tables.individuals()
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    pub fn populations(&self) -> &PopulationTable {
        self.tables.populations()
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances(&self) -> &crate::provenance::ProvenanceTable {
        self.tables.provenances()
    }

    /// Return an iterator over the individuals.
    pub fn individuals_iter(&self) -> impl Iterator<Item = crate::IndividualTableRow> + '_ {
        self.individuals().iter()
    }

    /// Return an iterator over the nodes.
    pub fn nodes_iter(&self) -> impl Iterator<Item = crate::NodeTableRow> + '_ {
        self.nodes().iter()
    }
    /// Return an iterator over the edges.
    pub fn edges_iter(&self) -> impl Iterator<Item = crate::EdgeTableRow> + '_ {
        self.edges().iter()
    }
    /// Return an iterator over the migrations.
    pub fn migrations_iter(&self) -> impl Iterator<Item = crate::MigrationTableRow> + '_ {
        self.migrations().iter()
    }
    /// Return an iterator over the mutations.
    pub fn mutations_iter(&self) -> impl Iterator<Item = crate::MutationTableRow> + '_ {
        self.mutations().iter()
    }
    /// Return an iterator over the populations.
    pub fn populations_iter(&self) -> impl Iterator<Item = crate::PopulationTableRow> + '_ {
        self.populations().iter()
    }
    /// Return an iterator over the sites.
    pub fn sites_iter(&self) -> impl Iterator<Item = crate::SiteTableRow> + '_ {
        self.sites().iter()
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Return an iterator over provenances
    pub fn provenances_iter(
        &self,
    ) -> impl Iterator<Item = crate::provenance::ProvenanceTableRow> + '_ {
        self.provenances().iter()
    }

    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::NodeFlags::is_sample`]
    /// is `true`.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::samples_as_vector`].
    pub fn samples_as_vector(&self) -> Vec<crate::NodeId> {
        self.tables.samples_as_vector()
    }

    /// Obtain a vector containing the indexes ("ids") of all nodes
    /// satisfying a certain criterion.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::create_node_id_vector`].
    ///
    /// # Parameters
    ///
    /// * `f`: a function.  The function is passed the current table
    ///   collection and each [`crate::node_table::NodeTableRow`].
    ///   If `f` returns `true`, the index of that row is included
    ///   in the return value.
    ///
    /// # Examples
    ///
    /// Get all nodes with time > 0.0:
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// tables
    ///     .add_node(tskit::NodeFlags::new_sample(), 0.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
    ///     .unwrap();
    /// tables
    ///     .add_node(tskit::NodeFlags::new_sample(), 1.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
    ///     .unwrap();
    /// let samples = tables.create_node_id_vector(
    ///     |row: &tskit::NodeTableRow| row.time > 0.,
    /// );
    /// assert_eq!(samples[0], 1);
    /// ```
    pub fn create_node_id_vector(
        &self,
        f: impl FnMut(&crate::NodeTableRow) -> bool,
    ) -> Vec<crate::NodeId> {
        self.tables.create_node_id_vector(f)
    }
}

impl TryFrom<TableCollection> for TreeSequence {
    type Error = TskitError;

    fn try_from(value: TableCollection) -> Result<Self, Self::Error> {
        Self::new(value, TreeSequenceFlags::default())
    }
}
