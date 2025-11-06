use std::vec;

use crate::error::TskitError;
use crate::metadata::EdgeMetadata;
use crate::metadata::MigrationMetadata;
use crate::metadata::MutationMetadata;
use crate::metadata::PopulationMetadata;
use crate::metadata::SiteMetadata;
#[cfg(feature = "provenance")]
use crate::provenance::ProvenanceTable;
use crate::sys::bindings as ll_bindings;
use crate::sys::TableCollection as LLTableCollection;
use crate::types::Bookmark;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::IndividualTableSortOptions;
use crate::MigrationId;
use crate::MigrationTable;
use crate::MutationId;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationId;
use crate::PopulationTable;
use crate::Position;
use crate::SimplificationOptions;
use crate::SiteId;
use crate::SiteTable;
use crate::TableClearOptions;
use crate::TableEqualityOptions;
use crate::TableIntegrityCheckFlags;
use crate::TableOutputOptions;
use crate::TableSortOptions;
use crate::Time;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::{EdgeId, NodeId};
use ll_bindings::tsk_id_t;
use ll_bindings::tsk_size_t;

/// A table collection.
///
/// This is a thin wrapper around the C type defining
/// a table collection.
///
/// # See also
///
/// * [`metadata`](crate::metadata)
///
/// # Examples
///
/// ```
///
/// let mut tables = tskit::TableCollection::new(100.).unwrap();
/// assert_eq!(tables.sequence_length(), 100.);
///
/// // Adding edges:
///
/// let rv = tables.add_edge(0., 53., 1, 11).unwrap();
///
/// // Add node:
///
/// let rv = tables.add_node(0, 3.2, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
///
/// // Get immutable reference to edge table
/// let edges = tables.edges();
/// assert_eq!(edges.num_rows(), 1);
///
/// // Get immutable reference to node table
/// let nodes = tables.nodes();
/// assert_eq!(nodes.num_rows(), 1);
/// ```
///
pub struct TableCollection {
    inner: LLTableCollection,
    idmap: Vec<NodeId>,
    edges: EdgeTable,
    nodes: NodeTable,
    sites: SiteTable,
    mutations: MutationTable,
    individuals: IndividualTable,
    populations: PopulationTable,
    migrations: MigrationTable,
    #[cfg(feature = "provenance")]
    provenances: ProvenanceTable,
}

impl TableCollection {
    /// Create a new table collection with a sequence length.
    ///
    /// # Examples
    ///
    /// ```
    /// let tables = tskit::TableCollection::new(55.0).unwrap();
    /// ```
    ///
    /// Negative sequence lengths are errors:
    ///
    /// ```should_panic
    /// let tables = tskit::TableCollection::new(-55.0).unwrap();
    /// ```
    pub fn new<P: Into<Position>>(sequence_length: P) -> Result<Self, TskitError> {
        let mut inner = LLTableCollection::new(sequence_length.into().into())?;
        // SAFETY: all the casts to *mut Foo are coming in via an implicit
        // cast from &mut Foo, which means that the ptr cannot be NULL.
        // Further, successful creation of LLTableCollection means
        // that tables are initialized.
        // Finally, none of these variables will be pub directly other than
        // by reference.
        let edges = unsafe { crate::EdgeTable::new_from_table(inner.edges_mut())? };
        let nodes = unsafe { crate::NodeTable::new_from_table(inner.nodes_mut())? };
        let sites = unsafe { crate::SiteTable::new_from_table(inner.sites_mut())? };
        let mutations = unsafe { crate::MutationTable::new_from_table(inner.mutations_mut())? };
        let individuals =
            unsafe { crate::IndividualTable::new_from_table(inner.individuals_mut())? };
        let populations =
            unsafe { crate::PopulationTable::new_from_table(inner.populations_mut())? };
        let migrations = unsafe { crate::MigrationTable::new_from_table(inner.migrations_mut())? };
        #[cfg(feature = "provenance")]
        let provenances =
            unsafe { crate::provenance::ProvenanceTable::new_from_table(inner.provenances_mut())? };
        Ok(Self {
            inner,
            idmap: vec![],
            edges,
            nodes,
            sites,
            mutations,
            individuals,
            populations,
            migrations,
            #[cfg(feature = "provenance")]
            provenances,
        })
    }

