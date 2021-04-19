use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::WrapTskitType;
use crate::metadata::*;
use crate::types::Bookmark;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SimplificationOptions;
use crate::SiteTable;
use crate::TableAccess;
use crate::TskReturnValue;
use crate::TskitTypeAccess;
use crate::{tsk_flags_t, tsk_id_t, tsk_size_t, TSK_NULL};
use ll_bindings::tsk_table_collection_free;

/// A table collection.
///
/// This is a thin wrapper around the C type `tsk_table_collection_t`.
///
/// # Current limitations
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
/// let rv = tables.add_node(0, 3.2, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
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
/// ## Metadata round trips and table iteration
///
/// ```
/// use tskit;
/// use tskit::TableAccess;
/// use tskit::metadata::MetadataRoundtrip;
///
/// // Define a type for metadata
/// struct F {
///     x: i32,
/// }
///
/// // Implement our metadata trait for type F.
/// // NOTE: this is hard because we are only using the
/// // rust standard library here.  See the examples/
/// // directory of the repository for examples using
/// // other, more convenient, crates.
/// impl tskit::metadata::MetadataRoundtrip for F {
///     fn encode(&self) -> Result<Vec<u8>, tskit::metadata::MetadataError> {
///         let mut rv = vec![];
///         rv.extend(self.x.to_le_bytes().iter().copied());
///         Ok(rv)
///     }
///     fn decode(md: &[u8]) -> Result<Self, tskit::metadata::MetadataError> {
///         use std::convert::TryInto;
///         let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
///         Ok(Self {
///             x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
///         })
///     }
/// }
///
/// // Crate a table and add a mutation with metadata
/// let mut tables = tskit::TableCollection::new(100.).unwrap();
///
/// // The metadata takes a reference in the event that it could
/// // be data store in some container somewhere, and you don't want
/// // it moved.
/// tables.add_mutation_with_metadata(0, 0, 0, 0., None, Some(&F{x: -33})).unwrap();
///
/// // Iterate over each row in the table.
/// // The "true" means to include (a copy of) the
/// // encoded metadata, if any exist.
/// for row in tables.mutations().iter(true) {
///     // Decode the metadata if any exists.
///     if !row.metadata.is_none() {
///         let md = F::decode(&row.metadata.unwrap()).unwrap();
///         assert_eq!(md.x, -33);
///     }
/// }
/// ```
///
/// # Future road map
///
/// 1. Support all table types.  Currently, we only support
///    those needed for current goals in ongoing projects.
/// 2. Strengthen some of the error handling.
///
/// Addressing point 3 may require API breakage.
pub struct TableCollection {
    inner: Box<ll_bindings::tsk_table_collection_t>,
}

build_tskit_type!(
    TableCollection,
    ll_bindings::tsk_table_collection_t,
    tsk_table_collection_free
);

