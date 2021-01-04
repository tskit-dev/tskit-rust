use crate::bindings as ll_bindings;
use crate::error::TskitRustError;
use crate::ffi::TskitType;
use crate::metadata::*;
use crate::types::Bookmark;
use crate::EdgeTable;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SiteTable;
use crate::TskReturnValue;
use crate::{tsk_flags_t, tsk_id_t};
use ll_bindings::tsk_table_collection_free;

/// A table collection.
///
/// This is a thin wrapper around the C type `tsk_table_collection_t`.
///
/// # Current limitations
///
/// 1. No support for adding metadata to tables.
///    In later versions, we will add separate
///    "add row" functions to allow metadata.
///
/// The issue with metadata is just a temporary
/// uncertainty in how best to handle the ``char *``
/// round trips to/from ``rust``.
///
/// # Examples
///
/// ```
/// let mut tables = tskit_rust::TableCollection::new(100.).unwrap();
/// assert_eq!(tables.sequence_length(), 100.);
///
/// // Adding edges:
///
/// let rv = tables.add_edge(0., 53., 1, 11).unwrap();
///
/// // Add node:
///
/// let rv = tables.add_node(0, 3.2, tskit_rust::TSK_NULL, tskit_rust::TSK_NULL).unwrap();
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
/// # Future road map
///
/// 1. Support all table types.  Currently, we only support
///    those needed for current goals in ongoing projects.
/// 2. For all ``add_foo`` functions, add an additional
///    ``add_foo_with_metadata``. (See above)
/// 3. Strengthen some of the error handling.
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
    pub fn new(sequence_length: f64) -> Result<Self, TskitRustError> {
        if sequence_length <= 0. {
            return Err(TskitRustError::ValueError {
                got: sequence_length.to_string(),
                expected: "sequence_length >= 0.0".to_string(),
            });
        }
        let mut tables = Self::wrap();
        let rv = unsafe { ll_bindings::tsk_table_collection_init(tables.as_mut_ptr(), 0) };
        if rv < 0 {
            return Err(crate::error::TskitRustError::ErrorCode { code: rv });
        }
        tables.inner.sequence_length = sequence_length;
        Ok(tables)
    }

    /// Load a table collection from a file.
    pub fn new_from_file(filename: &str) -> Result<Self, TskitRustError> {
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

        if rv < 0 {
            Err(TskitRustError::ErrorCode { code: rv })
        } else {
            Ok(tables)
        }
    }

    /// Length of the sequence/"genome".
    pub fn sequence_length(&self) -> f64 {
        unsafe { (*self.as_ptr()).sequence_length }
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn edges<'a>(&'a self) -> EdgeTable<'a> {
        EdgeTable::<'a>::new_from_table(&self.inner.edges)
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn nodes<'a>(&'a self) -> NodeTable<'a> {
        NodeTable::<'a>::new_from_table(&self.inner.nodes)
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn sites<'a>(&'a self) -> SiteTable<'a> {
        SiteTable::<'a>::new_from_table(&self.inner.sites)
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn mutations<'a>(&'a self) -> MutationTable<'a> {
        MutationTable::<'a>::new_from_table(&self.inner.mutations)
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn populations<'a>(&'a self) -> PopulationTable<'a> {
        PopulationTable::<'a>::new_from_table(&self.inner.populations)
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

        handle_tsk_return_value!(rv);
    }

    /// Add a row to the node table
    pub fn add_node(
        &mut self,
        flags: ll_bindings::tsk_flags_t,
        time: f64,
        population: tsk_id_t,
        individual: tsk_id_t,
    ) -> TskReturnValue {
        self.add_node_witha_metadata(flags, time, population, individual, None)
    }

    /// Add a row with metadata to the node table
    pub fn add_node_witha_metadata(
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

        handle_tsk_return_value!(rv);
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

        handle_tsk_return_value!(rv);
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

        handle_tsk_return_value!(rv);
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

        handle_tsk_return_value!(rv);
    }

    /// Build the "input" and "output"
    /// indexes for the edge table.
    ///
    /// `flags` is currently unused, so pass in `0`.
    pub fn build_index(&mut self, flags: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_build_index(self.as_mut_ptr(), flags) };
        handle_tsk_return_value!(rv);
    }

    /// Sort the tables.  
    /// The [``bookmark``](crate::types::Bookmark) can
    /// be used to affect where sorting starts from for each table.
    pub fn sort(&mut self, start: &Bookmark, options: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_sort(self.as_mut_ptr(), &start.offsets, options)
        };

        handle_tsk_return_value!(rv);
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

        handle_tsk_return_value!(rv);
    }

    /// Clear the contents of all tables.
    /// Does not release memory.
    /// Memory will be released when the object goes out
    /// of scope.
    pub fn clear(&mut self, options: tsk_flags_t) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_clear(self.as_mut_ptr(), options) };

        handle_tsk_return_value!(rv);
    }

    /// Free all memory allocated on the C side.
    /// Not public b/c not very safe.
    #[allow(dead_code)]
    fn free(&mut self) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_free(self.as_mut_ptr()) };

        handle_tsk_return_value!(rv);
    }

    /// Return ``true`` if ``self`` contains the same
    /// data as ``other``, and ``false`` otherwise.
    pub fn equals(&self, other: &TableCollection, options: tsk_flags_t) -> bool {
        unsafe { ll_bindings::tsk_table_collection_equals(self.as_ptr(), other.as_ptr(), options) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
    }

    struct F {
        x: i32,
        y: u32,
    }

    impl MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            let mut rv = vec![];
            rv.extend(self.x.to_le_bytes().iter().map(|&i| i));
            rv.extend(self.y.to_le_bytes().iter().map(|&i| i));
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
}