    pub(crate) fn new_from_ll(lltables: LLTableCollection) -> Result<Self, TskitError> {
        let mut inner = lltables;
        // SAFETY: all the casts to *mut Foo are coming in via an implicit
        // cast from &mut Foo, which means that the ptr cannot be NULL.
        // Further, successful creation of LLTableCollection means
        // that tables are initialized.
        // Finally, none of these variables will be pub directly other than
        // by reference.
        let edges = unsafe { crate::EdgeTable::new_from_table(inner.edges_mut())? };
        let nodes = unsafe { crate::NodeTable::new_from_table(inner.nodes_mut())? };
        let sites = unsafe { crate::SiteTable::new_from_table(inner.sites_mut())? };
        let mutations = unsafe { crate::MutationTable::new_from_table(inner.mutations_mut())? };
        let individuals =
            unsafe { crate::IndividualTable::new_from_table(inner.individuals_mut())? };
        let populations =
            unsafe { crate::PopulationTable::new_from_table(inner.populations_mut())? };
        let migrations = unsafe { crate::MigrationTable::new_from_table(inner.migrations_mut())? };
        #[cfg(feature = "provenance")]
        let provenances =
            unsafe { crate::provenance::ProvenanceTable::new_from_table(inner.provenances_mut())? };
        Ok(Self {
            inner,
            idmap: vec![],
            edges,
            nodes,
            sites,
            mutations,
            individuals,
            populations,
            migrations,
            #[cfg(feature = "provenance")]
            provenances,
        })
    }

    pub(crate) fn into_inner(self) -> crate::sys::TableCollection {
        self.inner
    }

    /// Load a table collection from a file.
    ///
    /// # Examples
    ///
    /// The function is generic over references to `str`:
    ///
    /// ```
    /// # let empty_tables = tskit::TableCollection::new(100.).unwrap();
    /// # empty_tables.dump("trees.file", tskit::TableOutputOptions::default()).unwrap();
    /// let tables = tskit::TableCollection::new_from_file("trees.file").unwrap();
    ///
    /// let filename = String::from("trees.file");
    /// // Pass filename by reference
    /// let tables = tskit::TableCollection::new_from_file(&filename).unwrap();
    ///
    /// // Move filename
    /// let tables = tskit::TableCollection::new_from_file(filename).unwrap();
    ///
    /// // Boxed String are an unlikely use case, but can be made to work:
    /// let filename = Box::new(String::from("trees.file"));
    /// let tables = tskit::TableCollection::new_from_file(&*filename.as_ref()).unwrap();
    /// # std::fs::remove_file("trees.file").unwrap();
    /// ```
    ///
    /// # Panics
    ///
    /// This function allocates a `CString` to pass the file name to the C API.
    /// A panic will occur if the system runs out of memory.
    pub fn new_from_file(filename: impl AsRef<str>) -> Result<Self, TskitError> {
        // Arbitrary sequence_length.
        let mut tables = TableCollection::new(1.0)?;

        let c_str = std::ffi::CString::new(filename.as_ref()).map_err(|_| {
            TskitError::LibraryError("call to ffi::CString::new failed".to_string())
        })?;
        let rv = unsafe {
            ll_bindings::tsk_table_collection_load(
                tables.as_mut_ptr(),
                c_str.as_ptr(),
                ll_bindings::TSK_NO_INIT,
            )
        };

        handle_tsk_return_value!(rv, tables)
    }