impl TableCollection {
    /// Create a new table collection with a sequence length.
    pub fn new(sequence_length: f64) -> Result<Self, TskitError> {
        if sequence_length <= 0. {
            return Err(TskitError::ValueError {
                got: sequence_length.to_string(),
                expected: "sequence_length >= 0.0".to_string(),
            });
        }
        let mut tables = Self::wrap();
        let rv = unsafe { ll_bindings::tsk_table_collection_init(tables.as_mut_ptr(), 0) };
        if rv < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: rv });
        }
        tables.inner.sequence_length = sequence_length;
        Ok(tables)
    }

    /// Load a table collection from a file.
    pub fn new_from_file(filename: &str) -> Result<Self, TskitError> {
        let tables = TableCollection::new(1.0); // Arbitrary sequence_length.
        match tables {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        let mut tables = tables.unwrap();

        let c_str = std::ffi::CString::new(filename).unwrap();
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
    pub fn sequence_length(&self) -> f64 {
        unsafe { (*self.as_ptr()).sequence_length }
    }

    /// Add a row to the edge table
    pub fn add_edge(
        &mut self,
        left: f64,
        right: f64,
        parent: tsk_id_t,
        child: tsk_id_t,
    ) -> TskReturnValue {
        self.add_edge_with_metadata(left, right, parent, child, None)
    }

    /// Add a row with metadata to the edge table
    pub fn add_edge_with_metadata(
        &mut self,
        left: f64,
        right: f64,
        parent: tsk_id_t,
        child: tsk_id_t,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*self.as_mut_ptr()).edges,
                left,
                right,
                parent,
                child,
                md.as_ptr(),
                md.len(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Add a row to the individual table
    pub fn add_individual(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
    ) -> TskReturnValue {
        self.add_individual_with_metadata(flags, location, parents, None)
    }

    /// Add a row with metadata to the individual table
    pub fn add_individual_with_metadata(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_individual_table_add_row(
                &mut (*self.as_mut_ptr()).individuals,
                flags,
                location.as_ptr(),
                location.len() as tsk_size_t,
                parents.as_ptr(),
                parents.len() as tsk_size_t,
                md.as_ptr(),
                md.len(),
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Add a row to the migration table
    ///
    /// # Warnings
    ///
    /// Migration tables are not currently supported
    /// by tree sequence simplification.
    pub fn add_migration(
        &mut self,
        span: (f64, f64),
        node: tsk_id_t,
        source_dest: (tsk_id_t, tsk_id_t),
        time: f64,
    ) -> TskReturnValue {
        self.add_migration_with_metadata(span, node, source_dest, time, None)
    }

    /// Add a row with metadata to the migration table
    ///
    /// # Warnings
    ///
    /// Migration tables are not currently supported
    /// by tree sequence simplification.
    pub fn add_migration_with_metadata(
        &mut self,
        span: (f64, f64),
        node: tsk_id_t,
        source_dest: (tsk_id_t, tsk_id_t),
        time: f64,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_migration_table_add_row(
                &mut (*self.as_mut_ptr()).migrations,
                span.0,
                span.1,
                node,
                source_dest.0,
                source_dest.1,
                time,
                md.as_ptr(),
                md.len(),
            )
        };
        handle_tsk_return_value!(rv)
    }

    /// Add a row to the node table
    pub fn add_node(
        &mut self,
        flags: ll_bindings::tsk_flags_t,
        time: f64,
        population: tsk_id_t,
        individual: tsk_id_t,
    ) -> TskReturnValue {
        self.add_node_with_metadata(flags, time, population, individual, None)
    }

    /// Add a row with metadata to the node table
    pub fn add_node_with_metadata(
        &mut self,
        flags: ll_bindings::tsk_flags_t,
        time: f64,
        population: tsk_id_t,
        individual: tsk_id_t,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_node_table_add_row(
                &mut (*self.as_mut_ptr()).nodes,
                flags,
                time,
                population,
                individual,
                md.as_ptr(),
                md.len(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Add a row to the site table
    pub fn add_site(&mut self, position: f64, ancestral_state: Option<&[u8]>) -> TskReturnValue {
        self.add_site_with_metadata(position, ancestral_state, None)
    }

    /// Add a row with metadata to the site table
    pub fn add_site_with_metadata(
        &mut self,
        position: f64,
        ancestral_state: Option<&[u8]>,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let astate = process_state_input!(ancestral_state);
        let md = EncodedMetadata::new(metadata)?;

        let rv = unsafe {
            ll_bindings::tsk_site_table_add_row(
                &mut (*self.as_mut_ptr()).sites,
                position,
                astate.0,
                astate.1,
                md.as_ptr(),
                md.len(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Add a row to the mutation table.
    pub fn add_mutation(
        &mut self,
        site: tsk_id_t,
        node: tsk_id_t,
        parent: tsk_id_t,
        time: f64,
        derived_state: Option<&[u8]>,
    ) -> TskReturnValue {
        self.add_mutation_with_metadata(site, node, parent, time, derived_state, None)
    }

    /// Add a row with metadata to the mutation table.
    pub fn add_mutation_with_metadata(
        &mut self,
        site: tsk_id_t,
        node: tsk_id_t,
        parent: tsk_id_t,
        time: f64,
        derived_state: Option<&[u8]>,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let dstate = process_state_input!(derived_state);
        let md = EncodedMetadata::new(metadata)?;

        let rv = unsafe {
            ll_bindings::tsk_mutation_table_add_row(
                &mut (*self.as_mut_ptr()).mutations,
                site,
                node,
                parent,
                time,
                dstate.0,
                dstate.1,
                md.as_ptr(),
                md.len(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Add a row to the population_table
    pub fn add_population(&mut self) -> TskReturnValue {
        self.add_population_with_metadata(None)
    }

    /// Add a row with metadata to the population_table
    pub fn add_population_with_metadata(
        &mut self,
        metadata: Option<&dyn MetadataRoundtrip>,
    ) -> TskReturnValue {
        let md = EncodedMetadata::new(metadata)?;
        let rv = unsafe {
            ll_bindings::tsk_population_table_add_row(
                &mut (*self.as_mut_ptr()).populations,
                md.as_ptr(),
                md.len(),
            )
        };

        handle_tsk_return_value!(rv)
    }

    /// Build the "input" and "output"
    /// indexes for the edge table.
    ///
    /// `flags` is currently unused, so pass in `0`.
    pub fn build_index(&mut self, flags: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_build_index(self.as_mut_ptr(), flags) };
        handle_tsk_return_value!(rv)
    }

    /// Sort the tables.  
    /// The [``bookmark``](crate::types::Bookmark) can
    /// be used to affect where sorting starts from for each table.
    pub fn sort(&mut self, start: &Bookmark, options: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_sort(self.as_mut_ptr(), &start.offsets, options)
        };

        handle_tsk_return_value!(rv)
    }

    /// Fully sort all functions.
    /// Implemented via a call to [``sort``](crate::TableCollection::sort).
    pub fn full_sort(&mut self) -> TskReturnValue {
        let b = Bookmark::new();
        self.sort(&b, 0)
    }

    /// Dump the table collection to file.
    /// If tables are not sorted and indexes, this function will raise
    /// and error.  In order to output such data,
    /// include [``TSK_NO_BUILD_INDEXES``](crate::TSK_NO_BUILD_INDEXES) in ``options``.
    /// Otherwisze, use ``0`` for ``options``.
    pub fn dump(&mut self, filename: &str, options: tsk_flags_t) -> TskReturnValue {
        let c_str = std::ffi::CString::new(filename).unwrap();
        let rv = unsafe {
            ll_bindings::tsk_table_collection_dump(self.as_mut_ptr(), c_str.as_ptr(), options)
        };

        handle_tsk_return_value!(rv)
    }

    /// Clear the contents of all tables.
    /// Does not release memory.
    /// Memory will be released when the object goes out
    /// of scope.
    pub fn clear(&mut self, options: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_clear(self.as_mut_ptr(), options) };

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
    pub fn equals(&self, other: &TableCollection, options: tsk_flags_t) -> bool {
        unsafe { ll_bindings::tsk_table_collection_equals(self.as_ptr(), other.as_ptr(), options) }
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
    pub fn tree_sequence(self) -> Result<crate::TreeSequence, TskitError> {
        crate::TreeSequence::new(self)
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
    ///   this vector either contains the node's new index or [`TSK_NULL`]
    ///   if the input node is not part of the simplified history.
    pub fn simplify(
        &mut self,
        samples: &[tsk_id_t],
        options: SimplificationOptions,
        idmap: bool,
    ) -> Result<Option<Vec<tsk_id_t>>, TskitError> {
        let mut output_node_map: Vec<tsk_id_t> = vec![];
        if idmap {
            output_node_map.resize(self.nodes().num_rows() as usize, TSK_NULL);
        }
        let rv = unsafe {
            ll_bindings::tsk_table_collection_simplify(
                self.as_mut_ptr(),
                samples.as_ptr(),
                samples.len() as tsk_size_t,
                options.bits(),
                match idmap {
                    true => output_node_map.as_mut_ptr(),
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
}

impl TableAccess for TableCollection {
    fn edges(&self) -> EdgeTable {
        EdgeTable::new_from_table(&self.inner.edges)
    }

    fn individuals(&self) -> IndividualTable {
        IndividualTable::new_from_table(&self.inner.individuals)
    }

    fn migrations(&self) -> MigrationTable {
        MigrationTable::new_from_table(&self.inner.migrations)
    }

    fn nodes(&self) -> NodeTable {
        NodeTable::new_from_table(&self.inner.nodes)
    }

    fn sites(&self) -> SiteTable {
        SiteTable::new_from_table(&self.inner.sites)
    }

    fn mutations(&self) -> MutationTable {
        MutationTable::new_from_table(&self.inner.mutations)
    }

    fn populations(&self) -> PopulationTable {
        PopulationTable::new_from_table(&self.inner.populations)
    }
}

impl crate::traits::NodeListGenerator for TableCollection {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TSK_NULL;

    fn make_small_table_collection() -> TableCollection {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_node(0, 1.0, TSK_NULL, TSK_NULL).unwrap();
        tables.add_node(0, 0.0, TSK_NULL, TSK_NULL).unwrap();
        tables.add_node(0, 0.0, TSK_NULL, TSK_NULL).unwrap();
        tables.add_edge(0., 1000., 0, 1).unwrap();
        tables.add_edge(0., 1000., 0, 2).unwrap();
        tables.build_index(0).unwrap();
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
    fn test_node_iteration() {
        let tables = make_small_table_collection();
        for (i, row) in tables.nodes().iter(true).enumerate() {
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

        for row in tables.nodes_iter(true) {
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
        for (i, row) in tables.edges().iter(true).enumerate() {
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
        for row in tables.edges_iter(true) {
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
        for (i, row) in sites.iter(true).enumerate() {
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
        for row in tables.sites_iter(true) {
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

    fn close_enough(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_add_mutation() {
        let mut tables = TableCollection::new(1000.).unwrap();

        tables
            .add_mutation(0, 0, crate::TSK_NULL, 1.123, Some(b"pajamas"))
            .unwrap();
        tables
            .add_mutation(1, 1, crate::TSK_NULL, 2.123, None)
            .unwrap();
        tables
            .add_mutation(2, 2, crate::TSK_NULL, 3.123, Some(b"more pajamas"))
            .unwrap();
        let mutations = tables.mutations();
        assert!(close_enough(mutations.time(0).unwrap(), 1.123));
        assert!(close_enough(mutations.time(1).unwrap(), 2.123));
        assert!(close_enough(mutations.time(2).unwrap(), 3.123));
        assert_eq!(mutations.node(0).unwrap(), 0);
        assert_eq!(mutations.node(1).unwrap(), 1);
        assert_eq!(mutations.node(2).unwrap(), 2);
        assert_eq!(mutations.parent(0).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(1).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(2).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.derived_state(0).unwrap().unwrap(), b"pajamas");

        if mutations.derived_state(1).unwrap().is_some() {
            panic!()
        }

        assert_eq!(
            mutations.derived_state(2).unwrap().unwrap(),
            b"more pajamas"
        );

        let mut nmuts = 0;
        for (i, row) in tables.mutations().iter(true).enumerate() {
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
        for row in tables.mutations_iter(true) {
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
        for row in tables.mutations().iter(false) {
            assert!(row.metadata.is_none());
        }

        nmuts = 0;
        for _ in tables.mutations().iter(true).skip(1) {
            nmuts += 1;
        }
        assert_eq!(nmuts, tables.mutations().num_rows() - 1);
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
            use std::convert::TryInto;
            let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
            let (y_int_bytes, _) = rest.split_at(std::mem::size_of::<u32>());
            Ok(Self {
                x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
                y: u32::from_le_bytes(y_int_bytes.try_into().unwrap()),
            })
        }
    }

    #[test]
    fn test_add_mutation_with_metadata() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_mutation_with_metadata(
                0,
                0,
                crate::TSK_NULL,
                1.123,
                None,
                Some(&F { x: -3, y: 666 }),
            )
            .unwrap();
        // The double unwrap is to first check for error
        // and then to process the Option.
        let md = tables.mutations().metadata::<F>(0).unwrap().unwrap();
        assert_eq!(md.x, -3);
        assert_eq!(md.y, 666);

        for row in tables.mutations().iter(true) {
            assert!(!row.metadata.is_none());
            let md = F::decode(&row.metadata.unwrap()).unwrap();
            assert_eq!(md.x, -3);
            assert_eq!(md.y, 666);
        }
    }

    #[test]
    fn test_add_mutation_with_metadata_for_some_columns() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_mutation_with_metadata(
                0,
                0,
                crate::TSK_NULL,
                1.123,
                None,
                Some(&F { x: -3, y: 666 }),
            )
            .unwrap();

        tables
            .add_mutation_with_metadata(1, 2, crate::TSK_NULL, 2.0, None, None)
            .unwrap();

        let mut num_with_metadata = 0;
        let mut num_without_metadata = 0;
        for i in 0..tables.mutations().num_rows() {
            match tables.mutations().metadata::<F>(i as tsk_id_t).unwrap() {
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
        tables.add_population().unwrap();
        assert_eq!(tables.populations().num_rows(), 1);
    }

    #[test]
    fn test_dump_tables() {
        let treefile = "trees.trees";
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_population().unwrap();
        tables
            .add_node(
                crate::TSK_NODE_IS_SAMPLE,
                0.0,
                crate::TSK_NULL,
                crate::TSK_NULL,
            )
            .unwrap();
        tables
            .add_node(
                crate::TSK_NODE_IS_SAMPLE,
                1.0,
                crate::TSK_NULL,
                crate::TSK_NULL,
            )
            .unwrap();
        tables.add_edge(0., tables.sequence_length(), 1, 0).unwrap();
        tables.dump(&treefile, 0).unwrap();

        let tables2 = TableCollection::new_from_file(&treefile).unwrap();
        assert!(tables.equals(&tables2, 0));

        std::fs::remove_file(&treefile).unwrap();
    }

    #[test]
    fn test_clear() {
        let mut tables = TableCollection::new(1000.).unwrap();
        for i in 0..5 {
            let _ = tables.add_edge(0., 1000., i, 2 * i).unwrap();
        }
        assert_eq!(tables.edges().num_rows(), 5);
        tables.clear(0).unwrap();
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
        assert!(tables.equals(&dumps, 0));
    }

    #[test]
    fn test_edge_table_row_equality() {
        let tables = make_small_table_collection();
        for (i, row) in tables.edges_iter(true).enumerate() {
            assert!(row.id == i as tsk_id_t);
            assert!(row == tables.edges().row(i as tsk_id_t, true).unwrap());
            assert!(!(row != tables.edges().row(i as tsk_id_t, true).unwrap()));
            if i > 0 {
                assert!(row != tables.edges().row(i as tsk_id_t - 1, true).unwrap());
            }
        }
    }

    #[test]
    fn test_node_table_row_equality() {
        let tables = make_small_table_collection();
        for (i, row) in tables.nodes_iter(true).enumerate() {
            assert!(row.id == i as tsk_id_t);
            assert!(row == tables.nodes().row(i as tsk_id_t, true).unwrap());
            assert!(!(row != tables.nodes().row(i as tsk_id_t, true).unwrap()));
        }
        assert!(tables.nodes().row(0, true) != tables.nodes().row(1, true));
        assert!(tables.nodes().row(1, true) != tables.nodes().row(2, true));
    }

    #[test]
    fn test_add_migration() {
        let mut tables = TableCollection::new(1.).unwrap();
        tables.add_migration((0., 0.25), 0, (0, 1), 0.).unwrap();
    }

    #[test]
    fn test_add_individual_with_location_and_parents() {
        let mut tables = TableCollection::new(1.).unwrap();
        let location = vec![0., 1., 2.];
        let parents = [0, 1, 2, 3, 4];
        tables.add_individual(0, &location, &parents).unwrap();

        match tables.individuals().parents(0).unwrap() {
            Some(x) => assert!(x == parents),
            None => panic!("expected some parents"),
        }

        match tables.individuals().location(0).unwrap() {
            Some(x) => {
                assert_eq!(x.len(), location.len());
                for (i, l) in x.iter().enumerate() {
                    assert!(crate::util::f64_partial_cmp_equal(&l, &location[i]));
                }
            }
            None => panic!("expected some locations"),
        }

        assert!(
            tables.individuals().row(0, true).unwrap()
                == tables.individuals().row(0, true).unwrap()
        );
    }
}
