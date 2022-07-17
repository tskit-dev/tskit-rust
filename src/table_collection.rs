use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::WrapTskitType;
use crate::metadata::*;
use crate::traits::IndividualLocation;
use crate::traits::IndividualParents;
use crate::types::Bookmark;
use crate::EdgeTable;
use crate::IndividualFlags;
use crate::IndividualTable;
use crate::IndividualTableSortOptions;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeFlags;
use crate::NodeTable;
use crate::PopulationTable;
use crate::Position;
use crate::SimplificationOptions;
use crate::SiteTable;
use crate::TableAccess;
use crate::TableClearOptions;
use crate::TableEqualityOptions;
use crate::TableIntegrityCheckFlags;
use crate::TableOutputOptions;
use crate::TableSortOptions;
use crate::Time;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::TskitTypeAccess;
use crate::{tsk_id_t, tsk_size_t};
use crate::{EdgeId, IndividualId, MigrationId, MutationId, NodeId, PopulationId, SiteId};
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
/// use tskit::TableAccess;
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
    pub(crate) inner: MBox<ll_bindings::tsk_table_collection_t>,
}

build_tskit_type!(
    TableCollection,
    ll_bindings::tsk_table_collection_t,
    tsk_table_collection_free
);

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
                got: sequence_length.0.to_string(),
                expected: "sequence_length >= 0.0".to_string(),
            });
        }
        let mut tables = Self::wrap();
        let rv = unsafe { ll_bindings::tsk_table_collection_init(tables.as_mut_ptr(), 0) };
        if rv < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: rv });
        }
        unsafe {
            (*tables.as_mut_ptr()).sequence_length = sequence_length.0;
        }
        Ok(tables)
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
    /// # std::fs::remove_file("trees.file").unwrap();
    /// ```
    ///
    /// ```
    /// # let empty_tables = tskit::TableCollection::new(100.).unwrap();
    /// # empty_tables.dump("trees.file", tskit::TableOutputOptions::default()).unwrap();
    /// let filename = String::from("trees.file");
    /// // Move filename
    /// let tables = tskit::TableCollection::new_from_file(filename).unwrap();
    /// # std::fs::remove_file("trees.file").unwrap();
    /// ```
    ///
    /// ```
    /// # let empty_tables = tskit::TableCollection::new(100.).unwrap();
    /// # empty_tables.dump("trees.file", tskit::TableOutputOptions::default()).unwrap();
    /// let filename = String::from("trees.file");
    /// // Pass filename by reference
    /// let tables = tskit::TableCollection::new_from_file(&filename).unwrap();
    /// # std::fs::remove_file("trees.file").unwrap();
    /// ```
    ///
    /// Boxed `String`s are an unlikely use case, but can be made to work:
    ///
    /// ```
    /// # let empty_tables = tskit::TableCollection::new(100.).unwrap();
    /// # empty_tables.dump("trees.file", tskit::TableOutputOptions::default()).unwrap();
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
            Ok(t) => (t),
            Err(e) => return Err(e),
        };

        let c_str = std::ffi::CString::new(filename.as_ref()).unwrap();
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
    pub fn add_edge<L: Into<Position>, R: Into<Position>, P: Into<NodeId>, C: Into<NodeId>>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
    ) -> Result<EdgeId, TskitError> {
        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*self.as_mut_ptr()).edges,
                left.into().0,
                right.into().0,
                parent.into().0,
                child.into().0,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv, EdgeId::from(rv))
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
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*self.as_mut_ptr()).edges,
                left.into().0,
                right.into().0,
                parent.into().0,
                child.into().0,
                md.as_ptr(),
                md.len().into(),
            )
        };

        handle_tsk_return_value!(rv, EdgeId::from(rv))
    }

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
    ///
    pub fn add_individual<F: Into<IndividualFlags>, L: IndividualLocation, I: IndividualParents>(
        &mut self,
        flags: F,
        location: L,
        parents: I,
    ) -> Result<IndividualId, TskitError> {
        let rv = unsafe {
            ll_bindings::tsk_individual_table_add_row(
                &mut (*self.as_mut_ptr()).individuals,
                flags.into().bits(),
                location.get_slice().as_ptr().cast::<f64>(),
                location.get_slice().len() as tsk_size_t,
                parents.get_slice().as_ptr().cast::<tsk_id_t>(),
                parents.get_slice().len() as tsk_size_t,
                std::ptr::null(),
                0,
            )
        };
        handle_tsk_return_value!(rv, IndividualId::from(rv))
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
    pub fn add_individual_with_metadata<
        F: Into<IndividualFlags>,
        L: IndividualLocation,
        I: IndividualParents,
        M: IndividualMetadata,
    >(
        &mut self,
        flags: F,
        location: L,
        parents: I,
        metadata: &M,
    ) -> Result<IndividualId, TskitError> {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_individual_table_add_row(
                &mut (*self.as_mut_ptr()).individuals,
                flags.into().bits(),
                location.get_slice().as_ptr().cast::<f64>(),
                location.get_slice().len() as tsk_size_t,
                parents.get_slice().as_ptr().cast::<tsk_id_t>(),
                parents.get_slice().len() as tsk_size_t,
                md.as_ptr(),
                md.len().into(),
            )
        };
        handle_tsk_return_value!(rv, IndividualId::from(rv))
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
    pub fn add_migration<
        L: Into<Position>,
        R: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        T: Into<Time>,
    >(
        &mut self,
        span: (L, R),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
    ) -> Result<MigrationId, TskitError> {
        let rv = unsafe {
            ll_bindings::tsk_migration_table_add_row(
                &mut (*self.as_mut_ptr()).migrations,
                span.0.into().0,
                span.1.into().0,
                node.into().0,
                source_dest.0.into().0,
                source_dest.1.into().0,
                time.into().0,
                std::ptr::null(),
                0,
            )
        };
        handle_tsk_return_value!(rv, MigrationId(rv))
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
    pub fn add_migration_with_metadata<
        L: Into<Position>,
        R: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        MD: MigrationMetadata,
        T: Into<Time>,
    >(
        &mut self,
        span: (L, R),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
        metadata: &MD,
    ) -> Result<MigrationId, TskitError> {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_migration_table_add_row(
                &mut (*self.as_mut_ptr()).migrations,
                span.0.into().0,
                span.1.into().0,
                node.into().0,
                source_dest.0.into().0,
                source_dest.1.into().0,
                time.into().0,
                md.as_ptr(),
                md.len().into(),
            )
        };
        handle_tsk_return_value!(rv, MigrationId(rv))
    }

    /// Add a row to the node table
    pub fn add_node<
        F: Into<NodeFlags>,
        T: Into<Time>,
        POP: Into<PopulationId>,
        I: Into<IndividualId>,
    >(
        &mut self,
        flags: F,
        time: T,
        population: POP,
        individual: I,
    ) -> Result<NodeId, TskitError> {
        let rv = unsafe {
            ll_bindings::tsk_node_table_add_row(
                &mut (*self.as_mut_ptr()).nodes,
                flags.into().bits(),
                time.into().0,
                population.into().0,
                individual.into().0,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv, rv.into())
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
    pub fn add_node_with_metadata<
        F: Into<NodeFlags>,
        T: Into<Time>,
        POP: Into<PopulationId>,
        I: Into<IndividualId>,
        M: NodeMetadata,
    >(
        &mut self,
        flags: F,
        time: T,
        population: POP,
        individual: I,
        metadata: &M,
    ) -> Result<NodeId, TskitError> {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_node_table_add_row(
                &mut (*self.as_mut_ptr()).nodes,
                flags.into().bits(),
                time.into().0,
                population.into().0,
                individual.into().0,
                md.as_ptr(),
                md.len().into(),
            )
        };

        handle_tsk_return_value!(rv, rv.into())
    }

    /// Add a row to the site table
    pub fn add_site<P: Into<Position>>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
    ) -> Result<SiteId, TskitError> {
        let astate = process_state_input!(ancestral_state);

        let rv = unsafe {
            ll_bindings::tsk_site_table_add_row(
                &mut (*self.as_mut_ptr()).sites,
                position.into().0,
                astate.0,
                astate.1,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv, SiteId::from(rv))
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
        let astate = process_state_input!(ancestral_state);
        let md = EncodedMetadata::new(metadata)?;

        let rv = unsafe {
            ll_bindings::tsk_site_table_add_row(
                &mut (*self.as_mut_ptr()).sites,
                position.into().0,
                astate.0,
                astate.1,
                md.as_ptr(),
                md.len().into(),
            )
        };

        handle_tsk_return_value!(rv, SiteId::from(rv))
    }

    /// Add a row to the mutation table.
    pub fn add_mutation<S: Into<SiteId>, N: Into<NodeId>, M: Into<MutationId>, T: Into<Time>>(
        &mut self,
        site: S,
        node: N,
        parent: M,
        time: T,
        derived_state: Option<&[u8]>,
    ) -> Result<MutationId, TskitError> {
        let dstate = process_state_input!(derived_state);
        let rv = unsafe {
            ll_bindings::tsk_mutation_table_add_row(
                &mut (*self.as_mut_ptr()).mutations,
                site.into().0,
                node.into().0,
                parent.into().0,
                time.into().0,
                dstate.0,
                dstate.1,
                std::ptr::null(),
                0,
            )
        };
        handle_tsk_return_value!(rv, MutationId::from(rv))
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
    pub fn add_mutation_with_metadata<
        S: Into<SiteId>,
        N: Into<NodeId>,
        M: Into<MutationId>,
        MD: MutationMetadata,
        T: Into<Time>,
    >(
        &mut self,
        site: S,
        node: N,
        parent: M,
        time: T,
        derived_state: Option<&[u8]>,
        metadata: &MD,
    ) -> Result<MutationId, TskitError> {
        let dstate = process_state_input!(derived_state);
        let md = EncodedMetadata::new(metadata)?;

        let rv = unsafe {
            ll_bindings::tsk_mutation_table_add_row(
                &mut (*self.as_mut_ptr()).mutations,
                site.into().0,
                node.into().0,
                parent.into().0,
                time.into().0,
                dstate.0,
                dstate.1,
                md.as_ptr(),
                md.len().into(),
            )
        };
        handle_tsk_return_value!(rv, MutationId::from(rv))
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
        let rv = unsafe {
            ll_bindings::tsk_population_table_add_row(
                &mut (*self.as_mut_ptr()).populations,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv, PopulationId::from(rv))
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
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_population_table_add_row(
                &mut (*self.as_mut_ptr()).populations,
                md.as_ptr(),
                md.len().into(),
            )
        };

        handle_tsk_return_value!(rv, PopulationId::from(rv))
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
        let mut copy = TableCollection::new(1.)?;

        let rv =
            unsafe { ll_bindings::tsk_table_collection_copy(self.as_ptr(), copy.as_mut_ptr(), 0) };

        handle_tsk_return_value!(rv, copy)
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
    ) -> Result<Option<Vec<NodeId>>, TskitError> {
        let mut output_node_map: Vec<NodeId> = vec![];
        if idmap {
            output_node_map.resize(usize::try_from(self.nodes().num_rows())?, NodeId::NULL);
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
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_add_row(
                &mut (*self.inner).provenances,
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, crate::ProvenanceId::from(rv))
    }
}

impl TableAccess for TableCollection {
    fn edges(&self) -> EdgeTable {
        EdgeTable::new_from_table(&(*self.inner).edges)
    }

    fn individuals(&self) -> IndividualTable {
        IndividualTable::new_from_table(&(*self.inner).individuals)
    }

    fn migrations(&self) -> MigrationTable {
        MigrationTable::new_from_table(&(*self.inner).migrations)
    }

    fn nodes(&self) -> NodeTable {
        NodeTable::new_from_table(&(*self.inner).nodes)
    }

    fn sites(&self) -> SiteTable {
        SiteTable::new_from_table(&(*self.inner).sites)
    }

    fn mutations(&self) -> MutationTable {
        MutationTable::new_from_table(&(*self.inner).mutations)
    }

    fn populations(&self) -> PopulationTable {
        PopulationTable::new_from_table(&unsafe{*self.as_ptr()}.populations)
    }

    #[cfg(any(feature = "provenance", doc))]
    fn provenances(&self) -> crate::provenance::ProvenanceTable {
        crate::provenance::ProvenanceTable::new_from_table(&(*self.inner).provenances)
    }
}

impl crate::traits::NodeListGenerator for TableCollection {}

#[cfg(test)]
mod test {
    use super::*;

    fn make_small_table_collection() -> TableCollection {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
            .unwrap();
        tables
            .add_node(0, 0.0, PopulationId::NULL, IndividualId::NULL)
            .unwrap();
        tables
            .add_node(0, 0.0, PopulationId::NULL, IndividualId::NULL)
            .unwrap();
        tables.add_edge(0., 1000., 0, 1).unwrap();
        tables.add_edge(0., 1000., 0, 2).unwrap();
        tables.build_index().unwrap();
        tables
    }

    #[test]
    fn test_sequence_length() {
        let tables = TableCollection::new(1000.).unwrap();
        assert!(close_enough(tables.sequence_length(), 1000.));
    }

    #[test]
    #[should_panic]
    fn test_zero_sequence_length() {
        let _ = TableCollection::new(0.).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_negative_sequence_length() {
        let _ = TableCollection::new(-1.).unwrap();
    }

    #[test]
    fn test_add_edges() {
        let mut tables = TableCollection::new(1000.).unwrap();
        for i in 0..5 {
            let _ = tables.add_edge(0., 1000., i, 2 * i).unwrap();
        }
        let edges = tables.edges();
        for i in 0..5 {
            assert_eq!(edges.parent(i).unwrap(), i);
            assert_eq!(edges.child(i).unwrap(), 2 * i);
        }
    }

    #[test]
    fn test_mutable_node_access() {
        let tables = TableCollection::new(1000.).unwrap();
        let mut nodes = tables.nodes();
        let f = nodes.flags_array_mut();
        for i in f {
            *i = NodeFlags::from(11);
        }

        for t in nodes.time_array_mut() {
            *t = Time::from(-33.0);
        }

        for i in tables.nodes_iter() {
            assert_eq!(i.flags.bits(), 11);
            assert_eq!(f64::from(i.time) as i64, -33);
        }
    }

    #[test]
    fn test_node_iteration() {
        let tables = make_small_table_collection();
        for (i, row) in tables.nodes().iter().enumerate() {
            assert!(close_enough(
                tables.nodes().time(i as tsk_id_t).unwrap(),
                row.time
            ));
            assert_eq!(tables.nodes().flags(i as tsk_id_t).unwrap(), row.flags);
            assert_eq!(
                tables.nodes().population(i as tsk_id_t).unwrap(),
                row.population
            );
            assert_eq!(
                tables.nodes().individual(i as tsk_id_t).unwrap(),
                row.individual
            );
            assert!(row.metadata.is_none());
        }

        for row in tables.nodes_iter() {
            assert!(close_enough(tables.nodes().time(row.id).unwrap(), row.time));
            assert_eq!(tables.nodes().flags(row.id).unwrap(), row.flags);
            assert_eq!(tables.nodes().population(row.id).unwrap(), row.population);
            assert_eq!(tables.nodes().individual(row.id).unwrap(), row.individual);
            assert!(row.metadata.is_none());
        }
    }

    #[test]
    fn test_edge_iteration() {
        let tables = make_small_table_collection();
        for (i, row) in tables.edges().iter().enumerate() {
            assert!(close_enough(
                tables.edges().left(i as tsk_id_t).unwrap(),
                row.left
            ));
            assert!(close_enough(
                tables.edges().right(i as tsk_id_t).unwrap(),
                row.right
            ));
            assert_eq!(tables.edges().parent(i as tsk_id_t).unwrap(), row.parent);
            assert_eq!(tables.edges().child(i as tsk_id_t).unwrap(), row.child);
            assert!(row.metadata.is_none());
        }
        for row in tables.edges_iter() {
            assert!(close_enough(tables.edges().left(row.id).unwrap(), row.left));
            assert!(close_enough(
                tables.edges().right(row.id).unwrap(),
                row.right
            ));
            assert_eq!(tables.edges().parent(row.id).unwrap(), row.parent);
            assert_eq!(tables.edges().child(row.id).unwrap(), row.child);
            assert!(row.metadata.is_none());
        }
    }

    #[test]
    fn test_edge_index_access() {
        let tables = make_small_table_collection();
        assert!(tables.is_indexed());
        assert_eq!(
            tables.edge_insertion_order().unwrap().len(),
            tables.edges().num_rows().try_into().unwrap()
        );
        assert_eq!(
            tables.edge_removal_order().unwrap().len(),
            tables.edges().num_rows().try_into().unwrap()
        );

        for i in tables.edge_insertion_order().unwrap() {
            assert!(*i >= 0);
            assert!(*i < tables.edges().num_rows());
        }

        for i in tables.edge_removal_order().unwrap() {
            assert!(*i >= 0);
            assert!(*i < tables.edges().num_rows());
        }

        // The "transparent" casts are such black magic that we
        // should probably test against what C thinks is going on :)
        let input = unsafe {
            std::slice::from_raw_parts(
                (*tables.inner).indexes.edge_insertion_order,
                (*tables.inner).indexes.num_edges as usize,
            )
        };

        assert!(!input.is_empty());

        let tables_input = tables.edge_insertion_order().unwrap();

        assert_eq!(input.len(), tables_input.len());

        for i in 0..input.len() {
            assert_eq!(EdgeId::from(input[i]), tables_input[i]);
        }

        let output = unsafe {
            std::slice::from_raw_parts(
                (*tables.inner).indexes.edge_removal_order,
                (*tables.inner).indexes.num_edges as usize,
            )
        };
        assert!(!output.is_empty());

        let tables_output = tables.edge_removal_order().unwrap();

        assert_eq!(output.len(), tables_output.len());

        for i in 0..output.len() {
            assert_eq!(EdgeId::from(output[i]), tables_output[i]);
        }
    }

    #[test]
    fn test_add_site() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_site(0.3, Some(b"Eggnog")).unwrap();
        tables.add_site(0.5, None).unwrap(); // No ancestral_state specified!!!
        let longer_metadata = "Hot Toddy";
        tables
            .add_site(0.9, Some(longer_metadata.as_bytes()))
            .unwrap();

        let sites = tables.sites();
        assert!(close_enough(sites.position(0).unwrap(), 0.3));
        assert!(close_enough(sites.position(1).unwrap(), 0.5));
        assert!(close_enough(sites.position(2).unwrap(), 0.9));

        match sites.ancestral_state(0).unwrap() {
            Some(astate) => assert_eq!(astate, b"Eggnog"),
            None => panic!(),
        };

        if sites.ancestral_state(1).unwrap().is_some() {
            panic!()
        }

        match sites.ancestral_state(2).unwrap() {
            Some(astate) => assert_eq!(astate, longer_metadata.as_bytes()),
            None => panic!(),
        };

        // NOTE: this is a useful test as not all rows have ancestral_state
        let mut no_anc_state = 0;
        for (i, row) in sites.iter().enumerate() {
            assert!(close_enough(
                sites.position(i as tsk_id_t).unwrap(),
                row.position
            ));
            if row.ancestral_state.is_some() {
                if i == 0 {
                    assert_eq!(row.ancestral_state.unwrap(), b"Eggnog");
                } else if i == 2 {
                    assert_eq!(row.ancestral_state.unwrap(), longer_metadata.as_bytes());
                }
            } else {
                no_anc_state += 1;
            }
        }
        assert_eq!(no_anc_state, 1);
        no_anc_state = 0;
        for row in tables.sites_iter() {
            assert!(close_enough(sites.position(row.id).unwrap(), row.position));
            if row.ancestral_state.is_some() {
                if row.id == 0 {
                    assert_eq!(row.ancestral_state.unwrap(), b"Eggnog");
                } else if row.id == 2 {
                    assert_eq!(row.ancestral_state.unwrap(), longer_metadata.as_bytes());
                }
            } else {
                no_anc_state += 1;
            }
        }
        assert_eq!(no_anc_state, 1);
    }

    fn close_enough<L: Into<f64>, R: Into<f64>>(a: L, b: R) -> bool {
        (a.into() - b.into()).abs() < f64::EPSILON
    }

    #[test]
    fn test_add_mutation() {
        let mut tables = TableCollection::new(1000.).unwrap();

        tables
            .add_mutation(0, 0, MutationId::NULL, 1.123, Some(b"pajamas"))
            .unwrap();
        tables
            .add_mutation(1, 1, MutationId::NULL, 2.123, None)
            .unwrap();
        tables
            .add_mutation(2, 2, MutationId::NULL, 3.123, Some(b"more pajamas"))
            .unwrap();
        let mutations = tables.mutations();
        assert!(close_enough(mutations.time(0).unwrap(), 1.123));
        assert!(close_enough(mutations.time(1).unwrap(), 2.123));
        assert!(close_enough(mutations.time(2).unwrap(), 3.123));
        assert_eq!(mutations.node(0).unwrap(), 0);
        assert_eq!(mutations.node(1).unwrap(), 1);
        assert_eq!(mutations.node(2).unwrap(), 2);
        assert_eq!(mutations.parent(0).unwrap(), MutationId::NULL);
        assert_eq!(mutations.parent(1).unwrap(), MutationId::NULL);
        assert_eq!(mutations.parent(2).unwrap(), MutationId::NULL);
        assert_eq!(mutations.derived_state(0).unwrap().unwrap(), b"pajamas");

        if mutations.derived_state(1).unwrap().is_some() {
            panic!()
        }

        assert_eq!(
            mutations.derived_state(2).unwrap().unwrap(),
            b"more pajamas"
        );

        let mut nmuts = 0;
        for (i, row) in tables.mutations().iter().enumerate() {
            assert_eq!(row.site, tables.mutations().site(i as tsk_id_t).unwrap());
            assert_eq!(row.node, tables.mutations().node(i as tsk_id_t).unwrap());
            assert_eq!(
                row.parent,
                tables.mutations().parent(i as tsk_id_t).unwrap()
            );
            assert!(close_enough(
                row.time,
                tables.mutations().time(i as tsk_id_t).unwrap()
            ));
            assert!(row.metadata.is_none());
            nmuts += 1;
        }
        assert_eq!(nmuts, tables.mutations().num_rows());
        assert_eq!(nmuts, 3);

        nmuts = 0;
        for row in tables.mutations_iter() {
            assert_eq!(row.site, tables.mutations().site(row.id).unwrap());
            assert_eq!(row.node, tables.mutations().node(row.id).unwrap());
            assert_eq!(row.parent, tables.mutations().parent(row.id).unwrap());
            assert!(close_enough(
                row.time,
                tables.mutations().time(row.id).unwrap()
            ));
            assert!(row.metadata.is_none());
            nmuts += 1;
        }
        assert_eq!(nmuts, tables.mutations().num_rows());
        assert_eq!(nmuts, 3);
        for row in tables.mutations().iter() {
            assert!(row.metadata.is_none());
        }

        nmuts = 0;
        for _ in tables.mutations().iter().skip(1) {
            nmuts += 1;
        }
        assert_eq!(
            crate::SizeType::try_from(nmuts + 1).unwrap(),
            tables.mutations().num_rows()
        );
    }

    struct F {
        x: i32,
        y: u32,
    }

    impl MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            let mut rv = vec![];
            rv.extend(self.x.to_le_bytes().iter().copied());
            rv.extend(self.y.to_le_bytes().iter().copied());
            Ok(rv)
        }
        fn decode(md: &[u8]) -> Result<Self, MetadataError> {
            let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
            let (y_int_bytes, _) = rest.split_at(std::mem::size_of::<u32>());
            Ok(Self {
                x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
                y: u32::from_le_bytes(y_int_bytes.try_into().unwrap()),
            })
        }
    }

    impl MutationMetadata for F {}

    #[test]
    fn test_add_mutation_with_metadata() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_mutation_with_metadata(0, 0, MutationId::NULL, 1.123, None, &F { x: -3, y: 666 })
            .unwrap();
        // The double unwrap is to first check for error
        // and then to process the Option.
        let md = tables.mutations().metadata::<F>(0.into()).unwrap().unwrap();
        assert_eq!(md.x, -3);
        assert_eq!(md.y, 666);

        for row in tables.mutations().iter() {
            assert!(row.metadata.is_some());
            let md = F::decode(&row.metadata.unwrap()).unwrap();
            assert_eq!(md.x, -3);
            assert_eq!(md.y, 666);
        }
    }

    #[test]
    fn test_add_mutation_with_metadata_for_some_columns() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_mutation_with_metadata(0, 0, MutationId::NULL, 1.123, None, &F { x: -3, y: 666 })
            .unwrap();

        tables
            .add_mutation(1, 2, MutationId::NULL, 2.0, None)
            .unwrap();

        let mut num_with_metadata = 0;
        let mut num_without_metadata = 0;
        for i in 0..usize::try_from(tables.mutations().num_rows()).unwrap() {
            match tables
                .mutations()
                .metadata::<F>((i as tsk_id_t).into())
                .unwrap()
            {
                Some(x) => {
                    num_with_metadata += 1;
                    assert_eq!(x.x, -3);
                    assert_eq!(x.y, 666);
                }
                None => {
                    num_without_metadata += 1;
                }
            }
        }
        assert_eq!(num_with_metadata, 1);
        assert_eq!(num_without_metadata, 1);
    }

    #[test]
    fn test_add_population() {
        let mut tables = TableCollection::new(1000.).unwrap();
        let pop_id = tables.add_population().unwrap();
        assert_eq!(pop_id, 0);
        assert_eq!(tables.populations().num_rows(), 1);

        tables
            .add_node(crate::TSK_NODE_IS_SAMPLE, 0.0, pop_id, IndividualId::NULL)
            .unwrap();

        match tables.nodes().row(NodeId::from(0)) {
            Ok(x) => match x.population {
                PopulationId(0) => (),
                _ => panic!("expected PopulationId(0)"),
            },
            Err(_) => panic!("expected Ok(_)"),
        };
    }

    #[test]
    fn test_dump_tables() {
        let treefile = "trees.trees";
        let mut tables = TableCollection::new(1000.).unwrap();
        let pop_id = tables.add_population().unwrap();
        tables
            .add_node(crate::TSK_NODE_IS_SAMPLE, 0.0, pop_id, IndividualId::NULL)
            .unwrap();
        tables
            .add_node(crate::TSK_NODE_IS_SAMPLE, 1.0, pop_id, IndividualId::NULL)
            .unwrap();
        tables.add_edge(0., tables.sequence_length(), 1, 0).unwrap();
        tables
            .dump(treefile, TableOutputOptions::default())
            .unwrap();

        let tables2 = TableCollection::new_from_file(treefile).unwrap();
        assert!(tables.equals(&tables2, TableEqualityOptions::default()));

        std::fs::remove_file(&treefile).unwrap();
    }

    #[test]
    fn test_clear() {
        let mut tables = TableCollection::new(1000.).unwrap();
        for i in 0..5 {
            let _ = tables.add_edge(0., 1000., i, 2 * i).unwrap();
        }
        assert_eq!(tables.edges().num_rows(), 5);
        tables.clear(TableClearOptions::default()).unwrap();
        assert_eq!(tables.edges().num_rows(), 0);
    }

    #[test]
    fn test_free() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.free().unwrap();
    }

    #[test]
    fn test_deepcopy() {
        let tables = make_small_table_collection();
        let dumps = tables.deepcopy().unwrap();
        assert!(tables.equals(&dumps, TableEqualityOptions::default()));
    }

    #[test]
    fn test_edge_table_row_equality() {
        let tables = make_small_table_collection();
        for (i, row) in tables.edges_iter().enumerate() {
            assert!(row.id == i as tsk_id_t);
            assert!(row == tables.edges().row(i as tsk_id_t).unwrap());
            assert!(!(row != tables.edges().row(i as tsk_id_t).unwrap()));
            if i > 0 {
                assert!(row != tables.edges().row(i as tsk_id_t - 1).unwrap());
            }
        }
    }

    #[test]
    fn test_node_table_row_equality() {
        let tables = make_small_table_collection();
        for (i, row) in tables.nodes_iter().enumerate() {
            assert!(row.id.0 == i as tsk_id_t);
            assert!(row == tables.nodes().row(i as tsk_id_t).unwrap());
            assert!(!(row != tables.nodes().row(i as tsk_id_t).unwrap()));
        }
        assert!(tables.nodes().row(0).unwrap() != tables.nodes().row(1).unwrap());
        assert!(tables.nodes().row(1).unwrap() != tables.nodes().row(2).unwrap());
    }

    #[test]
    fn test_add_individual_many_ways() {
        {
            let mut tables = TableCollection::new(1.).unwrap();
            let location = vec![0., 1., 2.];
            let parents = [0, 1, 2, 3, 4];
            tables.add_individual(0, location, parents).unwrap();
        }
        {
            let mut tables = TableCollection::new(1.).unwrap();
            let location = vec![0., 1., 2.];
            let parents = [0, 1, 2, 3, 4];
            tables
                .add_individual(0, location.as_slice(), parents.as_slice())
                .unwrap();
        }
        {
            let mut tables = TableCollection::new(1.).unwrap();
            let location = [0., 1., 2.];
            let parents = vec![0, 1, 2, 3, 4];
            tables.add_individual(0, location, parents).unwrap();
        }
        {
            let mut tables = TableCollection::new(1.).unwrap();
            let location = [0., 1., 2.];
            let parents = vec![0, 1, 2, 3, 4];
            tables
                .add_individual(0, location.as_slice(), parents.as_slice())
                .unwrap();
        }
    }
}

