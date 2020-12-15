use crate::bindings as ll_bindings;
use crate::error::TskitRustError;
use crate::types::Bookmark;
use crate::EdgeTable;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SiteTable;
use crate::TskReturnValue;
use crate::{tsk_flags_t, tsk_id_t, tsk_size_t};

/// Handle allocation details.
fn new_tsk_table_collection_t() -> Result<Box<ll_bindings::tsk_table_collection_t>, TskitRustError>
{
    let mut tsk_tables: std::mem::MaybeUninit<ll_bindings::tsk_table_collection_t> =
        std::mem::MaybeUninit::uninit();
    let rv = unsafe { ll_bindings::tsk_table_collection_init(tsk_tables.as_mut_ptr(), 0) };
    if rv < 0 {
        return Err(TskitRustError::ErrorCode { code: rv });
    }
    let rv = unsafe { Box::<ll_bindings::tsk_table_collection_t>::new(tsk_tables.assume_init()) };
    Ok(rv)
}

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
    tables: Box<ll_bindings::tsk_table_collection_t>,
}

impl TableCollection {
    /// Create a new table collection with a sequence length.
    pub fn new(sequence_length: f64) -> Result<Self, TskitRustError> {
        if sequence_length <= 0. {
            return Err(TskitRustError::ValueError {
                got: sequence_length.to_string(),
                expected: "sequence_length >= 0.0".to_string(),
            });
        }
        let tables = new_tsk_table_collection_t();
        match tables {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        let mut rv = TableCollection {
            tables: tables.unwrap(),
        };
        rv.tables.sequence_length = sequence_length;
        Ok(rv)
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
            return Err(TskitRustError::ErrorCode { code: rv });
        }

        return Ok(tables);
    }

