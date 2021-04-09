use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::ffi::{TskitTypeAccess, WrapTskitConsumingType};
use crate::{tsk_flags_t, tsk_id_t, tsk_size_t, TableCollection, TSK_NULL};
use bitflags::bitflags;
use ll_bindings::{tsk_tree_free, tsk_treeseq_free};

bitflags! {
    #[derive(Default)]
    pub struct TreeFlags: tsk_flags_t {
        const NONE = 0;
        const SAMPLE_LISTS = ll_bindings::TSK_SAMPLE_LISTS;
        const NO_SAMPLE_COUNTS = ll_bindings::TSK_NO_SAMPLE_COUNTS;
    }
}

pub struct Tree {
    inner: Box<ll_bindings::tsk_tree_t>,
    current_tree: i32,
    advanced: bool,
    num_nodes: tsk_size_t,
    flags: TreeFlags,
}

pub type BoxedNodeIterator = Box<dyn NodeIterator>;

drop_for_tskit_type!(Tree, tsk_tree_free);
tskit_type_access!(Tree, ll_bindings::tsk_tree_t);

impl Tree {
    fn wrap(num_nodes: tsk_size_t, flags: TreeFlags) -> Self {
        let temp: std::mem::MaybeUninit<ll_bindings::tsk_tree_t> = std::mem::MaybeUninit::uninit();
        Self {
            inner: unsafe { Box::<ll_bindings::tsk_tree_t>::new(temp.assume_init()) },
            current_tree: 0,
            advanced: false,
            num_nodes,
            flags,
        }
    }

    fn new(ts: &TreeSequence, flags: TreeFlags) -> Result<Self, TskitError> {
        let mut tree = Self::wrap(ts.consumed.nodes().num_rows(), flags);
        let rv = unsafe { ll_bindings::tsk_tree_init(tree.as_mut_ptr(), ts.as_ptr(), flags.bits) };
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

    fn parent_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.parent, self.inner.num_nodes)
    }

    fn samples_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.samples, num_samples)
        )
    }

    fn next_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.next_sample, self.inner.num_nodes)
        )
    }

    #[allow(dead_code)]
    fn left_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.left_sample, self.inner.num_nodes)
        )
    }

    #[allow(dead_code)]
    fn right_sample_array(&self) -> Result<crate::ffi::TskIdArray, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            crate::ffi::TskIdArray::new(self.inner.right_sample, self.inner.num_nodes)
        )
    }

    #[allow(dead_code)]
    fn left_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_sib, self.inner.num_nodes)
    }

    fn right_sib_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_sib, self.inner.num_nodes)
    }

    fn left_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.left_child, self.inner.num_nodes)
    }

    #[allow(dead_code)]
    fn right_child_array(&self) -> crate::ffi::TskIdArray {
        crate::ffi::TskIdArray::new(self.inner.right_child, self.inner.num_nodes)
    }

    fn left_sample(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_sample).unwrap()
        )
    }

    fn right_sample(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        err_if_not_tracking_samples!(
            self.flags,
            unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_sample).unwrap()
        )
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
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.parent)
    }

    pub fn left_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_child)
    }

    pub fn right_child(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_child)
    }

    pub fn left_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.left_sib)
    }

    pub fn right_sib(&self, u: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(u, 0, self.num_nodes, self.inner.right_sib)
    }

    pub fn samples_to_vec(&self) -> Vec<tsk_id_t> {
        let num_samples =
            unsafe { ll_bindings::tsk_treeseq_get_num_samples((*self.as_ptr()).tree_sequence) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = unsafe { *(*(*self.as_ptr()).tree_sequence).samples.offset(i as isize) };
            rv.push(u);
        }
        rv
    }

    pub fn path_to_root(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = PathToRootIterator::new(self, u)?;
        Ok(Box::new(iter))
    }

    pub fn children(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = ChildIterator::new(self, u)?;
        Ok(Box::new(iter))
    }

    pub fn samples(&self, u: tsk_id_t) -> Result<BoxedNodeIterator, TskitError> {
        let iter = SamplesIterator::new(self, u)?;
        Ok(Box::new(iter))
    }

    pub fn roots(&self) -> BoxedNodeIterator {
        Box::new(RootIterator::new(self))
    }

    pub fn roots_to_vec(&self) -> Vec<tsk_id_t> {
        let mut v = vec![];

        for r in self.roots() {
            v.push(r);
        }

        v
    }

    pub fn nodes(&self, order: NodeTraversalOrder) -> BoxedNodeIterator {
        match order {
            NodeTraversalOrder::Preorder => Box::new(PreorderNodeIterator::new(&self)),
        }
    }

    pub fn node_table<'a>(&'a self) -> crate::NodeTable<'a> {
        crate::NodeTable::<'a>::new_from_table(unsafe {
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