#[cfg(test)]
mod test_bad_metadata {
    use super::*;
    use crate::test_fixtures::bad_metadata::*;

    #[test]
    fn test_bad_mutation_metadata_roundtrip() {
        let mut tables = TableCollection::new(1.).unwrap();
        let md = F { x: 1, y: 11 };
        tables
            .add_mutation_with_metadata(0, 0, MutationId::NULL, 0.0, None, &md)
            .unwrap();
        if tables.mutations().metadata::<Ff>(0.into()).is_ok() {
            panic!("expected an error!!");
        }
    }
}

// The tests that follow involve more detailed analysis
// of the strong ID types.

#[cfg(test)]
mod test_adding_node {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_node_without_metadata() {
        let mut tables = make_empty_table_collection(10.);

        match tables.add_node(0, 0.0, PopulationId::NULL, IndividualId::NULL) {
            Ok(NodeId(0)) => (),
            _ => panic!("Expected NodeId(0)"),
        };

        let row = tables.nodes().row(NodeId::from(0)).unwrap();

        assert_eq!(row.id, NodeId::from(0));
        assert_eq!(row.population, PopulationId::NULL);
        assert_eq!(row.individual, IndividualId::NULL);
        assert!(row.metadata.is_none());

        let row_id = tables
            .add_node(0, 0.0, PopulationId::from(2), IndividualId::NULL)
            .unwrap();