    /// Length of the sequence/"genome".
    /// # Examples
    ///
    /// ```
    /// let tables = tskit::TableCollection::new(100.).unwrap();
    /// assert_eq!(tables.sequence_length(), 100.0);
    /// ```
    pub fn sequence_length(&self) -> Position {
        self.inner.sequence_length().into()
    }

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
    pub fn add_edge<L: Into<Position>, R: Into<Position>, P: Into<NodeId>, C: Into<NodeId>>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
    ) -> Result<EdgeId, TskitError> {
        self.edges_mut().add_row(left, right, parent, child)
    }

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
    pub fn add_edge_with_metadata<
        L: Into<Position>,
        R: Into<Position>,
        P: Into<NodeId>,
        C: Into<NodeId>,
        M: EdgeMetadata,
    >(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
        metadata: &M,
    ) -> Result<EdgeId, TskitError> {
        self.edges_mut()
            .add_row_with_metadata(left, right, parent, child, metadata)
    }

    /// Add a row to the individual table
    ///
    /// # Examples
    ///
    /// ## No flags, location, nor parents
    ///
    /// ```
    /// #
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// tables.add_individual(0, None, None).unwrap();
    /// # assert!(tables.individuals().location(0).is_none());
    /// # assert!(tables.individuals().parents(0).is_none());
    /// ```
    ///
    /// ## No flags, a 3d location, no parents
    ///
    /// ```
    /// #
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// tables.add_individual(0, &[-0.5, 0.3, 10.0], None).unwrap();
    /// # match tables.individuals().location(0) {
    /// #     Some(loc) => loc.iter().zip([-0.5, 0.3, 10.0].iter()).for_each(|(a,b)| assert_eq!(a, b)),
    /// #     None => panic!("expected a location"),
    /// # }
    /// ```
    ///
    /// ## No flags, no location, two parents
    /// ```
    /// # let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// #
    /// tables.add_individual(0, None, &[1, 11]);
    /// # match tables.individuals().parents(0) {
    /// #     Some(parents) => parents.iter().zip([1, 11].iter()).for_each(|(a,b)| assert_eq!(a, b)),
    /// #     None => panic!("expected parents"),
    /// # }
    /// ```
    pub fn add_individual<F, L, P>(
        &mut self,
        flags: F,
        location: L,
        parents: P,
    ) -> Result<crate::IndividualId, TskitError>
    where
        F: Into<crate::IndividualFlags>,
        L: crate::IndividualLocation,
        P: crate::IndividualParents,
    {
        self.individuals_mut().add_row(flags, location, parents)
    }

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
    ///
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #    x: i32,
    /// # }
    /// let metadata = IndividualMetadata{x: 1};
    /// assert!(tables.add_individual_with_metadata(0, None, None,
    ///                                             &metadata).is_ok());
    /// # let decoded = tables.individuals().metadata::<IndividualMetadata>(0).unwrap().unwrap();
    /// # assert_eq!(decoded.x, 1);
    /// # }
    pub fn add_individual_with_metadata<F, L, P, M>(
        &mut self,
        flags: F,
        location: L,
        parents: P,
        metadata: &M,
    ) -> Result<crate::IndividualId, TskitError>
    where
        F: Into<crate::IndividualFlags>,
        L: crate::IndividualLocation,
        P: crate::IndividualParents,
        M: crate::metadata::IndividualMetadata,
    {
        self.individuals_mut()
            .add_row_with_metadata(flags, location, parents, metadata)
    }

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
    pub fn add_migration<LEFT, RIGHT, N, SOURCE, DEST, T>(
        &mut self,
        span: (LEFT, RIGHT),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
    ) -> Result<MigrationId, TskitError>
    where
        LEFT: Into<Position>,
        RIGHT: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        T: Into<Time>,
    {
        self.migrations_mut().add_row(span, node, source_dest, time)
    }

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
    pub fn add_migration_with_metadata<LEFT, RIGHT, N, SOURCE, DEST, T, M>(
        &mut self,
        span: (LEFT, RIGHT),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
        metadata: &M,
    ) -> Result<MigrationId, TskitError>
    where
        LEFT: Into<Position>,
        RIGHT: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        T: Into<Time>,
        M: MigrationMetadata,
    {
        self.migrations_mut()
            .add_row_with_metadata(span, node, source_dest, time, metadata)
    }

    /// Add a row to the node table
    pub fn add_node<F, T, P, I>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
    {
        self.nodes_mut()
            .add_row(flags, time, population, individual)
    }

    /// Add a node using default values
    ///
    /// # Examples
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(1.).unwrap();
    /// let node_defaults = tskit::NodeDefaults::default();
    /// let rv = tables.add_node_with_defaults(1.0, &node_defaults).unwrap();
    /// assert_eq!(rv, 0);
    /// let rv = tables.add_node_with_defaults(2.0, &node_defaults).unwrap();
    /// assert_eq!(rv, 1);
    /// ```
    pub fn add_node_with_defaults<T: Into<crate::Time>, D: crate::node_table::DefaultNodeData>(
        &mut self,
        time: T,
        defaults: &D,
    ) -> Result<NodeId, TskitError> {
        self.nodes_mut().add_row_with_defaults(time, defaults)
    }

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
    pub fn add_node_with_metadata<F, T, P, I, N>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
        metadata: &N,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
        N: crate::metadata::NodeMetadata,
    {
        self.nodes_mut()
            .add_row_with_metadata(flags, time, population, individual, metadata)
    }

    /// Add a row to the site table
    pub fn add_site<P: Into<Position>>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
    ) -> Result<SiteId, TskitError> {
        self.sites_mut().add_row(position, ancestral_state)
    }

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
    pub fn add_site_with_metadata<P: Into<Position>, M: SiteMetadata>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
        metadata: &M,
    ) -> Result<SiteId, TskitError> {
        self.sites_mut()
            .add_row_with_metadata(position, ancestral_state, metadata)
    }

    /// Add a row to the mutation table.
    pub fn add_mutation<S, N, P, T>(
        &mut self,
        site: S,
        node: N,
        parent: P,
        time: T,
        derived_state: Option<&[u8]>,
    ) -> Result<MutationId, TskitError>
    where
        S: Into<SiteId>,
        N: Into<NodeId>,
        P: Into<MutationId>,
        T: Into<Time>,
    {
        self.mutations_mut()
            .add_row(site, node, parent, time, derived_state)
    }

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
    pub fn add_mutation_with_metadata<S, N, P, T, M>(
        &mut self,
        site: S,
        node: N,
        parent: P,
        time: T,
        derived_state: Option<&[u8]>,
        metadata: &M,
    ) -> Result<MutationId, TskitError>
    where
        S: Into<SiteId>,
        N: Into<NodeId>,
        P: Into<MutationId>,
        T: Into<Time>,
        M: MutationMetadata,
    {
        self.mutations_mut().add_row_with_metadata(
            site,
            node,
            parent,
            time,
            derived_state,
            metadata,
        )
    }

    /// Add a row to the population_table
    ///
    /// # Examples
    ///
    /// ```
    /// # let mut tables = tskit::TableCollection::new(55.0).unwrap();
    /// tables.add_population().unwrap();
    /// ```
    pub fn add_population(&mut self) -> Result<PopulationId, TskitError> {
        self.populations_mut().add_row()
    }

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
    pub fn add_population_with_metadata<M: PopulationMetadata>(
        &mut self,
        metadata: &M,
    ) -> Result<PopulationId, TskitError> {
        self.populations_mut().add_row_with_metadata(metadata)
    }

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
                    usize::try_from((*self.as_ptr()).indexes.num_edges).ok()?,
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
                    usize::try_from((*self.as_ptr()).indexes.num_edges).ok()?,
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
        let c_str = std::ffi::CString::new(filename).map_err(|_| {
            TskitError::LibraryError("call to ffi::CString::new failed".to_string())
        })?;
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
        let (rv, inner) = self.inner.copy();
        let tables = TableCollection::new_from_ll(inner)?;
        handle_tsk_return_value!(rv, tables)
    }

    /// Return a [`crate::TreeSequence`] based on the tables.
    /// This function will raise errors if tables are not sorted,
    /// not indexed, or invalid in any way.
    pub fn tree_sequence<F: Into<TreeSequenceFlags>>(
        self,
        flags: F,
    ) -> Result<crate::TreeSequence, TskitError> {
        crate::TreeSequence::new(self, flags.into())
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
    pub fn simplify<O: Into<SimplificationOptions>>(
        &mut self,
        samples: &[NodeId],
        options: O,
        idmap: bool,
    ) -> Result<Option<&[NodeId]>, TskitError> {
        if idmap {
            self.idmap
                .resize(usize::try_from(self.nodes().num_rows())?, NodeId::NULL);
        }
        let rv = unsafe {
            ll_bindings::tsk_table_collection_simplify(
                self.as_mut_ptr(),
                samples.as_ptr().cast::<tsk_id_t>(),
                samples.len() as tsk_size_t,
                options.into().bits(),
                match idmap {
                    true => self.idmap.as_mut_ptr().cast::<tsk_id_t>(),
                    false => std::ptr::null_mut(),
                },
            )
        };
        handle_tsk_return_value!(
            rv,
            match idmap {
                true => Some(&self.idmap),
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
    pub fn check_integrity<F: Into<TableIntegrityCheckFlags>>(&self, flags: F) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_check_integrity(self.as_ptr(), flags.into().bits())
        };
        handle_tsk_return_value!(rv)
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
    /// ```
    ///
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
    /// // You can get the `chrono` object back from the `String`:
    /// use core::str::FromStr;
    /// let timestamp_string = chrono::DateTime::<chrono::Utc>::from_str(&timestamp).unwrap();
    ///
    /// // Provenance transfers to the tree sequences
    /// let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// assert_eq!(treeseq.provenances().record(0).unwrap(), "Some provenance");
    /// // We can still compare to row_0 because it is a copy of the row data:
    /// assert_eq!(treeseq.provenances().record(0).unwrap(), row_0.record);
    /// # }
    /// ```
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        self.provenances_mut().add_row(record)
    }

    /// Set the edge table from an [`EdgeTable`](`crate::EdgeTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut edges = tskit::EdgeTable::default();
    /// edges.add_row(0., 1., 0, 12).unwrap();
    /// tables.set_edges(&edges).unwrap();
    /// assert_eq!(tables.edges().num_rows(), 1);
    /// assert_eq!(tables.edges().child(0).unwrap(), 12);
    /// # edges.clear().unwrap();
    /// # assert_eq!(edges.num_rows(), 0);
    /// ```
    ///
    /// The borrow checker will prevent ill-advised operations:
    ///
    /// ```compile_fail
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// tables.set_edges(&tables.edges()).unwrap();
    /// ```
    pub fn set_edges(&mut self, edges: &crate::EdgeTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_edge_table_set_columns(
                self.inner.edges_mut(),
                (edges.as_ref()).num_rows,
                (edges.as_ref()).left,
                (edges.as_ref()).right,
                (edges.as_ref()).parent,
                (edges.as_ref()).child,
                (edges.as_ref()).metadata,
                (edges.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the node table from an [`NodeTable`](`crate::NodeTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut nodes = tskit::NodeTable::default();
    /// nodes.add_row(0, 10.0, -1, -1).unwrap();
    /// tables.set_nodes(&nodes).unwrap();
    /// assert_eq!(tables.nodes().num_rows(), 1);
    /// assert_eq!(tables.nodes().time(0).unwrap(), 10.0);
    /// # nodes.clear().unwrap();
    /// # assert_eq!(nodes.num_rows(), 0);
    /// ```
    pub fn set_nodes(&mut self, nodes: &crate::NodeTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_node_table_set_columns(
                self.inner.nodes_mut(),
                (nodes.as_ref()).num_rows,
                (nodes.as_ref()).flags,
                (nodes.as_ref()).time,
                (nodes.as_ref()).population,
                (nodes.as_ref()).individual,
                (nodes.as_ref()).metadata,
                (nodes.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the site table from an [`SiteTable`](`crate::SiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut sites = tskit::SiteTable::default();
    /// sites.add_row(11.0, None).unwrap();
    /// tables.set_sites(&sites).unwrap();
    /// assert_eq!(tables.sites().num_rows(), 1);
    /// assert_eq!(tables.sites().position(0).unwrap(), 11.0);
    /// # sites.clear().unwrap();
    /// # assert_eq!(sites.num_rows(), 0);
    /// ```
    pub fn set_sites(&mut self, sites: &crate::SiteTable) -> TskReturnValue {
        // SAFETY: neither self nor sites are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_site_table_set_columns(
                self.inner.sites_mut(),
                (sites.as_ref()).num_rows,
                (sites.as_ref()).position,
                (sites.as_ref()).ancestral_state,
                (sites.as_ref()).ancestral_state_offset,
                (sites.as_ref()).metadata,
                (sites.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the mutation table from an [`MutationTable`](`crate::MutationTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut mutations = tskit::MutationTable::default();
    /// mutations.add_row(14, 12, -1, 11.3, None).unwrap();
    /// tables.set_mutations(&mutations).unwrap();
    /// assert_eq!(tables.mutations().num_rows(), 1);
    /// assert_eq!(tables.mutations().site(0).unwrap(), 14);
    /// # mutations.clear().unwrap();
    /// # assert_eq!(mutations.num_rows(), 0);
    /// ```
    pub fn set_mutations(&mut self, mutations: &crate::MutationTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_mutation_table_set_columns(
                self.inner.mutations_mut(),
                (mutations.as_ref()).num_rows,
                (mutations.as_ref()).site,
                (mutations.as_ref()).node,
                (mutations.as_ref()).parent,
                (mutations.as_ref()).time,
                (mutations.as_ref()).derived_state,
                (mutations.as_ref()).derived_state_offset,
                (mutations.as_ref()).metadata,
                (mutations.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the individual table from an [`IndividualTable`](`crate::IndividualTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut individuals = tskit::IndividualTable::default();
    /// individuals.add_row(0, [0.1, 10.0], None).unwrap();
    /// tables.set_individuals(&individuals).unwrap();
    /// assert_eq!(tables.individuals().num_rows(), 1);
    /// let expected = vec![tskit::Location::from(0.1), tskit::Location::from(10.0)];
    /// assert_eq!(tables.individuals().location(0), Some(expected.as_slice()));
    /// # individuals.clear().unwrap();
    /// # assert_eq!(individuals.num_rows(), 0);
    /// ```
    pub fn set_individuals(&mut self, individuals: &crate::IndividualTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_individual_table_set_columns(
                self.inner.individuals_mut(),
                (individuals.as_ref()).num_rows,
                (individuals.as_ref()).flags,
                (individuals.as_ref()).location,
                (individuals.as_ref()).location_offset,
                (individuals.as_ref()).parents,
                (individuals.as_ref()).parents_offset,
                (individuals.as_ref()).metadata,
                (individuals.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the migration table from an [`MigrationTable`](`crate::MigrationTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut migrations = tskit::MigrationTable::default();
    /// migrations.add_row((0.25, 0.37), 1, (0, 1), 111.0).unwrap();
    /// tables.set_migrations(&migrations).unwrap();
    /// assert_eq!(tables.migrations().num_rows(), 1);
    /// assert_eq!(tables.migrations().time(0).unwrap(), 111.0);
    /// # migrations.clear().unwrap();
    /// # assert_eq!(migrations.num_rows(), 0);
    /// ```
    pub fn set_migrations(&mut self, migrations: &crate::MigrationTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_migration_table_set_columns(
                self.inner.migrations_mut(),
                (migrations.as_ref()).num_rows,
                (migrations.as_ref()).left,
                (migrations.as_ref()).right,
                (migrations.as_ref()).node,
                (migrations.as_ref()).source,
                (migrations.as_ref()).dest,
                (migrations.as_ref()).time,
                (migrations.as_ref()).metadata,
                (migrations.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Set the population table from an [`PopulationTable`](`crate::SiteTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut populations = tskit::PopulationTable::default();
    /// populations.add_row().unwrap();
    /// tables.set_populations(&populations).unwrap();
    /// assert_eq!(tables.populations().num_rows(), 1);
    /// # populations.clear().unwrap();
    /// # assert_eq!(populations.num_rows(), 0);
    /// ```
    pub fn set_populations(&mut self, populations: &crate::PopulationTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_population_table_set_columns(
                self.inner.populations_mut(),
                (populations.as_ref()).num_rows,
                (populations.as_ref()).metadata,
                (populations.as_ref()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Set the provenance table from an
    /// [`ProvenanceTable`](`crate::provenance::ProvenanceTable`)
    ///
    /// # Errors
    ///
    /// Any errors from the C API propagate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature="provenance")] {
    /// #
    /// let mut tables = tskit::TableCollection::new(1.0).unwrap();
    /// let mut provenances = tskit::provenance::ProvenanceTable::default();
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
        provenances: &crate::provenance::ProvenanceTable,
    ) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_set_columns(
                self.inner.provenances_mut(),
                (provenances.as_ref()).num_rows,
                (provenances.as_ref()).timestamp,
                (provenances.as_ref()).timestamp_offset,
                (provenances.as_ref()).record,
                (provenances.as_ref()).record_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    pub fn edges(&self) -> &EdgeTable {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut EdgeTable {
        &mut self.edges
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    pub fn nodes(&self) -> &NodeTable {
        &self.nodes
    }

    /// Get mutable reference to the [``NodeTable``](crate::NodeTable).
    pub fn nodes_mut(&mut self) -> &mut NodeTable {
        &mut self.nodes
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    pub fn sites(&self) -> &SiteTable {
        &self.sites
    }

    pub fn sites_mut(&mut self) -> &mut SiteTable {
        &mut self.sites
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    pub fn mutations(&self) -> &MutationTable {
        &self.mutations
    }

    pub fn mutations_mut(&mut self) -> &mut MutationTable {
        &mut self.mutations
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    pub fn individuals(&self) -> &IndividualTable {
        &self.individuals
    }

    pub fn individuals_mut(&mut self) -> &mut IndividualTable {
        &mut self.individuals
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    pub fn populations(&self) -> &PopulationTable {
        &self.populations
    }

    pub fn populations_mut(&mut self) -> &mut PopulationTable {
        &mut self.populations
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    pub fn migrations(&self) -> &MigrationTable {
        &self.migrations
    }

    pub fn migrations_mut(&mut self) -> &mut MigrationTable {
        &mut self.migrations
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances(&self) -> &ProvenanceTable {
        &self.provenances
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances_mut(&mut self) -> &mut ProvenanceTable {
        &mut self.provenances
    }

    /// Return an iterator over the edges.
    pub fn edges_iter(&self) -> impl Iterator<Item = crate::EdgeTableRow> + '_ {
        self.edges.iter()
    }

    /// Return an iterator over the nodes.
    pub fn nodes_iter(&self) -> impl Iterator<Item = crate::NodeTableRow> + '_ {
        self.nodes.iter()
    }

    /// Return an iterator over the sites.
    pub fn sites_iter(&self) -> impl Iterator<Item = crate::SiteTableRow> + '_ {
        self.sites.iter()
    }

    /// Return an iterator over the mutations.
    pub fn mutations_iter(&self) -> impl Iterator<Item = crate::MutationTableRow> + '_ {
        self.mutations.iter()
    }

    /// Return an iterator over the individuals.
    pub fn individuals_iter(&self) -> impl Iterator<Item = crate::IndividualTableRow> + '_ {
        self.individuals.iter()
    }

    /// Return an iterator over the populations.
    pub fn populations_iter(&self) -> impl Iterator<Item = crate::PopulationTableRow> + '_ {
        self.populations.iter()
    }

    /// Return an iterator over the migrations.
    pub fn migrations_iter(&self) -> impl Iterator<Item = crate::MigrationTableRow> + '_ {
        self.migrations.iter()
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Return an iterator over provenances
    pub fn provenances_iter(
        &self,
    ) -> impl Iterator<Item = crate::provenance::ProvenanceTableRow> + '_ {
        self.provenances.iter()
    }

    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::NodeFlags::is_sample`]
    /// is `true`.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::samples_as_vector`].
    pub fn samples_as_vector(&self) -> Vec<crate::NodeId> {
        self.nodes().samples_as_vector()
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
        self.nodes().create_node_id_vector(f)
    }

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_table_collection_t {
        self.inner.as_ptr()
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_table_collection_t {
        self.inner.as_mut_ptr()
    }

    /// Truncate the [TableCollection] to specified genome intervals.
    ///
    /// # Return value
    /// - `Ok(None)`: when truncation leads to empty edge table.
    /// - `Ok(Some(TableCollection))`: when trunction is successfully performed
    ///   and results in non-empty edge table. The table collection is sorted.
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
    /// # use tskit::*;
    /// # let snode = NodeFlags::new_sample();
    /// # let anode = NodeFlags::default();
    /// # let pop = PopulationId::NULL;
    /// # let ind = IndividualId::NULL;
    /// # let seqlen = 100.0;
    /// # let (t0, t10) = (0.0, 10.0);
    /// # let (left, right) = (0.0, 100.0);
    /// # let sim_opts = SimplificationOptions::default();
    /// #
    /// # let mut tables = TableCollection::new(seqlen).unwrap();
    /// # let child1 = tables.add_node(snode, t0, pop, ind).unwrap();
    /// # let child2 = tables.add_node(snode, t0, pop, ind).unwrap();
    /// # let parent = tables.add_node(anode, t10, pop, ind).unwrap();
    /// #
    /// # tables.add_edge(left, right, parent, child1).unwrap();
    /// # tables.add_edge(left, right, parent, child2).unwrap();
    /// # tables.full_sort(TableSortOptions::all()).unwrap();
    /// # tables.simplify(&[child1, child2], sim_opts, false).unwrap();
    /// # tables.build_index().unwrap();
    /// #
    /// let intervals = [(0.0, 10.0), (90.0, 100.0)].into_iter();
    /// tables.keep_intervals(intervals).unwrap().unwrap();
    /// ```
    ///
    /// Note that no new provenance will be appended.
    pub fn keep_intervals<P>(
        self,
        intervals: impl Iterator<Item = (P, P)>,
    ) -> Result<Option<Self>, TskitError>
    where
        P: Into<Position>,
    {
        use crate::StreamingIterator;
        let mut tables = self;
        // use tables from sys to allow easier process with metadata
        let options = 0;
        let mut new_edges = crate::sys::EdgeTable::new(options)?;
        let mut new_migrations = crate::sys::MigrationTable::new(options)?;
        let mut new_sites = crate::sys::SiteTable::new(options)?;
        let mut new_mutations = crate::sys::MutationTable::new(options)?;

        // for old site id to new site id mapping
        let mut site_map = vec![-1i32; tables.sites().num_rows().as_usize()];

        // logicals to indicate whether a site (old) will be kept in new site table
        let mut keep_sites = vec![false; tables.sites().num_rows().try_into()?];

        let mut last_interval = (Position::from(0.0), Position::from(0.0));
        for (s, e) in intervals {
            let (s, e) = (s.into(), e.into());
            // make sure intervals are sorted
            if (s > e) || (s < last_interval.1) {
                return Err(TskitError::RangeError(
                    "intervals not valid or sorted".into(),
                ));
            }
            keep_sites
                .iter_mut()
                .zip(tables.sites_iter())
                .for_each(|(k, site_row)| {
                    *k = *k || ((site_row.position >= s) && (site_row.position < e));
                });

            // use stream_iter and while-let pattern for easier ? operator within a loop
            let mut edge_iter = tables
                .edges()
                .lending_iter()
                .filter(|edge_row| !((edge_row.right <= s) || (edge_row.left >= e)));

            while let Some(edge_row) = edge_iter.next() {
                new_edges.add_row_with_metadata(
                    if edge_row.left < s { s } else { edge_row.left }.into(),
                    if edge_row.right > e {
                        e
                    } else {
                        edge_row.right
                    }
                    .into(),
                    edge_row.parent.into(),
                    edge_row.child.into(),
                    edge_row.metadata.unwrap_or(&[0u8; 0]),
                )?;
            }

            let mut migration_iter = tables
                .migrations()
                .lending_iter()
                .filter(|mrow| !((mrow.right <= s) || (mrow.left >= e)));

            while let Some(migration_row) = migration_iter.next() {
                new_migrations.add_row_with_metadata(
                    (migration_row.left.into(), migration_row.right.into()),
                    migration_row.node.into(),
                    migration_row.source.into(),
                    migration_row.dest.into(),
                    migration_row.time.into(),
                    migration_row.metadata.unwrap_or(&[0u8; 0]),
                )?;
            }
            last_interval = (s, e);
        }

        let mut running_site_id = 0;
        let mut site_iter = tables.sites().lending_iter();
        while let Some(site_row) = site_iter.next() {
            let old_id = site_row.id.to_usize().unwrap();
            if keep_sites[old_id] {
                new_sites.add_row_with_metadata(
                    site_row.position.into(),
                    site_row.ancestral_state,
                    site_row.metadata.unwrap_or(&[0u8; 0]),
                )?;
                site_map[old_id] = running_site_id;
                running_site_id += 1;
            }
        }

        // build mutation_map
        let mutation_map: Vec<_> = {
            let mut n = 0;
            tables
                .mutations()
                .site_slice()
                .iter()
                .map(|site| {
                    if keep_sites[site.as_usize()] {
                        n += 1
                    };
                    n - 1
                })
                .collect()
        };

        let mut mutations_iter = tables.mutations().lending_iter();
        while let Some(mutation_row) = mutations_iter.next() {
            let old_id = mutation_row.site.to_usize().unwrap();
            if keep_sites[old_id] {
                let new_site = site_map[old_id];
                let new_parent = {
                    if mutation_row.parent.is_null() {
                        mutation_row.parent.into()
                    } else {
                        mutation_map[mutation_row.parent.as_usize()]
                    }
                };
                new_mutations.add_row_with_metadata(
                    new_site,
                    mutation_row.node.into(),
                    new_parent,
                    mutation_row.time.into(),
                    mutation_row.derived_state,
                    mutation_row.metadata.unwrap_or(&[0u8; 0]),
                )?;
            }
        }

        // convert sys version of tables to non-sys version of tables
        // SAFETY: all the casts to *mut Foo are coming in via an implicit
        // cast from &mut Foo, which means that the ptr cannot be NULL.
        // Further, all input tables are initialized.
        // Finally, none of these variables will be every be pub.
        let new_edges = unsafe { EdgeTable::new_from_table(new_edges.as_mut())? };
        let new_migrations = unsafe { MigrationTable::new_from_table(new_migrations.as_mut())? };
        let new_mutations = unsafe { MutationTable::new_from_table(new_mutations.as_mut())? };
        let new_sites = unsafe { SiteTable::new_from_table(new_sites.as_mut())? };

        // replace old tables with new tables
        tables.set_edges(&new_edges).map(|_| ())?;
        tables.set_migrations(&new_migrations).map(|_| ())?;
        tables.set_mutations(&new_mutations).map(|_| ())?;
        tables.set_sites(&new_sites)?;

        // sort tables
        tables.full_sort(TableSortOptions::default())?;

        // return None when edge table is empty
        if tables.edges().num_rows() == 0 {
            Ok(None)
        } else {
            Ok(Some(tables))
        }
    }

    /// Compute the parents of each mutation.
    pub fn compute_mutation_parents(
        &mut self,
        options: crate::MutationParentsFlags,
    ) -> Result<(), TskitError> {
        self.inner.compute_mutation_parents(options)
    }
}