pub trait NodeIterator {
    fn next_node(&mut self);
    fn current_node(&mut self) -> Option<tsk_id_t>;
}

impl Iterator for dyn NodeIterator {
    type Item = tsk_id_t;

    fn next(&mut self) -> Option<tsk_id_t> {
        self.next_node();
        self.current_node()
    }
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
            root_stack: tree.roots_to_vec(),
            node_stack: vec![],
            left_child: tree.left_child_array(),
            right_sib: tree.right_sib_array(),
            current_node_: None,
        };
        rv.root_stack.reverse();
        if let Some(root) = rv.root_stack.pop() {
            rv.node_stack.push(root);
        }
        rv
    }
}

impl NodeIterator for PreorderNodeIterator {
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
            None => {
                if let Some(r) = self.root_stack.pop() {
                    self.current_node_ = Some(r);
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node_
    }
}

struct RootIterator {
    current_root: Option<tsk_id_t>,
    next_root: tsk_id_t,
    right_sib: crate::ffi::TskIdArray,
}

impl RootIterator {
    fn new(tree: &Tree) -> Self {
        RootIterator {
            current_root: None,
            next_root: tree.inner.left_root,
            right_sib: tree.right_sib_array(),
        }
    }
}

impl NodeIterator for RootIterator {
    fn next_node(&mut self) {
        self.current_root = match self.next_root {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_root = self.right_sib[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_root
    }
}

struct ChildIterator {
    current_child: Option<tsk_id_t>,
    next_child: tsk_id_t,
    right_sib: crate::ffi::TskIdArray,
}

impl ChildIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        let c = tree.left_child(u)?;

        Ok(ChildIterator {
            current_child: None,
            next_child: c,
            right_sib: tree.right_sib_array(),
        })
    }
}

impl NodeIterator for ChildIterator {
    fn next_node(&mut self) {
        self.current_child = match self.next_child {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_child = self.right_sib[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_child
    }
}

struct PathToRootIterator {
    current_node: Option<tsk_id_t>,
    next_node: tsk_id_t,
    parent: crate::ffi::TskIdArray,
}

impl PathToRootIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        match u >= tree.num_nodes as tsk_id_t {
            true => Err(TskitError::IndexError),
            false => Ok(PathToRootIterator {
                current_node: None,
                next_node: u,
                parent: tree.parent_array(),
            }),
        }
    }
}

impl NodeIterator for PathToRootIterator {
    fn next_node(&mut self) {
        self.current_node = match self.next_node {
            TSK_NULL => None,
            r => {
                assert!(r >= 0);
                let cr = Some(r);
                self.next_node = self.parent[r];
                cr
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node
    }
}

struct SamplesIterator {
    current_node: Option<tsk_id_t>,
    next_sample_index: tsk_id_t,
    last_sample_index: tsk_id_t,
    next_sample: crate::ffi::TskIdArray,
    samples: crate::ffi::TskIdArray,
}

impl SamplesIterator {
    fn new(tree: &Tree, u: tsk_id_t) -> Result<Self, TskitError> {
        let rv = SamplesIterator {
            current_node: None,
            next_sample_index: tree.left_sample(u)?,
            last_sample_index: tree.right_sample(u)?,
            next_sample: tree.next_sample_array()?,
            samples: tree.samples_array()?,
        };

        Ok(rv)
    }
}

impl NodeIterator for SamplesIterator {
    fn next_node(&mut self) {
        self.current_node = match self.next_sample_index {
            TSK_NULL => None,
            r => {
                if r == self.last_sample_index {
                    let cr = Some(self.samples[r]);
                    self.next_sample_index = TSK_NULL;
                    cr
                } else {
                    assert!(r >= 0);
                    let cr = Some(self.samples[r]);
                    self.next_sample_index = self.next_sample[r];
                    cr
                }
            }
        };
    }

    fn current_node(&mut self) -> Option<tsk_id_t> {
        self.current_node
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

    pub fn tree_iterator(&self, flags: TreeFlags) -> Result<Tree, TskitError> {
        let tree = Tree::new(self, flags)?;

        Ok(tree)
    }

    pub fn samples_to_vec(&self) -> Vec<tsk_id_t> {
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

    pub fn kc_distance(&self, other: &TreeSequence, lambda: f64) -> Result<f64, TskitError> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        let code = unsafe {
            ll_bindings::tsk_treeseq_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        };
        handle_tsk_return_value!(code, kc)
    }
}

#[cfg(test)]
mod test_trees {
    use super::*;
    use crate::TSK_NODE_IS_SAMPLE;
    use streaming_iterator::StreamingIterator;

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

    fn treeseq_from_small_table_collection() -> TreeSequence {
        let tables = make_small_table_collection();
        tables.tree_sequence().unwrap()
    }

    #[test]
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let treeseq = TreeSequence::new(tables).unwrap();
        let samples = treeseq.samples_to_vec();
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
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            ntrees += 1;
            assert_eq!(tree.current_tree, ntrees);
            let samples = tree.samples_to_vec();
            assert_eq!(samples.len(), 2);
            for i in 1..3 {
                assert_eq!(samples[i - 1], i as tsk_id_t);

                let mut nsteps = 0;
                for _ in tree.path_to_root(samples[i - 1]).unwrap() {
                    nsteps += 1;
                }
                assert_eq!(nsteps, 2);
            }
            let roots = tree.roots_to_vec();
            for r in roots.iter() {
                let mut num_children = 0;
                for _ in tree.children(*r).unwrap() {
                    num_children += 1;
                }
                assert_eq!(num_children, 2);
            }
        }
        assert_eq!(ntrees, 1);
    }

    #[test]
    fn test_iterate_no_roots() {
        let mut tables = TableCollection::new(100.).unwrap();
        tables.build_index(0).unwrap();
        let treeseq = tables.tree_sequence().unwrap();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            let mut num_roots = 0;
            for _ in tree.roots() {
                num_roots += 1;
            }
            assert_eq!(num_roots, 0);
        }
    }

    #[should_panic]
    #[test]
    fn test_samples_iterator_error_when_not_tracking_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence().unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            for n in tree.nodes(NodeTraversalOrder::Preorder) {
                for _ in tree.samples(n).unwrap() {}
            }
        }
    }

    #[test]
    fn test_iterate_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence().unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            let mut s = vec![];
            for i in tree.samples(0).unwrap() {
                s.push(i);
            }
            assert_eq!(s.len(), 2);
            assert_eq!(s[0], 1);
            assert_eq!(s[1], 2);

            for u in 1..3 {
                let mut s = vec![];
                for i in tree.samples(u).unwrap() {
                    s.push(i);
                }
                assert_eq!(s.len(), 1);
                assert_eq!(s[0], u);
            }
        } else {
            panic!("Expected a tree");
        }
    }

    #[test]
    fn test_kc_distance_naive_test() {
        let ts1 = treeseq_from_small_table_collection();
        let ts2 = treeseq_from_small_table_collection();

        let kc = ts1.kc_distance(&ts2, 0.0).unwrap();
        assert!(kc.is_finite());
        assert!((kc - 0.).abs() < f64::EPSILON);
    }
}