        assert_eq!(tables.nodes().population(row_id).unwrap(), PopulationId(2));
        assert_eq!(
            tables.nodes().individual(row_id).unwrap(),
            IndividualId::NULL,
        );
        assert!(tables
            .nodes()
            .metadata::<GenericMetadata>(row_id)
            .unwrap()
            .is_none());

        // We are playing a dangerous game here,
        // in that we do not have any populations.
        // Fortunately, we are range-checked everywhere.
        assert!(tables
            .populations()
            .row(tables.nodes().population(row_id).unwrap())
            .is_err());

        let row_id = tables
            .add_node(0, 0.0, PopulationId::NULL, IndividualId::from(17))
            .unwrap();

        assert_eq!(
            tables.nodes().population(row_id).unwrap(),
            PopulationId::NULL,
        );
        assert_eq!(tables.nodes().individual(row_id).unwrap(), IndividualId(17));

        assert!(tables
            .individuals()
            .row(tables.nodes().individual(row_id).unwrap())
            .is_err());
    }

    #[test]
    fn test_adding_node_with_metadata() {
        let mut tables = make_empty_table_collection(10.);
        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 12345 }];

        for (mi, m) in metadata.iter().enumerate() {
            let row_id = match tables.add_node_with_metadata(
                0,
                1.0,
                PopulationId::from(11),
                IndividualId::from(12),
                m,
            ) {
                Ok(NodeId(x)) => NodeId(x),
                Err(_) => panic!("unexpected Err"),
            };
            assert_eq!(
                tables.nodes().metadata::<GenericMetadata>(row_id).unwrap(),
                Some(metadata[mi])
            );
        }
    }
}

