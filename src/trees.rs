use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::{TskitTypeAccess, WrapTskitConsumingType};
use crate::{tsk_flags_t, tsk_id_t, tsk_size_t, TableCollection, TSK_NULL};
use ll_bindings::{tsk_tree_free, tsk_treeseq_free};
use streaming_iterator::StreamingIterator;

pub struct Tree {
    inner: Box<ll_bindings::tsk_tree_t>,
    current_tree: i32,
    advanced: bool,
    num_nodes: tsk_size_t,
}

drop_for_tskit_type!(Tree, tsk_tree_free);
tskit_type_access!(Tree, ll_bindings::tsk_tree_t);

impl Tree {
    fn wrap(num_nodes: tsk_size_t) -> Self {
        let temp: std::mem::MaybeUninit<ll_bindings::tsk_tree_t> = std::mem::MaybeUninit::uninit();
        Self {
            inner: unsafe { Box::<ll_bindings::tsk_tree_t>::new(temp.assume_init()) },
            current_tree: 0,
            advanced: false,
            num_nodes,
        }
    }

    fn new(ts: &TreeSequence) -> Result<Self, TskitError> {
        let mut tree = Self::wrap(ts.consumed.nodes().num_rows());
        let rv = unsafe { ll_bindings::tsk_tree_init(tree.as_mut_ptr(), ts.as_ptr(), 0) };
        handle_tsk_return_value!(rv, tree)
    }

    fn advance_details(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_first(self.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_next(self.as_mut_ptr()) }
        };
        if rv == 0 {
            self.advanced = false;
            self.current_tree += 1;
        } else if rv == 1 {
            self.advanced = true;
            self.current_tree += 1;
        } else if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }

    pub fn left_root(&self) -> tsk_id_t {
        self.inner.left_root
    }

    pub fn parent(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.parent);
    }

    pub fn left_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_child);
    }

    pub fn right_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_child);
    }

    pub fn left_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_sib);
    }

    pub fn right_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_sib);
    }

    pub fn sample_list(&self) -> Vec<tsk_id_t> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = unsafe { *(*(*self.as_ptr()).tree_sequence).samples.offset(i as isize) };
            rv.push(u);
        }
        rv
    }
}

impl streaming_iterator::StreamingIterator for Tree {
    type Item = Tree;
    fn advance(&mut self) {
        self.advance_details();
    }

    fn get(&self) -> Option<&Tree> {
        match self.advanced {
            true => Some(&self),
            false => None,
        }
    }
}

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
        handle_tsk_return_value!(rv, treeseq)
    }

    /// Obtain a copy of the [`TableCollection`]
    pub fn dump_tables(&self) -> Result<TableCollection, TskitError> {
        self.consumed.deepcopy()
    }

    pub fn tree_iterator(&self) -> Result<Tree, TskitError> {
        let tree = Tree::new(self)?;

        Ok(tree)
    }

    pub fn sample_list(&self) -> Vec<tsk_id_t> {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = unsafe { *(*self.as_ptr()).samples.offset(i as isize) };
            rv.push(u);
        }
        rv
    }
}

#[cfg(test)]
mod test_trees {
    use super::*;
    use crate::TSK_NODE_IS_SAMPLE;

    fn make_small_table_collection() -> TableCollection {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_node(0, 1.0, TSK_NULL, TSK_NULL).unwrap();
        tables
            .add_node(TSK_NODE_IS_SAMPLE, 0.0, TSK_NULL, TSK_NULL)
            .unwrap();
        tables
            .add_node(TSK_NODE_IS_SAMPLE, 0.0, TSK_NULL, TSK_NULL)
            .unwrap();
        tables.add_edge(0., 1000., 0, 1).unwrap();
        tables.add_edge(0., 1000., 0, 2).unwrap();
        tables.build_index(0).unwrap();
        tables
    }

    #[test]
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let treeseq = TreeSequence::new(tables).unwrap();
        let samples = treeseq.sample_list();
        assert_eq!(samples.len(), 2);
        for i in 1..3 {
            assert_eq!(samples[i - 1], i as tsk_id_t);
        }
    }

    #[test]
    fn test_create_treeseq_from_tables() {
        let tables = make_small_table_collection();
        let _treeseq = tables.tree_sequence().unwrap();
    }

    #[test]
    fn test_iterate_tree_seq_with_one_tree() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence().unwrap();
        let mut ntrees = 0;
        let mut tree_iter = treeseq.tree_iterator().unwrap();
        while let Some(tree) = tree_iter.next() {
            ntrees += 1;
            assert_eq!(tree.current_tree, ntrees);
            let samples = tree.sample_list();
            assert_eq!(samples.len(), 2);
            for i in 1..3 {
                assert_eq!(samples[i - 1], i as tsk_id_t);
            }
        }
        assert_eq!(ntrees, 1);
    }
}
