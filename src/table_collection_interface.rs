use crate::bindings as ll_bindings;
use crate::tsk_id_t;
use crate::tsk_size_t;
use crate::types::Bookmark;
use crate::EdgeId;
use crate::IndividualTableSortOptions;
use crate::NodeId;
use crate::Position;
use crate::SimplificationOptions;
use crate::TableClearOptions;
use crate::TableCollection;
use crate::TableEqualityOptions;
use crate::TableIntegrityCheckFlags;
use crate::TableOutputOptions;
use crate::TableSortOptions;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::TskitError;
use crate::TskitTypeAccess;
use std::ptr::NonNull;

pub struct TableCollectionInterface {
    inner: NonNull<ll_bindings::tsk_table_collection_t>,
}

impl TskitTypeAccess<ll_bindings::tsk_table_collection_t> for TableCollectionInterface {
    fn as_ptr(&self) -> *const ll_bindings::tsk_table_collection_t {
        self.inner.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_table_collection_t {
        self.inner.as_ptr()
    }
}

impl TableCollectionInterface {
    pub(crate) fn new(ptr: *mut ll_bindings::tsk_table_collection_t) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(inner) => Some(Self { inner }),
            None => None,
        }
    }

    /// Length of the sequence/"genome".
    /// # Examples
    ///
    /// ```
    /// let tables = tskit::TableCollection::new(100.).unwrap();
    /// assert_eq!(tables.sequence_length(), 100.0);
    /// ```
    pub fn sequence_length(&self) -> Position {
        unsafe { (*self.as_ptr()).sequence_length }.into()
    }

    edge_table_add_row!(
    /// Add a row to the edge table
    ///
    /// # Examples
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    ///
    /// // left, right, parent, child
    /// match tables.add_edge(0., 53., 1, 11) {
    ///     // This is the first edge, so its id will be
    ///     // zero (0).
    ///     Ok(edge_id) => assert_eq!(edge_id, 0),
    ///     Err(e) => panic!("{:?}", e),
    /// }
    /// ```
    ///
    /// You may also use [`Position`] and [`NodeId`] as inputs.
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// let left = tskit::Position::from(0.0);
    /// let right = tskit::Position::from(53.0);
    /// let parent = tskit::NodeId::from(1);
    /// let child = tskit::NodeId::from(11);
    /// match tables.add_edge(left, right, parent, child) {
    ///     Ok(edge_id) => assert_eq!(edge_id, 0),
    ///     Err(e) => panic!("{:?}", e),
    /// }
    /// ```
    ///
    /// Adding invalid data is allowed at this point:
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// assert!(tables.add_edge(0., 53.,
    ///                         tskit::NodeId::NULL,
    ///                         tskit::NodeId::NULL).is_ok());
    /// # assert!(tables.check_integrity(tskit::TableIntegrityCheckFlags::default()).is_err());
    /// ```
    ///
    /// See [`TableCollection::check_integrity`] for how to catch these data model
    /// violations.
    => add_edge, self, (*self.as_mut_ptr()).edges);

    edge_table_add_row_with_metadata!(
    /// Add a row with optional metadata to the edge table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::EdgeMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct EdgeMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = EdgeMetadata{x: 1};
    /// assert!(tables.add_edge_with_metadata(0., 53., 1, 11, &metadata).is_ok());
    /// # }
    /// ```
    => add_edge_with_metadata, self, (*self.as_mut_ptr()).edges);

    individual_table_add_row!(
    /// Add a row to the individual table
    ///
    /// # Examples
    ///
    /// ## No flags, location, nor parents
    ///
    /// ```
    /// # use tskit::TableAccess;
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// tables.add_individual(0, None, None).unwrap();
    /// # assert!(tables.individuals().location(0).unwrap().is_none());
    /// # assert!(tables.individuals().parents(0).unwrap().is_none());
    /// ```
    ///
    /// ## No flags, a 3d location, no parents
    ///
    /// ```
    /// # use tskit::TableAccess;
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// tables.add_individual(0, &[-0.5, 0.3, 10.0], None).unwrap();
    /// # match tables.individuals().location(0).unwrap() {
    /// #     Some(loc) => loc.iter().zip([-0.5, 0.3, 10.0].iter()).for_each(|(a,b)| assert_eq!(a, b)),
    /// #     None => panic!("expected a location"),
    /// # }
    /// ```
    ///
    /// ## No flags, no location, two parents
    /// ```
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// # use tskit::TableAccess;
    /// tables.add_individual(0, None, &[1, 11]).unwrap();
    /// # match tables.individuals().parents(0).unwrap() {
    /// #     Some(parents) => parents.iter().zip([1, 11].iter()).for_each(|(a,b)| assert_eq!(a, b)),
    /// #     None => panic!("expected parents"),
    /// # }
    /// ```
    => add_individual, self, (*self.as_mut_ptr()).individuals);

    individual_table_add_row_with_metadata!(
    /// Add a row with metadata to the individual table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// use tskit::TableAccess;
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = IndividualMetadata{x: 1};
    /// assert!(tables.add_individual_with_metadata(0, None, None,
    ///                                             &metadata).is_ok());
    /// # let decoded = tables.individuals().metadata::<IndividualMetadata>(0.into()).unwrap().unwrap();
    /// # assert_eq!(decoded.x, 1);
    /// # }
    => add_individual_with_metadata, self, (*self.as_mut_ptr()).individuals);

    migration_table_add_row!(
    /// Add a row to the migration table
    ///
    /// # Warnings
    ///
    /// Migration tables are not currently supported
    /// by tree sequence simplification.
    /// # Examples
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// assert!(tables.add_migration((0.5, 100.0),
    ///                              3,
    ///                              (0, 1),
    ///                              53.5).is_ok());
    /// ```
    => add_migration, self, (*self.as_mut_ptr()).migrations);

    migration_table_add_row_with_metadata!(
    /// Add a row with optional metadata to the migration table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MigrationMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct MigrationMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = MigrationMetadata{x: 1};
    /// assert!(tables.add_migration_with_metadata((0.5, 100.0),
    ///                                            3,
    ///                                            (0, 1),
    ///                                            53.5,
    ///                                            &metadata).is_ok());
    /// # }
    /// ```
    ///
    /// # Warnings
    ///
    /// Migration tables are not currently supported
    /// by tree sequence simplification.
    => add_migration_with_metadata, self, (*self.as_mut_ptr()).migrations);

    node_table_add_row!(
    /// Add a row to the node table
    => add_node, self, (*self.as_mut_ptr()).nodes
    );

    node_table_add_row_with_metadata!(

    /// Add a row with optional metadata to the node table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::NodeMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct NodeMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = NodeMetadata{x: 1};
    /// assert!(tables.add_node_with_metadata(0, 0.0, -1, -1, &metadata).is_ok());
    /// # }
    /// ```
    => add_node_with_metadata,self, (*self.as_mut_ptr()).nodes);

    site_table_add_row!(
    /// Add a row to the site table
    => add_site, self, (*self.as_mut_ptr()).sites);

    site_table_add_row_with_metadata!(
    /// Add a row with optional metadata to the site table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::SiteMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct SiteMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = SiteMetadata{x: 1};
    /// assert!(tables.add_site_with_metadata(tskit::Position::from(111.0),
    ///                                       Some(&[111]),
    ///                                       &metadata).is_ok());
    /// # }
    /// ```
    => add_site_with_metadata, self, (*self.as_mut_ptr()).sites);

    mutation_table_add_row!(
    /// Add a row to the mutation table.
    => add_mutation, self, (*self.as_mut_ptr()).mutations);

    mutation_table_add_row_with_metadata!(
    /// Add a row with optional metadata to the mutation table.
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct MutationMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = MutationMetadata{x: 1};
    /// assert!(tables.add_mutation_with_metadata(0, 0, 0, 100.0, None,
    ///                                           &metadata).is_ok());
    /// # }
    /// ```
    => add_mutation_with_metadata, self, (*self.as_mut_ptr()).mutations);

    population_table_add_row!(
    /// Add a row to the population_table
    ///
    /// # Examples
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(55.0).unwrap();
    /// tables.add_population().unwrap();
    /// ```
    => add_population, self, (*self.as_mut_ptr()).populations);

    population_table_add_row_with_metadata!(
    /// Add a row with optional metadata to the population_table
    ///
    /// # Examples
    ///
    /// See [`metadata`](crate::metadata) for more details about required
    /// trait implementations.
    /// Those details have been omitted from this example.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::PopulationMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct PopulationMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = PopulationMetadata{x: 1};
    /// assert!(tables.add_population_with_metadata(&metadata).is_ok());
    /// # }
    => add_population_with_metadata, self, (*self.as_mut_ptr()).populations);

    /// Build the "input" and "output"
    /// indexes for the edge table.
    ///
    /// # Note
    ///
    /// The `C API` call behind this takes a `flags` argument
    /// that is currently unused.  A future release may break `API`
    /// here if the `C` library is updated to use flags.
    pub fn build_index(&mut self) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_build_index(self.as_mut_ptr(), 0) };
        handle_tsk_return_value!(rv)
    }

    /// Return `true` if tables are indexed.
    pub fn is_indexed(&self) -> bool {
        unsafe { ll_bindings::tsk_table_collection_has_index(self.as_ptr(), 0) }
    }

    /// If `self.is_indexed()` is `true`, return a non-owning
    /// slice containing the edge insertion order.
    /// Otherwise, return `None`.
    pub fn edge_insertion_order(&self) -> Option<&[EdgeId]> {
        if self.is_indexed() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    (*self.as_ptr()).indexes.edge_insertion_order as *const EdgeId,
                    usize::try_from((*self.as_ptr()).indexes.num_edges).unwrap(),
                )
            })
        } else {
            None
        }
    }

    /// If `self.is_indexed()` is `true`, return a non-owning
    /// slice containing the edge removal order.
    /// Otherwise, return `None`.
    pub fn edge_removal_order(&self) -> Option<&[EdgeId]> {
        if self.is_indexed() {
            Some(unsafe {
                std::slice::from_raw_parts(
                    (*self.as_ptr()).indexes.edge_removal_order as *const EdgeId,
                    usize::try_from((*self.as_ptr()).indexes.num_edges).unwrap(),
                )
            })
        } else {
            None
        }
    }

    /// Sort the tables.  
    /// The [``bookmark``](crate::types::Bookmark) can
    /// be used to affect where sorting starts from for each table.
    ///
    /// # Details
    ///
    /// See [`full_sort`](crate::TableCollection::full_sort)
    /// for more details about which tables are sorted.
    pub fn sort<O: Into<TableSortOptions>>(
        &mut self,
        start: &Bookmark,
        options: O,
    ) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_sort(
                self.as_mut_ptr(),
                &start.offsets,
                options.into().bits(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Fully sort all tables.
    /// Implemented via a call to [``sort``](crate::TableCollection::sort).
    ///
    /// # Details
    ///
    /// This function only sorts the tables that have a strict sortedness
    /// requirement according to the `tskit` [data
    /// model](https://tskit.dev/tskit/docs/stable/data-model.html).
    ///
    /// These tables are:
    ///
    /// * edges
    /// * mutations
    /// * sites
    ///
    /// For some use cases it is desirable to have the individual table
    /// sorted so that parents appear before offspring. See
    /// [``topological_sort_individuals``](crate::TableCollection::topological_sort_individuals).
    pub fn full_sort<O: Into<TableSortOptions>>(&mut self, options: O) -> TskReturnValue {
        let b = Bookmark::new();
        self.sort(&b, options)
    }

    /// Sorts the individual table in place, so that parents come before children,
    /// and the parent column is remapped as required. Node references to individuals
    /// are also updated.
    ///
    /// This function is needed because neither [``sort``](crate::TableCollection::sort) nor
    /// [``full_sort``](crate::TableCollection::full_sort) sorts
    /// the individual table!
    ///
    /// # Examples
    ///
    /// ```
    /// // Parent comes AFTER the child
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let i0 = tables.add_individual(0, None, &[1]).unwrap();
    /// assert_eq!(i0, 0);
    /// let i1 = tables.add_individual(0, None, None).unwrap();
    /// assert_eq!(i1, 1);
    /// let n0 = tables.add_node(0, 0.0, -1, i1).unwrap();
    /// assert_eq!(n0, 0);
    /// let n1 = tables.add_node(0, 1.0, -1, i0).unwrap();
    /// assert_eq!(n1, 1);
    ///
    /// // Testing for valid individual order will Err:
    /// match tables.check_integrity(tskit::TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING) {
    ///     Ok(_) => panic!("expected Err"),
    ///     Err(_) => (),
    /// };
    ///
    /// // The standard sort doesn't fix the Err...:
    /// tables.full_sort(tskit::TableSortOptions::default()).unwrap();
    /// match tables.check_integrity(tskit::TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING) {
    ///     Ok(_) => panic!("expected Err"),
    ///     Err(_) => (),
    /// };
    ///
    /// // ... so we need to intentionally sort the individuals.
    /// let _ = tables.topological_sort_individuals(tskit::IndividualTableSortOptions::default()).unwrap();
    /// tables.check_integrity(tskit::TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Will return an error code if the underlying `C` function returns an error.
    pub fn topological_sort_individuals<O: Into<IndividualTableSortOptions>>(
        &mut self,
        options: O,
    ) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_individual_topological_sort(
                self.as_mut_ptr(),
                options.into().bits(),
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Dump the table collection to file.
    ///
    /// # Panics
    ///
    /// This function allocates a `CString` to pass the file name to the C API.
    /// A panic will occur if the system runs out of memory.
    pub fn dump<O: Into<TableOutputOptions>>(&self, filename: &str, options: O) -> TskReturnValue {
        let c_str = std::ffi::CString::new(filename).unwrap();
        let rv = unsafe {
            ll_bindings::tsk_table_collection_dump(
                self.as_ptr(),
                c_str.as_ptr(),
                options.into().bits(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Clear the contents of all tables.
    /// Does not release memory.
    /// Memory will be released when the object goes out
    /// of scope.
    pub fn clear<O: Into<TableClearOptions>>(&mut self, options: O) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_clear(self.as_mut_ptr(), options.into().bits())
        };

        handle_tsk_return_value!(rv)
    }

    /// Free all memory allocated on the C side.
    /// Not public b/c not very safe.
    #[allow(dead_code)]
    fn free(&mut self) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_free(self.as_mut_ptr()) };

        handle_tsk_return_value!(rv)
    }

    /// Return ``true`` if ``self`` contains the same
    /// data as ``other``, and ``false`` otherwise.
    pub fn equals<O: Into<TableEqualityOptions>>(
        &self,
        other: &TableCollection,
        options: O,
    ) -> bool {
        unsafe {
            ll_bindings::tsk_table_collection_equals(
                self.as_ptr(),
                other.as_ptr(),
                options.into().bits(),
            )
        }
    }

    /// Return a "deep" copy of the tables.
    pub fn deepcopy(&self) -> Result<TableCollection, TskitError> {
        // The output is UNINITIALIZED tables,
        // else we leak memory
        let mut copy = crate::table_collection::uninit_table_collection();

        let rv = unsafe { ll_bindings::tsk_table_collection_copy(self.as_ptr(), &mut *copy, 0) };

        // SAFETY: we just initialized it.
        // The C API won't free NULL pointers.
        handle_tsk_return_value!(rv, unsafe { TableCollection::new_from_mbox(copy) })
    }

    /// Simplify tables in place.
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
    pub fn simplify<N: Into<NodeId>, O: Into<SimplificationOptions>>(
        &mut self,
        samples: &[N],
        options: O,
        idmap: bool,
    ) -> Result<Option<Vec<NodeId>>, TskitError> {
        let mut output_node_map: Vec<NodeId> = vec![];
        if idmap {
            let num_nodes: usize = unsafe { (*self.as_ptr()).nodes.num_rows }
                .try_into()
                .unwrap();
            output_node_map.resize(num_nodes, NodeId::NULL);
        }
        let rv = unsafe {
            ll_bindings::tsk_table_collection_simplify(
                self.as_mut_ptr(),
                samples.as_ptr().cast::<tsk_id_t>(),
                samples.len() as tsk_size_t,
                options.into().bits(),
                match idmap {
                    true => output_node_map.as_mut_ptr().cast::<tsk_id_t>(),
                    false => std::ptr::null_mut(),
                },
            )
        };
        handle_tsk_return_value!(
            rv,
            match idmap {
                true => Some(output_node_map),
                false => None,
            }
        )
    }

    /// Validate the contents of the table collection
    ///
    /// # Parameters
    ///
    /// `flags` is an instance of [`TableIntegrityCheckFlags`]
    ///
    /// # Return value
    ///
    /// `0` upon success, or an error code.
    /// However, if `flags` contains [`TableIntegrityCheckFlags::CHECK_TREES`],
    /// and no error is returned, then the return value is the number
    /// of trees.
    ///
    /// # Note
    ///
    /// Creating a [`crate::TreeSequence`] from a table collection will automatically
    /// run an integrity check.
    /// See [`TableCollection::tree_sequence`].
    ///
    /// # Examples
    ///
    /// There are many ways for a table colletion to be invalid.
    /// These examples are just the tip of the iceberg.
    ///
    /// ```should_panic
    /// let mut tables = tskit::TableCollection::new(10.0).unwrap();
    /// // Right position is > sequence_length
    /// tables.add_edge(0.0, 11.0, 0, 0);
    /// tables.check_integrity(tskit::TableIntegrityCheckFlags::default()).unwrap();
    /// ```
    ///
    /// ```should_panic
    /// # let mut tables = tskit::TableCollection::new(10.0).unwrap();
    /// // Left position is < 0.0
    /// tables.add_edge(-1., 10.0, 0, 0);
    /// tables.check_integrity(tskit::TableIntegrityCheckFlags::default()).unwrap();
    /// ```
    ///
    /// ```should_panic
    /// # let mut tables = tskit::TableCollection::new(10.0).unwrap();
    /// // Edges cannot have null node ids
    /// tables.add_edge(0., 10.0, tskit::NodeId::NULL, 0);
    /// tables.check_integrity(tskit::TableIntegrityCheckFlags::default()).unwrap();
    /// ```
    pub fn check_integrity(&self, flags: TableIntegrityCheckFlags) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_check_integrity(self.as_ptr(), flags.bits())
        };
        handle_tsk_return_value!(rv)
    }

    #[cfg(any(feature = "provenance", doc))]
    provenance_table_add_row!(
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
    /// ```
    /// use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// # #[cfg(feature = "provenance")] {
    /// tables.add_provenance(&String::from("Some provenance")).unwrap();
    ///
    /// // Get reference to the table
    /// let prov_ref = tables.provenances();
    ///
    /// // Get the first row
    /// let row_0 = prov_ref.row(0).unwrap();
    ///
    /// assert_eq!(row_0.record, "Some provenance");
    ///
    /// // Get the first record
    /// let record_0 = prov_ref.record(0).unwrap();
    /// assert_eq!(record_0, row_0.record);
    ///
    /// // Get the first time stamp
    /// let timestamp = prov_ref.timestamp(0).unwrap();
    /// assert_eq!(timestamp, row_0.timestamp);
    ///
    /// // You can get the `humantime::Timestamp` object back from the `String`:
    /// use core::str::FromStr;
    /// let timestamp_string = humantime::Timestamp::from_str(&timestamp).unwrap();
    ///
    /// // Provenance transfers to the tree sequences
    /// let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// assert_eq!(treeseq.provenances().record(0).unwrap(), "Some provenance");
    /// // We can still compare to row_0 because it is a copy of the row data:
    /// assert_eq!(treeseq.provenances().record(0).unwrap(), row_0.record);
    /// # }
    /// ```
    => add_provenance, self, (self.inner.as_mut()).provenances);

    /// Set the edge table from an [`OwnedEdgeTable`](`crate::OwnedEdgeTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut edges = tskit::OwnedEdgeTable::default();
    /// edges.add_row(0., 1., 0, 12).unwrap();
    /// tables.set_edges(&edges).unwrap();
    /// assert_eq!(tables.edges().num_rows(), 1);
    /// assert_eq!(tables.edges().child(0).unwrap(), 12);
    /// # edges.clear().unwrap();
    /// # assert_eq!(edges.num_rows(), 0);
    /// ```
    pub fn set_edges(&mut self, edges: &crate::OwnedEdgeTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_edge_table_set_columns(
                &mut (*self.as_mut_ptr()).edges,
                (*edges.as_ptr()).num_rows,
                (*edges.as_ptr()).left,
                (*edges.as_ptr()).right,
                (*edges.as_ptr()).parent,
                (*edges.as_ptr()).child,
                (*edges.as_ptr()).metadata,
                (*edges.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the node table from an [`OwnedNodeTable`](`crate::OwnedNodeTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut nodes = tskit::OwnedNodeTable::default();
    /// nodes.add_row(0, 10.0, -1, -1).unwrap();
    /// tables.set_nodes(&nodes).unwrap();
    /// assert_eq!(tables.nodes().num_rows(), 1);
    /// assert_eq!(tables.nodes().time(0).unwrap(), 10.0);
    /// # nodes.clear().unwrap();
    /// # assert_eq!(nodes.num_rows(), 0);
    /// ```
    pub fn set_nodes(&mut self, nodes: &crate::OwnedNodeTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_node_table_set_columns(
                &mut (*self.as_mut_ptr()).nodes,
                (*nodes.as_ptr()).num_rows,
                (*nodes.as_ptr()).flags,
                (*nodes.as_ptr()).time,
                (*nodes.as_ptr()).population,
                (*nodes.as_ptr()).individual,
                (*nodes.as_ptr()).metadata,
                (*nodes.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the site table from an [`OwnedSiteTable`](`crate::OwnedSiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut sites = tskit::OwnedSiteTable::default();
    /// sites.add_row(11.0, None).unwrap();
    /// tables.set_sites(&sites).unwrap();
    /// assert_eq!(tables.sites().num_rows(), 1);
    /// assert_eq!(tables.sites().position(0).unwrap(), 11.0);
    /// # sites.clear().unwrap();
    /// # assert_eq!(sites.num_rows(), 0);
    /// ```
    pub fn set_sites(&mut self, sites: &crate::OwnedSiteTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_site_table_set_columns(
                &mut (*self.as_mut_ptr()).sites,
                (*sites.as_ptr()).num_rows,
                (*sites.as_ptr()).position,
                (*sites.as_ptr()).ancestral_state,
                (*sites.as_ptr()).ancestral_state_offset,
                (*sites.as_ptr()).metadata,
                (*sites.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the mutation table from an [`OwnedMutationTable`](`crate::OwnedSiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut mutations = tskit::OwnedMutationTable::default();
    /// mutations.add_row(14, 12, -1, 11.3, None).unwrap();
    /// tables.set_mutations(&mutations).unwrap();
    /// assert_eq!(tables.mutations().num_rows(), 1);
    /// assert_eq!(tables.mutations().site(0).unwrap(), 14);
    /// # mutations.clear().unwrap();
    /// # assert_eq!(mutations.num_rows(), 0);
    /// ```
    pub fn set_mutations(&mut self, mutations: &crate::OwnedMutationTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_mutation_table_set_columns(
                &mut (*self.as_mut_ptr()).mutations,
                (*mutations.as_ptr()).num_rows,
                (*mutations.as_ptr()).site,
                (*mutations.as_ptr()).node,
                (*mutations.as_ptr()).parent,
                (*mutations.as_ptr()).time,
                (*mutations.as_ptr()).derived_state,
                (*mutations.as_ptr()).derived_state_offset,
                (*mutations.as_ptr()).metadata,
                (*mutations.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the individual table from an [`OwnedIndividualTable`](`crate::OwnedSiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut individuals = tskit::OwnedIndividualTable::default();
    /// individuals.add_row(0, [0.1, 10.0], None).unwrap();
    /// tables.set_individuals(&individuals).unwrap();
    /// assert_eq!(tables.individuals().num_rows(), 1);
    /// let expected = vec![tskit::Location::from(0.1), tskit::Location::from(10.0)];
    /// assert_eq!(tables.individuals().location(0).unwrap(), Some(expected.as_slice()));
    /// # individuals.clear().unwrap();
    /// # assert_eq!(individuals.num_rows(), 0);
    /// ```
    pub fn set_individuals(&mut self, individuals: &crate::OwnedIndividualTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_individual_table_set_columns(
                &mut (*self.as_mut_ptr()).individuals,
                (*individuals.as_ptr()).num_rows,
                (*individuals.as_ptr()).flags,
                (*individuals.as_ptr()).location,
                (*individuals.as_ptr()).location_offset,
                (*individuals.as_ptr()).parents,
                (*individuals.as_ptr()).parents_offset,
                (*individuals.as_ptr()).metadata,
                (*individuals.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the migration table from an [`OwnedMigrationTable`](`crate::OwnedSiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut migrations = tskit::OwnedMigrationTable::default();
    /// migrations.add_row((0.25, 0.37), 1, (0, 1), 111.0).unwrap();
    /// tables.set_migrations(&migrations).unwrap();
    /// assert_eq!(tables.migrations().num_rows(), 1);
    /// assert_eq!(tables.migrations().time(0).unwrap(), 111.0);
    /// # migrations.clear().unwrap();
    /// # assert_eq!(migrations.num_rows(), 0);
    /// ```
    pub fn set_migrations(&mut self, migrations: &crate::OwnedMigrationTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_migration_table_set_columns(
                &mut (*self.as_mut_ptr()).migrations,
                (*migrations.as_ptr()).num_rows,
                (*migrations.as_ptr()).left,
                (*migrations.as_ptr()).right,
                (*migrations.as_ptr()).node,
                (*migrations.as_ptr()).source,
                (*migrations.as_ptr()).dest,
                (*migrations.as_ptr()).time,
                (*migrations.as_ptr()).metadata,
                (*migrations.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the population table from an [`OwnedPopulationTable`](`crate::OwnedSiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut populations = tskit::OwnedPopulationTable::default();
    /// populations.add_row().unwrap();
    /// tables.set_populations(&populations).unwrap();
    /// assert_eq!(tables.populations().num_rows(), 1);
    /// # populations.clear().unwrap();
    /// # assert_eq!(populations.num_rows(), 0);
    /// ```
    pub fn set_populations(&mut self, populations: &crate::OwnedPopulationTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_population_table_set_columns(
                &mut (*self.as_mut_ptr()).populations,
                (*populations.as_ptr()).num_rows,
                (*populations.as_ptr()).metadata,
                (*populations.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    #[cfg(any(doc, feature = "provenance"))]
    /// Set the provenance table from an
    /// [`OwnedProvenanceTable`](`crate::provenance::OwnedProvenanceTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature="provenance")] {
    /// # use tskit::TableAccess;
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut provenances = tskit::provenance::OwnedProvenanceTable::default();
    /// provenances.add_row("I like pancakes").unwrap();
    /// tables.set_provenances(&provenances).unwrap();
    /// assert_eq!(tables.provenances().num_rows(), 1);
    /// assert_eq!(tables.provenances().record(0).unwrap(), "I like pancakes");
    /// # provenances.clear().unwrap();
    /// # assert_eq!(provenances.num_rows(), 0);
    /// # }
    /// ```
    pub fn set_provenances(
        &mut self,
        provenances: &crate::provenance::OwnedProvenanceTable,
    ) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_set_columns(
                &mut (self.inner.as_mut()).provenances,
                (*provenances.as_ptr()).num_rows,
                (*provenances.as_ptr()).timestamp,
                (*provenances.as_ptr()).timestamp_offset,
                (*provenances.as_ptr()).record,
                (*provenances.as_ptr()).record_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }
}