#[cfg(test)]
mod test_adding_individual {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_individual_without_metadata() {
        let mut tables = make_empty_table_collection(10.);
        match tables.add_individual(0, &[0., 0., 0.], &[IndividualId::NULL, IndividualId::NULL]) {
            Ok(IndividualId(0)) => (),
            _ => panic!("Expected NodeId(0)"),
        };

        let row = tables.individuals().row(IndividualId::from(0)).unwrap();
        assert_eq!(row.id, IndividualId::from(0));
        assert!(row.location.is_some());
        assert_eq!(row.location.unwrap().len(), 3);

        assert_eq!(
            row.parents,
            Some(vec![IndividualId::NULL, IndividualId::NULL,])
        );

        // Empty slices are a thing, causing None to be in the rows.

        let mut tables = crate::test_fixtures::make_empty_table_collection(10.);
        let row_id = tables
            .add_individual(0, &[] as &[f64], &[] as &[IndividualId])
            .unwrap();
        let row = tables.individuals().row(row_id).unwrap();
        assert_eq!(row.id, IndividualId::from(0));
        assert!(row.location.is_none());
        assert!(row.parents.is_none());
    }

    #[test]
    fn test_adding_individual_with_metadata() {
        let mut tables = crate::test_fixtures::make_empty_table_collection(10.);
        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 12345 }];

        for (mi, m) in metadata.iter().enumerate() {
            let row_id = match tables.add_individual_with_metadata(
                0,
                &[] as &[f64],
                &[] as &[IndividualId],
                m,
            ) {
                Ok(IndividualId(x)) => IndividualId(x),
                Err(_) => panic!("unexpected Err"),
            };
            assert_eq!(
                tables
                    .individuals()
                    .metadata::<GenericMetadata>(row_id)
                    .unwrap(),
                Some(metadata[mi])
            );
        }

        for (i, j) in tables.individuals().iter().enumerate() {
            assert!(
                tables
                    .individuals()
                    .row(IndividualId::from(i as tsk_id_t))
                    .unwrap()
                    == j
            );
        }

        for (i, j) in tables.individuals_iter().enumerate() {
            assert!(
                tables
                    .individuals()
                    .row(IndividualId::from(i as tsk_id_t))
                    .unwrap()
                    == j
            );
        }
    }
}

