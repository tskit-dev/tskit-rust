use crate::error::{TskitError, TskitErrorEnum};
use crate::sys;
use crate::NodeId;
use crate::SimplificationOptions;
use crate::SizeType;
use crate::TableCollection;
use crate::TableOutputOptions;
use crate::TreeFlags;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use ll_bindings::tsk_id_t;
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
    pub(crate) inner: sys::LLTreeSeq,
    views: crate::table_views::TableViews,
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
        let raw_tables_ptr = tables.into_raw()?;
        let mut inner = sys::LLTreeSeq::new(raw_tables_ptr, flags.into())?;
        let views = crate::table_views::TableViews::new_from_tree_sequence(inner.as_mut_ptr())?;
        Ok(Self { inner, views })
    }

    fn as_ref(&self) -> &ll_bindings::tsk_treeseq_t {
        self.inner.as_ref()
    }

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_treeseq_t {
        self.inner.as_ptr()
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_treeseq_t {
        self.inner.as_mut_ptr()
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
            TskitError::from(TskitErrorEnum::LibraryError(
                "call to ffi::Cstring::new failed".to_string(),
            ))
        })?;
        self.inner
            .dump(c_str, options.into().bits())
            .map_err(|e| e.into())
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
        handle_tsk_return_value!(rv, unsafe { TableCollection::new_from_mbox(inner)? })
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
    /// ## Coupled liftimes
    ///
    /// A `Tree`'s lifetime is tied to that of its tree sequence:
    ///
    /// ```{compile_fail}
    /// # use streaming_iterator::StreamingIterator;
    /// # use streaming_iterator::DoubleEndedStreamingIterator;
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
    /// use streaming_iterator::StreamingIterator;
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
        sys::generate_slice(self.as_ref().samples, num_samples)
    }

    /// Get the number of trees.
    pub fn num_trees(&self) -> SizeType {
        self.inner.num_trees().into()
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
        self.inner
            .kc_distance(&other.inner, lambda)
            .map_err(|e| e.into())
    }

    // FIXME: document
    pub fn num_samples(&self) -> SizeType {
        self.inner.num_samples().into()
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
        let llsamples = unsafe {
            std::slice::from_raw_parts(samples.as_ptr().cast::<tsk_id_t>(), samples.len())
        };
        let mut inner = self.inner.simplify(
            llsamples,
            options.into(),
            match idmap {
                true => output_node_map.as_mut_ptr().cast::<tsk_id_t>(),
                false => std::ptr::null_mut(),
            },
        )?;
        let views = crate::table_views::TableViews::new_from_tree_sequence(inner.as_mut_ptr())?;
        Ok((
            Self { inner, views },
            match idmap {
                true => Some(output_node_map),
                false => None,
            },
        ))
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
    /// The implementations used here use the [`humantime`](https://docs.rs/humantime/latest/humantime/) crate.
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
    /// let dt_utc = humantime::Timestamp::from_str(&timestamp).unwrap();
    /// println!("utc = {}", dt_utc);
    /// # }
    /// ```
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        if record.is_empty() {
            return Err(TskitError::from(TskitErrorEnum::ValueError {
                got: "empty string".to_string(),
                expected: "provenance record".to_string(),
            }));
        }
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
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

    delegate_table_view_api!();

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
}

impl TryFrom<TableCollection> for TreeSequence {
    type Error = TskitError;

    fn try_from(value: TableCollection) -> Result<Self, Self::Error> {
        Self::new(value, TreeSequenceFlags::default())
    }
}
