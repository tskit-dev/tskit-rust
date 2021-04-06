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

    fn left_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_sib, self.inner.num_nodes)
    }

    fn right_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_sib, self.inner.num_nodes)
    }

    fn left_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_child, self.inner.num_nodes)
    }

    fn right_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_child, self.inner.num_nodes)
    }

    pub fn interval(&self) -> (f64, f64) {
        unsafe { ((*self.as_ptr()).left, (*self.as_ptr()).right) }
    }

    pub fn span(&self) -> f64 {
        let i = self.interval();
        i.1 - i.0
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

    pub fn traverse_to_root(
        &self,
        u: tsk_id_t,
        mut f: impl FnMut(tsk_id_t) -> (),
    ) -> Result<(), TskitError> {
        let mut p = u;
        while p != TSK_NULL {
            f(p);
            p = self.parent(p)?;
        }
        Ok(())
    }

    pub fn process_children(
        &self,
        u: tsk_id_t,
        mut f: impl FnMut(tsk_id_t) -> (),
    ) -> Result<(), TskitError> {
        let mut c = self.left_child(u)?;
        while c != TSK_NULL {
            f(c);
            c = self.right_sib(c)?;
        }
        Ok(())
    }

    pub fn roots(&self) -> Result<Vec<tsk_id_t>, TskitError> {
        let mut v = vec![];

        let mut r = self.inner.left_root;

        while r != TSK_NULL {
            v.push(r);
            r = self.right_sib(r)?;
        }

        Ok(v)
    }

    pub fn nodes(&self, order: NodeTraversalOrder) -> Box<dyn NodeIteration> {
        match order {
            NodeTraversalOrder::Preorder => Box::new(PreorderNodeIterator::new(&self)),
        }
    }

    pub fn node_table<'a>(&'a self) -> crate::NodeTable<'a> {
        crate::NodeTable::new_from_table(unsafe {
            &(*(*(*self.as_ptr()).tree_sequence).tables).nodes
        })
    }

    pub fn total_branch_length(&self, by_span: bool) -> Result<f64, TskitError> {
        let nt = self.node_table();
        let mut b = 0.;
        for n in self.nodes(NodeTraversalOrder::Preorder) {
            let p = self.parent(n)?;
            if p != TSK_NULL {
                b += nt.time(p)? - nt.time(n)?;
            }
        }

        match by_span {
            true => Ok(b * self.span()),
            false => Ok(b),
        }
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

pub enum NodeTraversalOrder {
    Preorder,
}

struct PreorderNodeIterator {
    root_stack: Vec<i32>,
    node_stack: Vec<i32>,
    left_child: crate::ffi::TskIdArray,
    right_sib: crate::ffi::TskIdArray,
    current_node_: Option<tsk_id_t>,
}

impl PreorderNodeIterator {
    fn new(tree: &Tree) -> Self {
        let mut rv = PreorderNodeIterator {
            root_stack: tree.roots().unwrap(),
            node_stack: vec![],
            left_child: tree.left_child_array(),
            right_sib: tree.right_sib_array(),
            current_node_: None,
        };
        rv.root_stack.reverse();
        let root = rv.root_stack.pop();
        match root {
            Some(x) => rv.node_stack.push(x),
            None => (),
        };

        rv
    }
}

pub trait NodeIteration {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<tsk_id_t>;
}

impl NodeIteration for PreorderNodeIterator {
    fn next_node(&mut self) {
        self.current_node_ = self.node_stack.pop();
        match self.current_node_ {
            Some(u) => {
                let mut c = self.left_child[u];
                while c != TSK_NULL {
                    self.node_stack.push(c);
                    c = self.right_sib[c];
                }
            }
            None => match self.root_stack.pop() {
                Some(r) => {
                    self.current_node_ = Some(r);
                }
                None => (),
            },
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node_
    }
}

impl Iterator for dyn NodeIteration {
    type Item = tsk_id_t;

    fn next(&mut self) -> Option<tsk_id_t> {
        self.next_node();
        self.current_node()
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

    pub fn load(filename: &str) -> Result<Self, TskitError> {
        let tables = TableCollection::new_from_file(filename)?;

        Self::new(tables)
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

    pub fn num_trees(&self) -> tsk_size_t {
        unsafe { ll_bindings::tsk_treeseq_get_num_trees(self.as_ptr()) }
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

                let mut nsteps = 0;
                tree.traverse_to_root(samples[i - 1], |_x: tsk_id_t| {
                    nsteps += 1;
                })
                .unwrap();
                assert_eq!(nsteps, 2);
            }
            let roots = tree.roots().unwrap();
            for r in roots.iter() {
                let mut num_children = 0;
                tree.process_children(*r, |_x: tsk_id_t| {
                    num_children += 1;
                })
                .unwrap();
                assert_eq!(num_children, 2);
            }
        }
        assert_eq!(ntrees, 1);
    }
}