#[cfg(test)]
mod test_adding_edge {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_edge_without_metadata() {
        let mut tables = make_empty_table_collection(10.0);

        let edge_id = tables
            .add_edge(0., tables.sequence_length(), 0, 11)
            .unwrap();

        assert_eq!(edge_id, EdgeId(0));
        assert_eq!(tables.edges().parent(edge_id).unwrap(), NodeId(0));
        assert_eq!(tables.edges().child(edge_id).unwrap(), NodeId(11));
    }

    #[test]
    fn test_adding_edge_with_metadata() {
        let mut tables = make_empty_table_collection(10.0);
        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 12345 }];

        for (mi, m) in metadata.iter().enumerate() {
            let edge_id =
                match tables.add_edge_with_metadata(0., tables.sequence_length(), 0, 11, m) {
                    Ok(EdgeId(x)) => EdgeId(x),
                    Err(_) => panic!("unexpected Err"),
                };
            assert_eq!(
                tables.edges().metadata::<GenericMetadata>(edge_id).unwrap(),
                Some(metadata[mi])
            );
        }
    }
}

#[cfg(test)]
mod test_adding_mutation {
    use crate::metadata::MetadataRoundtrip;
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_mutation_without_metadata() {
        let mut tables = make_empty_table_collection(1.0);

        let mut_id = tables.add_mutation(0, 0, -1, 1.0, None).unwrap();

        assert_eq!(mut_id, MutationId(0));
        assert_eq!(mut_id, 0);

        let row_0 = tables.mutations().row(mut_id).unwrap();

        assert_eq!(row_0.id, 0);

        let mut_id_two = tables.add_mutation(0, 0, -1, 1.0, None).unwrap();

        assert!(mut_id_two > mut_id);
        assert_ne!(mut_id_two, mut_id);

        let row_1 = tables.mutations().row(mut_id_two).unwrap();

        assert!(row_0 != row_1);

        for row in [mut_id, mut_id_two] {
            if tables
                .mutations()
                .metadata::<GenericMetadata>(row)
                .unwrap()
                .is_some()
            {
                panic!("expected None");
            }
        }
    }