    /// Access to raw C pointer as const tsk_table_collection_t *.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_table_collection_t {
        return &*self.tables;
    }

    /// Access to raw C pointer as tsk_table_collection_t *.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_table_collection_t {
        return &mut *self.tables;
    }

    /// Length of the sequence/"genome".
    pub fn sequence_length(&self) -> f64 {
        return unsafe { (*self.as_ptr()).sequence_length };
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn edges<'a>(&'a self) -> EdgeTable<'a> {
        return EdgeTable::<'a>::new_from_table(&self.tables.edges);
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn nodes<'a>(&'a self) -> NodeTable<'a> {
        return NodeTable::<'a>::new_from_table(&self.tables.nodes);
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn sites<'a>(&'a self) -> SiteTable<'a> {
        return SiteTable::<'a>::new_from_table(&self.tables.sites);
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn mutations<'a>(&'a self) -> MutationTable<'a> {
        return MutationTable::<'a>::new_from_table(&self.tables.mutations);
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    /// Lifetime of return value is tied to (this)
    /// parent object.
    pub fn populations<'a>(&'a self) -> PopulationTable<'a> {
        return PopulationTable::<'a>::new_from_table(&self.tables.populations);
    }

    /// Add a row to the edge table
    pub fn add_edge(
        &mut self,
        left: f64,
        right: f64,
        parent: tsk_id_t,
        child: tsk_id_t,
    ) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*self.as_mut_ptr()).edges,
                left,
                right,
                parent,
                child,
                std::ptr::null(),
                0,
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
        let rv = unsafe {
            ll_bindings::tsk_node_table_add_row(
                &mut (*self.as_mut_ptr()).nodes,
                flags,
                time,
                population,
                individual,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv);
    }

    /// Add a row to the site table
    pub fn add_site(&mut self, position: f64, ancestral_state: Option<&[u8]>) -> TskReturnValue {
        let astate = if ancestral_state.is_some() {
            (
                std::ffi::CString::new(ancestral_state.unwrap()).unwrap(),
                ancestral_state.unwrap().len() as tsk_size_t,
            )
        } else {
            (std::ffi::CString::new("".to_string()).unwrap(), 0)
        };

        let rv = unsafe {
            ll_bindings::tsk_site_table_add_row(
                &mut (*self.as_mut_ptr()).sites,
                position,
                astate.0.as_ptr(),
                astate.1,
                std::ptr::null(),
                0,
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
        let dstate = if derived_state.is_some() {
            (
                std::ffi::CString::new(derived_state.unwrap()).unwrap(),
                derived_state.unwrap().len() as tsk_size_t,
            )
        } else {
            (std::ffi::CString::new("".to_string()).unwrap(), 0)
        };

        let rv = unsafe {
            ll_bindings::tsk_mutation_table_add_row(
                &mut (*self.as_mut_ptr()).mutations,
                site,
                node,
                parent,
                time,
                dstate.0.as_ptr(),
                dstate.1,
                std::ptr::null(),
                0,
            )
        };

        handle_tsk_return_value!(rv);
    }

    /// Add a row to the population_table
    pub fn add_population(&mut self) -> TskReturnValue {
        let rv = unsafe {
            ll_bindings::tsk_population_table_add_row(
                &mut (*self.as_mut_ptr()).populations,
                std::ptr::null(),
                0,
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
        return self.sort(&b, 0);
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
    fn free(&mut self) -> TskReturnValue {
        let rv = unsafe { ll_bindings::tsk_table_collection_free(self.as_mut_ptr()) };

        handle_tsk_return_value!(rv);
    }

    /// Return ``true`` if ``self`` contains the same
    /// data as ``other``, and ``false`` otherwise.
    pub fn equals(&self, other: &TableCollection, options: tsk_flags_t) -> bool {
        let rv = unsafe {
            ll_bindings::tsk_table_collection_equals(self.as_ptr(), other.as_ptr(), options)
        };
        return rv;
    }
}

impl Drop for TableCollection {
    fn drop(&mut self) {
        let rv = unsafe { ll_bindings::tsk_table_collection_free(&mut *self.tables) };
        panic_on_tskit_error!(rv);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sequence_length() {
        let tables = TableCollection::new(1000.).unwrap();
        assert_eq!(tables.sequence_length(), 1000.);
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
        tables.add_site(0.3, Some("Eggnog".as_bytes())).unwrap();
        tables.add_site(0.5, None).unwrap(); // No ancestral_state specified!!!
        let longer_metadata = "Hot Toddy";
        tables
            .add_site(0.9, Some(longer_metadata.as_bytes()))
            .unwrap();

        let sites = tables.sites();
        assert_eq!(sites.position(0).unwrap(), 0.3);
        assert_eq!(sites.position(1).unwrap(), 0.5);
        assert_eq!(sites.position(2).unwrap(), 0.9);

        match sites.ancestral_state(0).unwrap() {
            Some(astate) => assert_eq!(astate, "Eggnog".as_bytes()),
            None => assert!(false),
        };

        match sites.ancestral_state(1).unwrap() {
            Some(_) => assert!(false),
            None => assert!(true),
        };

        match sites.ancestral_state(2).unwrap() {
            Some(astate) => assert_eq!(astate, longer_metadata.as_bytes()),
            None => assert!(false),
        };
    }

    #[test]
    fn test_add_mutation() {
        let mut tables = TableCollection::new(1000.).unwrap();

        tables
            .add_mutation(0, 0, crate::TSK_NULL, 1.123, Some("pajamas".as_bytes()))
            .unwrap();
        tables
            .add_mutation(1, 1, crate::TSK_NULL, 2.123, None)
            .unwrap();
        tables
            .add_mutation(
                2,
                2,
                crate::TSK_NULL,
                3.123,
                Some("more pajamas".as_bytes()),
            )
            .unwrap();
        let mutations = tables.mutations();
        assert_eq!(mutations.time(0).unwrap(), 1.123);
        assert_eq!(mutations.time(1).unwrap(), 2.123);
        assert_eq!(mutations.time(2).unwrap(), 3.123);
        assert_eq!(mutations.node(0).unwrap(), 0);
        assert_eq!(mutations.node(1).unwrap(), 1);
        assert_eq!(mutations.node(2).unwrap(), 2);
        assert_eq!(mutations.parent(0).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(1).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(2).unwrap(), crate::TSK_NULL);
        assert_eq!(
            mutations.derived_state(0).unwrap().unwrap(),
            "pajamas".as_bytes()
        );
        match mutations.derived_state(1).unwrap() {
            Some(_) => assert!(false),
            None => assert!(true),
        };
        assert_eq!(
            mutations.derived_state(2).unwrap().unwrap(),
            "more pajamas".as_bytes()
        );
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
