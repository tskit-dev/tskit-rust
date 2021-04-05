use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::{TskitConsumingType, TskitType};
use crate::{tsk_flags_t, TableCollection, TSK_NULL};
use ll_bindings::tsk_treeseq_free;

/// A tree sequence.
///
/// This is a thin wrapper around the C type `tsk_treeseq_t`.
///
/// When created from a [`TableCollection`], the input tables are
/// moved into the `TreeSequence` object.
/// # Examples
///
/// ```
/// let mut tables = tskit::TableCollection::new(1000.).unwrap();
/// tables.add_node(0, 1.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::TSK_NULL, tskit::TSK_NULL).unwrap();
/// tables.add_edge(0., 1000., 0, 1).unwrap();
/// tables.add_edge(0., 1000., 0, 2).unwrap();
///
/// // tables gets moved into our treeseq variable:
/// let treeseq = tables.tree_sequence();
/// ```
pub struct TreeSequence {
    consumed: TableCollection,
    inner: Box<ll_bindings::tsk_treeseq_t>,
}

build_consuming_tskit_type!(
    TreeSequence,
    ll_bindings::tsk_treeseq_t,
    tsk_treeseq_free,
    TableCollection
);

impl TreeSequence {
    /// Create a tree sequence from a [`TableCollection`].
    /// In general, [`TableCollection::tree_sequence`] may be preferred.
    /// The table collection is moved/consumed.
    pub fn new(tables: TableCollection) -> Result<Self, TskitError> {
        let mut treeseq = Self::wrap(tables);
        let rv = unsafe {
            ll_bindings::tsk_treeseq_init(treeseq.as_mut_ptr(), treeseq.consumed.as_ptr(), 0)
        };
        if rv < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: rv });
        }
        Ok(treeseq)
    }

    /// Obtain a copy of the [`TableCollection`]
    pub fn dump_tables(&self) -> Result<TableCollection, TskitError> {
        self.consumed.deepcopy()
    }
}

#[cfg(test)]
mod test_trees {
    use super::*;

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
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let _treeseq = TreeSequence::new(tables).unwrap();
    }

    #[test]
    fn test_create_treeseq_from_tables() {
        let tables = make_small_table_collection();
        let _treeseq = tables.tree_sequence().unwrap();
    }
}