    #[test]
    fn test_adding_mutation_with_metadata() {
        let mut tables = make_empty_table_collection(1.0);
        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 12345 }];

        for (mi, m) in metadata.iter().enumerate() {
            let mut_id = match tables.add_mutation_with_metadata(0, 0, -1, 1.0, None, m) {
                Ok(MutationId(x)) => MutationId(x),
                Err(_) => panic!("unexpected Err"),
            };
            assert_eq!(
                tables
                    .mutations()
                    .metadata::<GenericMetadata>(mut_id)
                    .unwrap(),
                Some(metadata[mi])
            );
            assert_eq!(
                GenericMetadata::decode(&tables.mutations().row(mut_id).unwrap().metadata.unwrap())
                    .unwrap(),
                *m
            );
        }
    }
}

#[cfg(test)]
mod test_adding_site {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_site_without_metadata() {
        let mut tables = make_empty_table_collection(11.0);
        let site_id = tables.add_site(0.1, None).unwrap();

        match site_id {
            SiteId(0) => (),
            _ => panic!("Expected SiteId(0)"),
        };

        assert_eq!(site_id, 0);

        assert!(tables
            .sites()
            .metadata::<GenericMetadata>(site_id)
            .unwrap()
            .is_none());

        let row = tables.sites().row(site_id).unwrap();
        assert_eq!(row.id, site_id);
        assert!(row.ancestral_state.is_none());
        assert!(row.metadata.is_none());
    }

