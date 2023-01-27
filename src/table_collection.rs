use delegate::delegate;
use std::vec;

use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::types::Bookmark;
use crate::IndividualTableSortOptions;
use crate::Position;
use crate::SimplificationOptions;
use crate::TableClearOptions;
use crate::TableEqualityOptions;
use crate::TableIntegrityCheckFlags;
use crate::TableOutputOptions;
use crate::TableSortOptions;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::{tsk_id_t, tsk_size_t};
use crate::{EdgeId, NodeId};
use ll_bindings::tsk_table_collection_free;
use mbox::MBox;

/// A table collection.
///
/// This is a thin wrapper around the C type
/// [`tsk_table_collection_t`](crate::bindings::tsk_table_collection_t).
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
    inner: MBox<ll_bindings::tsk_table_collection_t>,
    idmap: Vec<NodeId>,
    views: crate::table_views::TableViews,
}

impl Drop for TableCollection {
    fn drop(&mut self) {
        let rv = unsafe { tsk_table_collection_free(self.as_mut_ptr()) };
        assert_eq!(rv, 0);
    }
}

/// Returns a pointer to an uninitialized tsk_table_collection_t
pub(crate) fn uninit_table_collection() -> MBox<ll_bindings::tsk_table_collection_t> {
    let temp = unsafe {
        libc::malloc(std::mem::size_of::<ll_bindings::tsk_table_collection_t>())
            as *mut ll_bindings::tsk_table_collection_t
    };
    let nonnull = match std::ptr::NonNull::<ll_bindings::tsk_table_collection_t>::new(temp) {
        Some(x) => x,
        None => panic!("out of memory"),
    };
    unsafe { MBox::from_non_null_raw(nonnull) }
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
    /// ```{should_panic}
    /// let tables = tskit::TableCollection::new(-55.0).unwrap();
    /// ```
    pub fn new<P: Into<Position>>(sequence_length: P) -> Result<Self, TskitError> {
        let sequence_length = sequence_length.into();
        if sequence_length <= 0. {
            return Err(TskitError::ValueError {
                got: f64::from(sequence_length).to_string(),
                expected: "sequence_length >= 0.0".to_string(),
            });
        }
        let mut mbox = uninit_table_collection();
        let rv = unsafe { ll_bindings::tsk_table_collection_init(&mut *mbox, 0) };
        if rv < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: rv });
        }
        let views = crate::table_views::TableViews::new_from_mbox_table_collection(&mut mbox)?;
        // AHA?
        assert!(std::ptr::eq(
            &mbox.as_ref().edges as *const ll_bindings::tsk_edge_table_t,
            views.edges().as_ref() as *const ll_bindings::tsk_edge_table_t
        ));
        let mut tables = Self {
            inner: mbox,
            idmap: vec![],
            views,
        };
        unsafe {
            (*tables.as_mut_ptr()).sequence_length = f64::from(sequence_length);
        }
        Ok(tables)
    }

    /// # Safety
    ///
    /// It is possible that the mbox's inner pointer has not be run through
    /// tsk_table_collection_init, meaning that it is in an uninitialized state.
    /// Or, it may be initialized and about to be used in a part of the C API
    /// requiring an uninitialized table collection.
    /// Consult the C API docs before using!
    pub(crate) unsafe fn new_from_mbox(
        mbox: MBox<ll_bindings::tsk_table_collection_t>,
    ) -> Result<Self, TskitError> {
        let mut mbox = mbox;
        let views = crate::table_views::TableViews::new_from_mbox_table_collection(&mut mbox)?;
        Ok(Self {
            inner: mbox,
            idmap: vec![],
            views,
        })
    }

    pub(crate) fn into_raw(self) -> Result<*mut ll_bindings::tsk_table_collection_t, TskitError> {
        let mut tables = self;
        // rust won't let use move inner out b/c this type implements Drop.
        // So we have to replace the existing pointer with a new one.
        let table_ptr = unsafe {
            libc::malloc(std::mem::size_of::<ll_bindings::tsk_table_collection_t>())
                as *mut ll_bindings::tsk_table_collection_t
        };
        let rv = unsafe { ll_bindings::tsk_table_collection_init(table_ptr, 0) };

        let mut temp = unsafe { MBox::from_raw(table_ptr) };
        std::mem::swap(&mut temp, &mut tables.inner);
        handle_tsk_return_value!(rv, MBox::into_raw(temp))
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
        let mut tables = match TableCollection::new(1.0) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

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
        unsafe { (*self.as_ptr()).sequence_length }.into()
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
    pub fn add_edge<L, R, P, C>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
    ) -> Result<crate::EdgeId, crate::TskitError>
    where
        L: Into<crate::Position>,
        R: Into<crate::Position>,
        P: Into<crate::NodeId>,
        C: Into<crate::NodeId>,
    {
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
    pub fn add_edge_with_metadata<L, R, P, C, M>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
        metadata: &M,
    ) -> Result<crate::EdgeId, crate::TskitError>
    where
        L: Into<crate::Position>,
        R: Into<crate::Position>,
        P: Into<crate::NodeId>,
        C: Into<crate::NodeId>,
        M: crate::metadata::EdgeMetadata,
    {
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
    ) -> Result<crate::IndividualId, crate::TskitError>
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
    /// # let decoded = tables.individuals().metadata::<IndividualMetadata>(0.into()).unwrap().unwrap();
    /// # assert_eq!(decoded.x, 1);
    /// # }
    pub fn add_individual_with_metadata<F, L, P, M>(
        &mut self,
        flags: F,
        location: L,
        parents: P,
        metadata: &M,
    ) -> Result<crate::IndividualId, crate::TskitError>
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
    pub fn add_migration<L, R, N, S, D, T>(
        &mut self,
        span: (L, R),
        node: N,
        source_dest: (S, D),
        time: T,
    ) -> Result<crate::MigrationId, TskitError>
    where
        L: Into<Position>,
        R: Into<Position>,
        N: Into<NodeId>,
        S: Into<crate::PopulationId>,
        D: Into<crate::PopulationId>,
        T: Into<crate::Time>,
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
    pub fn add_migration_with_metadata<L, R, N, S, D, T, M>(
        &mut self,
        span: (L, R),
        node: N,
        source_dest: (S, D),
        time: T,
        metadata: &M,
    ) -> Result<crate::MigrationId, TskitError>
    where
        L: Into<Position>,
        R: Into<Position>,
        N: Into<NodeId>,
        S: Into<crate::PopulationId>,
        D: Into<crate::PopulationId>,
        T: Into<crate::Time>,
        M: crate::metadata::MigrationMetadata,
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
        self.views
            .nodes_mut()
            .add_row(flags, time, population, individual)
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
    pub fn add_node_with_metadata<F, T, P, I, M>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
        metadata: &M,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
        M: crate::metadata::NodeMetadata,
    {
        self.views
            .nodes_mut()
            .add_row_with_metadata(flags, time, population, individual, metadata)
    }

    /// Add a row to the site table
    pub fn add_site<P>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
    ) -> Result<crate::SiteId, TskitError>
    where
        P: Into<Position>,
    {
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
    pub fn add_site_with_metadata<P, M>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
        metadata: &M,
    ) -> Result<crate::SiteId, TskitError>
    where
        P: Into<Position>,
        M: crate::metadata::SiteMetadata,
    {
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
    ) -> Result<crate::MutationId, crate::TskitError>
    where
        S: Into<crate::SiteId>,
        N: Into<crate::NodeId>,
        P: Into<crate::MutationId>,
        T: Into<crate::Time>,
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
    ) -> Result<crate::MutationId, crate::TskitError>
    where
        S: Into<crate::SiteId>,
        N: Into<crate::NodeId>,
        P: Into<crate::MutationId>,
        T: Into<crate::Time>,
        M: crate::metadata::MutationMetadata,
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
    pub fn add_population(&mut self) -> Result<crate::PopulationId, crate::TskitError> {
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
    pub fn add_population_with_metadata<M>(
        &mut self,
        metadata: &M,
    ) -> Result<crate::PopulationId, crate::TskitError>
    where
        M: crate::metadata::PopulationMetadata,
    {
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
        // The output is UNINITIALIZED tables,
        // else we leak memory
        let mut inner = uninit_table_collection();

        let rv = unsafe { ll_bindings::tsk_table_collection_copy(self.as_ptr(), &mut *inner, 0) };

        // SAFETY: we just initialized it.
        // The C API doesn't free NULL pointers.
        handle_tsk_return_value!(rv, unsafe { Self::new_from_mbox(inner)? })
    }

    /// Return a [`crate::TreeSequence`] based on the tables.
    /// This function will raise errors if tables are not sorted,
    /// not indexed, or invalid in any way.
    pub fn tree_sequence(
        self,
        flags: TreeSequenceFlags,
    ) -> Result<crate::TreeSequence, TskitError> {
        crate::TreeSequence::new(self, flags)
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
    ) -> Result<Option<&[NodeId]>, TskitError> {
        if idmap {
            self.idmap.resize(
                usize::try_from(self.views.nodes().num_rows())?,
                NodeId::NULL,
            );
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
    pub fn check_integrity(&self, flags: TableIntegrityCheckFlags) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_check_integrity(self.as_ptr(), flags.bits())
        };
        handle_tsk_return_value!(rv)
    }

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
    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        self.provenances_mut().add_row(record)
    }

    /// Set the edge table from an [`OwningEdgeTable`](`crate::OwningEdgeTable`)
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
    /// let mut edges = tskit::OwningEdgeTable::default();
    /// edges.add_row(0., 1., 0, 12).unwrap();
    /// tables.set_edges(&edges).unwrap();
    /// assert_eq!(tables.edges().num_rows(), 1);
    /// assert_eq!(tables.edges().child(0).unwrap(), 12);
    /// # edges.clear().unwrap();
    /// # assert_eq!(edges.num_rows(), 0);
    /// ```
    pub fn set_edges(&mut self, edges: &crate::OwningEdgeTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_edge_table_set_columns(
                &mut self.inner.edges,
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

    /// Set the node table from an [`OwningNodeTable`](`crate::OwningNodeTable`)
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
    /// let mut nodes = tskit::OwningNodeTable::default();
    /// nodes.add_row(0, 10.0, -1, -1).unwrap();
    /// tables.set_nodes(&nodes).unwrap();
    /// assert_eq!(tables.nodes().num_rows(), 1);
    /// assert_eq!(tables.nodes().time(0).unwrap(), 10.0);
    /// # nodes.clear().unwrap();
    /// # assert_eq!(nodes.num_rows(), 0);
    /// ```
    pub fn set_nodes(&mut self, nodes: &crate::OwningNodeTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_node_table_set_columns(
                &mut self.inner.nodes,
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

    /// Set the site table from an [`OwningSiteTable`](`crate::OwningSiteTable`)
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
    /// let mut sites = tskit::OwningSiteTable::default();
    /// sites.add_row(11.0, None).unwrap();
    /// tables.set_sites(&sites).unwrap();
    /// assert_eq!(tables.sites().num_rows(), 1);
    /// assert_eq!(tables.sites().position(0).unwrap(), 11.0);
    /// # sites.clear().unwrap();
    /// # assert_eq!(sites.num_rows(), 0);
    /// ```
    pub fn set_sites(&mut self, sites: &crate::OwningSiteTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_site_table_set_columns(
                &mut self.inner.sites,
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

    /// Set the mutation table from an [`OwningMutationTable`](`crate::OwningSiteTable`)
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
    /// let mut mutations = tskit::OwningMutationTable::default();
    /// mutations.add_row(14, 12, -1, 11.3, None).unwrap();
    /// tables.set_mutations(&mutations).unwrap();
    /// assert_eq!(tables.mutations().num_rows(), 1);
    /// assert_eq!(tables.mutations().site(0).unwrap(), 14);
    /// # mutations.clear().unwrap();
    /// # assert_eq!(mutations.num_rows(), 0);
    /// ```
    pub fn set_mutations(&mut self, mutations: &crate::OwningMutationTable) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_mutation_table_set_columns(
                &mut self.inner.mutations,
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

    /// Set the individual table from an [`OwningIndividualTable`](`crate::OwningSiteTable`)
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
    /// let mut individuals = tskit::OwningIndividualTable::default();
    /// individuals.add_row(0, [0.1, 10.0], None).unwrap();
    /// tables.set_individuals(&individuals).unwrap();
    /// assert_eq!(tables.individuals().num_rows(), 1);
    /// let expected = vec![tskit::Location::from(0.1), tskit::Location::from(10.0)];
    /// assert_eq!(tables.individuals().location(0), Some(expected.as_slice()));
    /// # individuals.clear().unwrap();
    /// # assert_eq!(individuals.num_rows(), 0);
    /// ```
    pub fn set_individuals(
        &mut self,
        individuals: &crate::OwningIndividualTable,
    ) -> TskReturnValue {
        // SAFETY: neither self nor nodes are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_individual_table_set_columns(
                &mut self.inner.individuals,
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

    /// Set the migration table from an [`OwningMigrationTable`](`crate::OwningSiteTable`)
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
    /// let mut migrations = tskit::OwningMigrationTable::default();
    /// migrations.add_row((0.25, 0.37), 1, (0, 1), 111.0).unwrap();
    /// tables.set_migrations(&migrations).unwrap();
    /// assert_eq!(tables.migrations().num_rows(), 1);
    /// assert_eq!(tables.migrations().time(0).unwrap(), 111.0);
    /// # migrations.clear().unwrap();
    /// # assert_eq!(migrations.num_rows(), 0);
    /// ```
    pub fn set_migrations(&mut self, migrations: &crate::OwningMigrationTable) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_migration_table_set_columns(
                &mut self.inner.migrations,
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

    /// Set the population table from an [`OwningPopulationTable`](`crate::OwningSiteTable`)
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
    /// let mut populations = tskit::OwningPopulationTable::default();
    /// populations.add_row().unwrap();
    /// tables.set_populations(&populations).unwrap();
    /// assert_eq!(tables.populations().num_rows(), 1);
    /// # populations.clear().unwrap();
    /// # assert_eq!(populations.num_rows(), 0);
    /// ```
    pub fn set_populations(
        &mut self,
        populations: &crate::OwningPopulationTable,
    ) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_population_table_set_columns(
                &mut self.inner.populations,
                (*populations.as_ptr()).num_rows,
                (*populations.as_ptr()).metadata,
                (*populations.as_ptr()).metadata_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Set the provenance table from an
    /// [`OwningProvenanceTable`](`crate::provenance::OwningProvenanceTable`)
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
    /// let mut provenances = tskit::provenance::OwningProvenanceTable::default();
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
        provenances: &crate::provenance::OwningProvenanceTable,
    ) -> TskReturnValue {
        // SAFETY: neither self nor edges are possible
        // to create with null pointers.
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_set_columns(
                &mut self.inner.provenances,
                (*provenances.as_ptr()).num_rows,
                (*provenances.as_ptr()).timestamp,
                (*provenances.as_ptr()).timestamp_offset,
                (*provenances.as_ptr()).record,
                (*provenances.as_ptr()).record_offset,
            )
        };
        handle_tsk_return_value!(rv)
    }

    delegate! {
        to self.views {
            /// Get mutable reference to the [``NodeTable``](crate::NodeTable).
            pub fn nodes_mut(&mut self) -> &mut crate::NodeTable;

            /// Get mutable reference to the [``EdgeTable``](crate::EdgeTable).
            pub fn edges_mut(&mut self) -> &mut crate::EdgeTable;

            /// Get mutable reference to the [``SiteTable``](crate::SiteTable).
            pub fn sites_mut(&mut self) -> &mut crate::SiteTable;

            /// Get mutable reference to the [``MutationTable``](crate::MutationTable).
            pub fn mutations_mut(&mut self) -> &mut crate::MutationTable;

            /// Get mutable reference to the [``IndividualTable``](crate::IndividualTable).
            pub fn individuals_mut(&mut self) -> &mut crate::IndividualTable;

            /// Get mutable reference to the [``PopulationTable``](crate::PopulationTable).
            pub fn populations_mut(&mut self) -> &mut crate::PopulationTable;

            /// Get mutable reference to the [``MigrationTable``](crate::PopulationTable).
            pub fn migrations_mut(&mut self) -> &mut crate::MigrationTable;

            #[cfg(feature="provenance")]
            #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
            /// Get mutable reference to the [``ProvenanceTable``](crate::ProvenanceTable).
            pub fn provenances_mut(&mut self) -> &mut crate::provenance::ProvenanceTable;
        }
    }

    delegate_table_view_api!();

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_table_collection_t {
        &*self.inner
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_table_collection_t {
        &mut *self.inner
    }
}

#[test]
fn test_mutable_node_references() {
    let mut tables = crate::TableCollection::new(10.0).unwrap();
    let _nodes: &mut crate::NodeTable = tables.nodes_mut();
    let _edges: &mut crate::EdgeTable = tables.edges_mut();
    let _sites: &mut crate::SiteTable = tables.sites_mut();
    let _mutations: &mut crate::MutationTable = tables.mutations_mut();
    let _individuals: &mut crate::IndividualTable = tables.individuals_mut();
    let _populations: &mut crate::PopulationTable = tables.populations_mut();
    let _migrations: &mut crate::MigrationTable = tables.migrations_mut();
    #[cfg(feature = "provenance")]
    let _provenances: &mut crate::provenance::ProvenanceTable = tables.provenances_mut();
}