    #[test]
    fn test_adding_site_with_metadata() {
        let mut tables = make_empty_table_collection(11.0);
        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 12345 }];

        for (mi, m) in metadata.iter().enumerate() {
            let site_id = match tables.add_site_with_metadata(0.1, None, m) {
                Ok(SiteId(x)) => SiteId(x),
                Err(_) => panic!("unexpected Err"),
            };
            assert_eq!(
                tables.sites().metadata::<GenericMetadata>(site_id).unwrap(),
                Some(metadata[mi])
            );
        }
        for i in 0..usize::try_from(tables.sites().num_rows()).unwrap() {
            assert!(
                tables.sites().row(SiteId::from(i as tsk_id_t)).unwrap()
                    == tables.sites().row(SiteId::from(i as tsk_id_t)).unwrap()
            );
            if i > 0 {
                assert!(
                    tables.sites().row(SiteId::from(i as tsk_id_t)).unwrap()
                        != tables
                            .sites()
                            .row(SiteId::from((i - 1) as tsk_id_t))
                            .unwrap()
                );
            }
        }
    }
}

#[cfg(test)]
mod test_adding_population {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_adding_population_without_metadata() {
        let mut tables = make_empty_table_collection(11.0);
        let pop_id = tables.add_population().unwrap();

        assert!(pop_id == PopulationId(0));
        assert!(pop_id == 0);
        assert!(tables
            .populations()
            .metadata::<GenericMetadata>(pop_id)
            .unwrap()
            .is_none());

        for row in tables.populations_iter() {
            assert!(row.metadata.is_none());
        }

        for row in tables.populations().iter() {
            assert!(row.metadata.is_none());
        }

        assert!(
            tables.populations().row(pop_id).unwrap() == tables.populations().row(pop_id).unwrap()
        );
    }

    #[test]
    fn test_adding_population_with_metadata() {
        let mut tables = make_empty_table_collection(11.0);
        let pop_id = tables
            .add_population_with_metadata(&GenericMetadata::default())
            .unwrap();
        assert!(
            tables
                .populations()
                .metadata::<GenericMetadata>(pop_id)
                .unwrap()
                == Some(GenericMetadata::default())
        );
    }
}

#[cfg(test)]
mod test_adding_migrations {
    use crate::test_fixtures::make_empty_table_collection;
    use crate::test_fixtures::GenericMetadata;
    use crate::*;

    #[test]
    fn test_add_migration_without_metadata() {
        let mut tables = make_empty_table_collection(1.0);
        let mig_id = tables.add_migration((0., 1.), 7, (0, 1), 1e-3).unwrap();

        match mig_id {
            MigrationId(0) => (),
            _ => panic!("Extend MigrationId(0)"),
        };

        assert_eq!(mig_id, 0);
        assert_eq!(tables.migrations().node(mig_id).unwrap(), NodeId(7));
        assert_eq!(tables.migrations().source(mig_id).unwrap(), PopulationId(0));
        assert_eq!(tables.migrations().dest(mig_id).unwrap(), PopulationId(1));
        assert!(tables
            .migrations()
            .metadata::<GenericMetadata>(mig_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_add_migration_with_metadata() {
        use crate::metadata::MetadataRoundtrip;

        let metadata = vec![GenericMetadata::default(), GenericMetadata { data: 84 }];

        let mut tables = make_empty_table_collection(1.0);

        for (i, md) in metadata.iter().enumerate() {
            let id_i = i as tsk_id_t;
            let mig_id =
                tables.add_migration_with_metadata((0., 1.), 7 * id_i, (id_i, id_i + 1), 1e-3, md);

            match mig_id {
                Ok(MigrationId(x)) => {
                    assert_eq!(x, id_i);
                }
                Err(_) => panic!("got unexpected error"),
            };

            let mig_id = mig_id.unwrap();

            let row = tables.migrations().row(mig_id).unwrap();
            assert_eq!(row.id, mig_id);
            assert_eq!(row.source, PopulationId(id_i * tsk_id_t::from(mig_id)));
            assert_eq!(row.dest, PopulationId(id_i * tsk_id_t::from(mig_id) + 1));
            assert_eq!(row.node, NodeId(7 * id_i));
        }

        for i in 0..tables.migrations().num_rows().try_into().unwrap() {
            assert!(
                tables.migrations().row(MigrationId::from(i)).unwrap()
                    == tables.migrations().row(MigrationId::from(i)).unwrap()
            );
            if i > 0 {
                assert!(
                    tables.migrations().row(MigrationId::from(i)).unwrap()
                        != tables.migrations().row(MigrationId::from(i - 1)).unwrap()
                );
            }
        }

        for (i, r) in tables.migrations_iter().enumerate() {
            assert_eq!(r.id, i as crate::tsk_id_t);
            assert_eq!(
                GenericMetadata::decode(&r.metadata.unwrap()).unwrap(),
                metadata[i]
            );
        }

        for (i, r) in tables.migrations().iter().enumerate() {
            assert_eq!(r.id, i as crate::tsk_id_t);
            assert_eq!(
                GenericMetadata::decode(&r.metadata.unwrap()).unwrap(),
                metadata[i]
            );
        }
    }
}
