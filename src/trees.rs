use std::ops::Deref;
use std::ops::DerefMut;

use crate::bindings as ll_bindings;
use crate::error::TskitError;
use crate::sys;
use crate::NodeId;
use crate::SimplificationOptions;
use crate::SizeType;
use crate::TableOutputOptions;
use crate::TreeFlags;
use crate::TreeInterface;
use crate::TreeSequenceFlags;
use crate::TskReturnValue;
use crate::{tsk_id_t, TableCollection};
use ll_bindings::tsk_tree_free;
use std::ptr::NonNull;
use streaming_iterator::StreamingIterator;

/// A Tree.
///
/// Wrapper around `tsk_tree_t`.
pub struct Tree {
    pub(crate) inner: mbox::MBox<ll_bindings::tsk_tree_t>,
    api: TreeInterface,
    current_tree: i32,
    advanced: bool,
}

impl Drop for Tree {
    fn drop(&mut self) {
        // SAFETY: Mbox<_> cannot hold a NULL ptr
        let rv = unsafe { tsk_tree_free(self.inner.as_mut()) };
        assert_eq!(rv, 0);
    }
}

impl Deref for Tree {
    type Target = TreeInterface;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}

impl Tree {
    fn new<F: Into<TreeFlags>>(ts: &TreeSequence, flags: F) -> Result<Self, TskitError> {
        let flags = flags.into();

        // SAFETY: this is the type we want :)
        let temp = unsafe {
            libc::malloc(std::mem::size_of::<ll_bindings::tsk_tree_t>())
                as *mut ll_bindings::tsk_tree_t
        };

        // Get our pointer into MBox ASAP
        let nonnull = NonNull::<ll_bindings::tsk_tree_t>::new(temp)
            .ok_or_else(|| TskitError::LibraryError("failed to malloc tsk_tree_t".to_string()))?;

        // SAFETY: if temp is NULL, we have returned Err already.
        let mut tree = unsafe { mbox::MBox::from_non_null_raw(nonnull) };
        let mut rv =
            unsafe { ll_bindings::tsk_tree_init(tree.as_mut(), ts.as_ptr(), flags.bits()) };
        if rv < 0 {
            return Err(TskitError::ErrorCode { code: rv });
        }
        // Gotta ask Jerome about this one--why isn't this handled in tsk_tree_init??
        if !flags.contains(TreeFlags::NO_SAMPLE_COUNTS) {
            // SAFETY: nobody is null here.
            rv = unsafe {
                ll_bindings::tsk_tree_set_tracked_samples(
                    tree.as_mut(),
                    ts.num_samples().into(),
                    (tree.as_mut()).samples,
                )
            };
        }

        let num_nodes = unsafe { (*(*ts.as_ptr()).tables).nodes.num_rows };
        let api = TreeInterface::new(nonnull, num_nodes, num_nodes + 1, flags);
        handle_tsk_return_value!(
            rv,
            Tree {
                inner: tree,
                current_tree: 0,
                advanced: false,
                api
            }
        )
    }

    pub fn new_at_index<F: Into<TreeFlags>>(
        ts: &TreeSequence,
        tree_index: SizeType,
        tree_indexes: &TreesIndex,
        flags: F,
    ) -> Result<Self, TskitError> {
        let mut tree = Self::new(ts, flags)?;

        let num_edges = ts.edges().num_rows().as_usize();

        let edge_insertion = unsafe {
            std::slice::from_raw_parts(
                (*(*ts.as_ptr()).tables).indexes.edge_insertion_order,
                num_edges,
            )
        };
        let edge_removal = unsafe {
            std::slice::from_raw_parts(
                (*(*ts.as_ptr()).tables).indexes.edge_removal_order,
                num_edges,
            )
        };
        let left = ts.edges().left_slice();
        let right = ts.edges().right_slice();
        let parent = ts.edges().parent_slice();
        let child = ts.edges().child_slice();

        // FIXME: will panic if index is out of range
        let pos = tree_indexes.left[tree_index.as_usize()];
        let seqlen = unsafe { (*ts.as_ref().tables).sequence_length };
        if pos <= seqlen / 2. {
            for e in edge_insertion {
                let idx = usize::try_from(*e).unwrap();
                if left[idx] > pos {
                    break;
                }
                if pos >= left[idx] && pos < right[idx] {
                    unsafe {
                        ll_bindings::tsk_tree_insert_edge(
                            tree.as_mut_ptr(),
                            parent[idx].into(),
                            child[idx].into(),
                            *e,
                        )
                    }
                }
            }
        } else {
            for e in edge_removal.iter().rev() {
                let idx = usize::try_from(*e).unwrap();
                if right[idx] < pos {
                    break;
                }
                if pos >= left[idx] && pos < right[idx] {
                    unsafe {
                        ll_bindings::tsk_tree_insert_edge(
                            tree.as_mut_ptr(),
                            parent[idx].into(),
                            child[idx].into(),
                            *e,
                        )
                    }
                }
            }
        }

        // clunky -- seems we should be working with i32 and not a size type.
        unsafe { (*tree.as_mut_ptr()).index = tree_index.as_usize() as i32 };
        unsafe { (*tree.as_mut_ptr()).interval.left = pos };

        let num_trees: u64 = ts.num_trees().into();

        let right = if tree_index < num_trees - 1 {
            tree_indexes.left[tree_index.as_usize() + 1]
        } else {
            unsafe { (*ts.as_ref().tables).sequence_length }
        };
        unsafe { (*tree.as_mut_ptr()).interval.right = right };

        // this is the part I am unsure of
        unsafe {
            (*tree.as_mut_ptr()).left_index =
                tree_indexes.insertion[tree_index.as_usize() + 1] as i32
        };
        unsafe {
            (*tree.as_mut_ptr()).right_index =
                tree_indexes.removal[tree_index.as_usize() + 1] as i32
        };
        unsafe { (*tree.as_mut_ptr()).num_nodes = (*ts.as_ref().tables).nodes.num_rows };
        tree.current_tree = tree_index.as_usize() as i32;

        Ok(tree)
    }

    pub fn new_at_index_jk<F: Into<TreeFlags>>(
        ts: &TreeSequence,
        tree_index: SizeType,
        tree_indexes: &TreesIndex,
        flags: F,
    ) -> Result<Self, TskitError> {
        let mut tree = Self::new(ts, flags)?;

        let edge_left = ts.edges().left_slice();
        let edge_right = ts.edges().right_slice();
        let edge_parent = ts.edges().parent_slice();
        let edge_child = ts.edges().child_slice();
        let num_edges = edge_left.len();

        let edge_insertion = unsafe {
            std::slice::from_raw_parts(
                (*(*ts.as_ptr()).tables).indexes.edge_insertion_order,
                num_edges,
            )
        };
        let edge_removal = unsafe {
            std::slice::from_raw_parts(
                (*(*ts.as_ptr()).tables).indexes.edge_removal_order,
                num_edges,
            )
        };

        // FIXME: will panic if index is out of range
        let pos = tree_indexes.left[tree_index.as_usize()];

        let seqlen = unsafe { (*ts.as_ref().tables).sequence_length };
        let mut j = 0_usize;
        let mut k = 0_usize;
        let mut right: f64;
        let mut left = 0.0;

        //while (j < num_edges || left <= seqlen) && pos >= left {
        while j < num_edges  && pos >= left {
            println!("{} {} {} {} | {}", j, num_edges, left, seqlen, pos);
            while k < num_edges && edge_right[edge_removal[k] as usize] == left {
                k += 1;
            }
            while j < num_edges && edge_left[edge_insertion[j] as usize] == left {
                if pos >= edge_left[edge_insertion[j] as usize]
                    && pos < edge_right[edge_insertion[j] as usize]
                {
                    let p: i32 = edge_parent[edge_insertion[j] as usize].into();
                    let c: i32 = edge_child[edge_insertion[j] as usize].into();
                    unsafe {
                        ll_bindings::tsk_tree_insert_edge(
                            tree.as_mut_ptr(),
                            p,
                            c,
                            edge_insertion[j],
                        )
                    };
                }
                j += 1;
            }
            right = seqlen;
            if j < num_edges {
                right = if right < edge_left[edge_insertion[j] as usize] {
                    right
                } else {
                    edge_left[edge_insertion[j] as usize].into()
                };
            }
            if k < num_edges {
                right = if right < edge_right[edge_removal[k] as usize] {
                    right
                } else {
                    edge_right[edge_removal[k] as usize].into()
                };
            }
            //if pos >= left && pos < right {
            //    break;
            //}
            left = right;
        }
        // HACK: why is this needed?
        if pos > seqlen / 2. {
            j -= 1;
            k -= 1;
        }
        // manually determine the tree index
        let breakpoints = unsafe {
            std::slice::from_raw_parts(ts.as_ref().breakpoints, ts.num_trees().as_usize())
        };
        let i = match breakpoints.iter().position(|b| b > &pos) {
            Some(value) => value - 1,
            None => panic!("bad things happened that should be an Err"),
        };
        assert_eq!(i, tree_index.as_usize());
        assert!(pos >= breakpoints[i]);
        assert!(pos < breakpoints[i + 1]);
        // clunky -- seems we should be working with i32 and not a size type.
        unsafe { (*tree.as_mut_ptr()).index = tree_index.as_usize() as i32 };
        unsafe { (*tree.as_mut_ptr()).interval.left = pos };

        let num_trees: u64 = ts.num_trees().into();

        let right = if tree_index < num_trees - 1 {
            breakpoints[tree_index.as_usize() + 1]
        } else {
            unsafe { (*ts.as_ref().tables).sequence_length }
        };
        unsafe { (*tree.as_mut_ptr()).interval.right = right };

        // this is the part I am unsure of
        unsafe { (*tree.as_mut_ptr()).left_index = j as i32 };
        unsafe { (*tree.as_mut_ptr()).right_index = k as i32 };
        unsafe { (*tree.as_mut_ptr()).num_nodes = (*ts.as_ref().tables).nodes.num_rows };
        tree.current_tree = tree_index.as_usize() as i32;

        println!(
            "leaving with {} {}|{}, {} {}|{}",
            j,
            tree_indexes.insertion[tree_index.as_usize()],
            tree_indexes.insertion[tree_index.as_usize() + 1],
            k,
            tree_indexes.removal[tree_index.as_usize()],
            tree_indexes.removal[tree_index.as_usize() + 1],
        );

        Ok(tree)
    }
}

impl streaming_iterator::StreamingIterator for Tree {
    type Item = Tree;
    fn advance(&mut self) {
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

    fn get(&self) -> Option<&Tree> {
        match self.advanced {
            true => Some(self),
            false => None,
        }
    }
}

impl streaming_iterator::DoubleEndedStreamingIterator for Tree {
    fn advance_back(&mut self) {
        let rv = if self.current_tree == 0 {
            unsafe { ll_bindings::tsk_tree_last(self.as_mut_ptr()) }
        } else {
            unsafe { ll_bindings::tsk_tree_prev(self.as_mut_ptr()) }
        };
        if rv == 0 {
            self.advanced = false;
            self.current_tree -= 1;
        } else if rv == 1 {
            self.advanced = true;
            self.current_tree -= 1;
        } else if rv < 0 {
            panic_on_tskit_error!(rv);
        }
    }
}

pub struct TreesIndex {
    insertion: Vec<usize>,
    removal: Vec<usize>,
    left: Vec<f64>,
}

impl TreesIndex {
    pub fn new(treeseq: &TreeSequence) -> Result<Self, TskitError> {
        let mut insertion = vec![];
        let mut removal = vec![];
        let mut left = vec![];
        let mut j: usize = 0;
        let mut k: usize = 0;

        let mut diffs = treeseq.edge_differences_iter()?;

        while let Some(local_diffs) = diffs.next() {
            insertion.push(j);
            removal.push(k);
            left.push(local_diffs.left().into());
            k += local_diffs.edge_removals().count();
            j += local_diffs.edge_insertions().count();
        }
        Ok(Self {
            insertion,
            removal,
            left,
        })
    }
}

/// A tree sequence.
///
/// This is a thin wrapper around the C type `tsk_treeseq_t`.
///
/// When created from a [`TableCollection`], the input tables are
/// moved into the `TreeSequence` object.
///
/// # Examples
///
/// ```
/// let mut tables = tskit::TableCollection::new(1000.).unwrap();
/// tables.add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// tables.add_edge(0., 1000., 0, 1).unwrap();
/// tables.add_edge(0., 1000., 0, 2).unwrap();
///
/// // index
/// tables.build_index();
///
/// // tables gets moved into our treeseq variable:
/// let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
/// assert_eq!(treeseq.nodes().num_rows(), 3);
/// assert_eq!(treeseq.edges().num_rows(), 2);
/// ```
///
/// This type does not [`std::ops::DerefMut`] to [`crate::table_views::TableViews`]:
///
/// ```compile_fail
/// # let mut tables = tskit::TableCollection::new(1000.).unwrap();
/// # tables.add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_node(0, 0.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL).unwrap();
/// # tables.add_edge(0., 1000., 0, 1).unwrap();
/// # tables.add_edge(0., 1000., 0, 2).unwrap();
///
/// # // index
/// # tables.build_index();
///
/// # // tables gets moved into our treeseq variable:
/// # let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
/// assert_eq!(treeseq.nodes_mut().num_rows(), 3);
/// ```
pub struct TreeSequence {
    pub(crate) inner: sys::LLTreeSeq,
    views: crate::table_views::TableViews,
}

unsafe impl Send for TreeSequence {}
unsafe impl Sync for TreeSequence {}

impl TreeSequence {
    /// Create a tree sequence from a [`TableCollection`].
    /// In general, [`TableCollection::tree_sequence`] may be preferred.
    /// The table collection is moved/consumed.
    ///
    /// # Parameters
    ///
    /// * `tables`, a [`TableCollection`]
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if the tables are not indexed.
    /// * [`TskitError`] if the tables are not properly sorted.
    ///   See [`TableCollection::full_sort`](crate::TableCollection::full_sort).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tskit::TreeSequence::try_from(tables).unwrap();
    /// ```
    ///
    /// The following may be preferred to the previous example, and more closely
    /// mimics the Python `tskit` interface:
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// ```
    ///
    /// The following raises an error because the tables are not indexed:
    ///
    /// ```should_panic
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// let tree_sequence = tskit::TreeSequence::try_from(tables).unwrap();
    /// ```
    ///
    /// ## Note
    ///
    /// This function makes *no extra copies* of the tables.
    /// There is, however, a temporary allocation of an empty table collection
    /// in order to convince rust that we are safely handling all memory.
    pub fn new<F: Into<TreeSequenceFlags>>(
        tables: TableCollection,
        flags: F,
    ) -> Result<Self, TskitError> {
        let raw_tables_ptr = tables.into_raw()?;
        let mut inner = sys::LLTreeSeq::new(raw_tables_ptr, flags.into().bits())?;
        let views = crate::table_views::TableViews::new_from_tree_sequence(inner.as_mut_ptr())?;
        Ok(Self { inner, views })
    }

    fn as_ref(&self) -> &ll_bindings::tsk_treeseq_t {
        self.inner.as_ref()
    }

    /// Pointer to the low-level C type.
    pub fn as_ptr(&self) -> *const ll_bindings::tsk_treeseq_t {
        self.inner.as_ptr()
    }

    /// Mutable pointer to the low-level C type.
    pub fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_treeseq_t {
        self.inner.as_mut_ptr()
    }

    /// Dump the tree sequence to file.
    ///
    /// # Note
    ///
    /// * `options` is currently not used.  Set to default value.
    ///   This behavior may change in a future release, which could
    ///   break `API`.
    ///
    /// # Panics
    ///
    /// This function allocates a `CString` to pass the file name to the C API.
    /// A panic will occur if the system runs out of memory.
    pub fn dump<O: Into<TableOutputOptions>>(&self, filename: &str, options: O) -> TskReturnValue {
        let c_str = std::ffi::CString::new(filename).map_err(|_| {
            TskitError::LibraryError("call to ffi::Cstring::new failed".to_string())
        })?;
        self.inner
            .dump(c_str, options.into().bits())
            .map_err(|e| e.into())
    }

    /// Load from a file.
    ///
    /// This function calls [`TableCollection::new_from_file`] with
    /// [`TreeSequenceFlags::default`].
    pub fn load(filename: impl AsRef<str>) -> Result<Self, TskitError> {
        let tables = TableCollection::new_from_file(filename.as_ref())?;

        Self::new(tables, TreeSequenceFlags::default())
    }

    /// Obtain a copy of the [`TableCollection`].
    /// The result is a "deep" copy of the tables.
    ///
    /// # Errors
    ///
    /// [`TskitError`] will be raised if the underlying C library returns an error code.
    pub fn dump_tables(&self) -> Result<TableCollection, TskitError> {
        let mut inner = crate::table_collection::uninit_table_collection();

        let rv = unsafe {
            ll_bindings::tsk_table_collection_copy((*self.as_ptr()).tables, &mut *inner, 0)
        };

        // SAFETY: we just initialized it.
        // The C API doesn't free NULL pointers.
        handle_tsk_return_value!(rv, unsafe { TableCollection::new_from_mbox(inner)? })
    }

    /// Create an iterator over trees.
    ///
    /// # Parameters
    ///
    /// * `flags` A [`TreeFlags`] bit field.
    ///
    /// # Errors
    ///
    /// # Examples
    ///
    /// ```
    /// // You must include streaming_iterator as a dependency
    /// // and import this type.
    /// use streaming_iterator::StreamingIterator;
    /// // Import this to allow .next_back() for reverse
    /// // iteration over trees.
    /// use streaming_iterator::DoubleEndedStreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// let mut tree_iterator = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap();
    /// while let Some(tree) = tree_iterator.next() {
    /// }
    /// ```
    ///
    /// # Warning
    ///
    /// The following code results in an infinite loop.
    /// Be sure to note the difference from the previous example.
    ///
    /// ```no_run
    /// use streaming_iterator::StreamingIterator;
    ///
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// tables.build_index();
    /// let tree_sequence = tables.tree_sequence(tskit::TreeSequenceFlags::default()).unwrap();
    /// while let Some(tree) = tree_sequence.tree_iterator(tskit::TreeFlags::default()).unwrap().next() {
    /// }
    /// ```
    pub fn tree_iterator<F: Into<TreeFlags>>(&self, flags: F) -> Result<Tree, TskitError> {
        let tree = Tree::new(self, flags)?;

        Ok(tree)
    }

    pub fn tree_iterator_at_index<F: Into<TreeFlags>>(
        &self,
        index: SizeType,
        tree_indexes: &TreesIndex,
        flags: F,
    ) -> Result<Tree, TskitError> {
        let tree = Tree::new_at_index(self, index, tree_indexes, flags)?;

        Ok(tree)
    }

    pub fn tree_iterator_at_index_jk<F: Into<TreeFlags>>(
        &self,
        index: SizeType,
        tree_indexes: &TreesIndex,
        flags: F,
    ) -> Result<Tree, TskitError> {
        let tree = Tree::new_at_index_jk(self, index, tree_indexes, flags)?;

        Ok(tree)
    }

    /// Uses the indexes to cheat to find the right position
    pub fn tree_iterator_at_index_lib<F: Into<TreeFlags>>(
        &self,
        index: SizeType,
        tree_indexes: &TreesIndex,
        flags: F,
    ) -> Result<Tree, TskitError> {
        let index = index.as_usize();
        let position = tree_indexes.left[index];
        let flags = flags.into().bits();
        let mut tree = Tree::new(self, flags)?;
        tree.current_tree = index as i32;
        let rv = unsafe { ll_bindings::tsk_tree_seek(tree.as_mut_ptr(), position, flags) };
        handle_tsk_return_value!(rv, tree)
    }

    /// Get the list of samples as a vector.
    /// # Panics
    ///
    /// Will panic if the number of samples is too large to cast to a valid id.
    #[deprecated(
        since = "0.2.3",
        note = "Please use TreeSequence::sample_nodes instead"
    )]
    pub fn samples_to_vec(&self) -> Vec<NodeId> {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        let mut rv = vec![];

        for i in 0..num_samples {
            let u = match isize::try_from(i) {
                Ok(o) => NodeId::from(unsafe { *(*self.as_ptr()).samples.offset(o) }),
                Err(e) => panic!("{}", e),
            };
            rv.push(u);
        }
        rv
    }

    /// Get the list of sample nodes as a slice.
    pub fn sample_nodes(&self) -> &[NodeId] {
        let num_samples = unsafe { ll_bindings::tsk_treeseq_get_num_samples(self.as_ptr()) };
        sys::generate_slice(self.as_ref().samples, num_samples)
    }

    /// Get the number of trees.
    pub fn num_trees(&self) -> SizeType {
        self.inner.num_trees().into()
    }

    /// Calculate the average Kendall-Colijn (`K-C`) distance between
    /// pairs of trees whose intervals overlap.
    ///
    /// # Note
    ///
    /// * [Citation](https://doi.org/10.1093/molbev/msw124)
    ///
    /// # Parameters
    ///
    /// * `lambda` specifies the relative weight of topology and branch length.
    ///    See [`TreeInterface::kc_distance`] for more details.
    pub fn kc_distance(&self, other: &TreeSequence, lambda: f64) -> Result<f64, TskitError> {
        self.inner
            .kc_distance(&other.inner, lambda)
            .map_err(|e| e.into())
    }

    // FIXME: document
    pub fn num_samples(&self) -> SizeType {
        self.inner.num_samples().into()
    }

    /// Simplify tables and return a new tree sequence.
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
        &self,
        samples: &[NodeId],
        options: O,
        idmap: bool,
    ) -> Result<(Self, Option<Vec<NodeId>>), TskitError> {
        let mut output_node_map: Vec<NodeId> = vec![];
        if idmap {
            output_node_map.resize(usize::try_from(self.nodes().num_rows())?, NodeId::NULL);
        }
        let llsamples = unsafe {
            std::slice::from_raw_parts(samples.as_ptr().cast::<tsk_id_t>(), samples.len())
        };
        let mut inner = self.inner.simplify(
            llsamples,
            options.into().bits(),
            match idmap {
                true => output_node_map.as_mut_ptr().cast::<tsk_id_t>(),
                false => std::ptr::null_mut(),
            },
        )?;
        let views = crate::table_views::TableViews::new_from_tree_sequence(inner.as_mut_ptr())?;
        Ok((
            Self { inner, views },
            match idmap {
                true => Some(output_node_map),
                false => None,
            },
        ))
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
    /// The implementations used here use the [`humantime`](https://docs.rs/humantime/latest/humantime/) crate.
    ///
    /// # Parameters
    ///
    /// * `record`: the provenance record
    ///
    /// # Examples
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(1000.).unwrap();
    /// let mut treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
    /// # #[cfg(feature = "provenance")] {
    /// treeseq.add_provenance(&String::from("All your provenance r belong 2 us.")).unwrap();
    ///
    /// let prov_ref = treeseq.provenances();
    /// let row_0 = prov_ref.row(0).unwrap();
    /// assert_eq!(row_0.record, "All your provenance r belong 2 us.");
    /// let record_0 = prov_ref.record(0).unwrap();
    /// assert_eq!(record_0, row_0.record);
    /// let timestamp = prov_ref.timestamp(0).unwrap();
    /// assert_eq!(timestamp, row_0.timestamp);
    /// use core::str::FromStr;
    /// let dt_utc = humantime::Timestamp::from_str(&timestamp).unwrap();
    /// println!("utc = {}", dt_utc);
    /// # }
    /// ```
    pub fn add_provenance(&mut self, record: &str) -> Result<crate::ProvenanceId, TskitError> {
        if record.is_empty() {
            return Err(TskitError::ValueError {
                got: "empty string".to_string(),
                expected: "provenance record".to_string(),
            });
        }
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        let rv = unsafe {
            ll_bindings::tsk_provenance_table_add_row(
                &mut (*self.inner.as_ref().tables).provenances,
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as ll_bindings::tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as ll_bindings::tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, crate::ProvenanceId::from(rv))
    }

    delegate_table_view_api!();

    /// Build a lending iterator over edge differences.
    ///
    /// # Errors
    ///
    /// * [`TskitError`] if the `C` back end is unable to allocate
    ///   needed memory
    pub fn edge_differences_iter(
        &self,
    ) -> Result<crate::edge_differences::EdgeDifferencesIterator, TskitError> {
        crate::edge_differences::EdgeDifferencesIterator::new_from_treeseq(self, 0)
    }
}

impl TryFrom<TableCollection> for TreeSequence {
    type Error = TskitError;

    fn try_from(value: TableCollection) -> Result<Self, Self::Error> {
        Self::new(value, TreeSequenceFlags::default())
    }
}

#[cfg(test)]
pub(crate) mod test_trees {
    use super::*;
    use crate::test_fixtures::{
        make_small_table_collection, make_small_table_collection_two_trees,
        treeseq_from_small_table_collection, treeseq_from_small_table_collection_two_trees,
    };
    use crate::NodeTraversalOrder;
    use streaming_iterator::DoubleEndedStreamingIterator;
    use streaming_iterator::StreamingIterator;

    #[test]
    fn test_create_treeseq_new_from_tables() {
        let tables = make_small_table_collection();
        let treeseq = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
        let samples = treeseq.sample_nodes();
        assert_eq!(samples.len(), 2);
        for i in 1..3 {
            assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));
        }
    }

    #[test]
    fn test_create_treeseq_from_tables() {
        let tables = make_small_table_collection();
        let _treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    }

    #[test]
    fn test_iterate_tree_seq_with_one_tree() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let mut ntrees = 0;
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            ntrees += 1;
            assert_eq!(tree.current_tree, ntrees);
            let samples = tree.sample_nodes();
            assert_eq!(samples.len(), 2);
            for i in 1..3 {
                assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));

                let mut nsteps = 0;
                for _ in tree.parents(samples[i - 1]) {
                    nsteps += 1;
                }
                assert_eq!(nsteps, 2);
            }

            // These nodes are all out of range
            for i in 100..110 {
                let mut nsteps = 0;
                for _ in tree.parents(i) {
                    nsteps += 1;
                }
                assert_eq!(nsteps, 0);
            }

            assert_eq!(tree.parents(-1_i32).count(), 0);
            assert_eq!(tree.children(-1_i32).count(), 0);

            let roots = tree.roots_to_vec();
            for r in roots.iter() {
                let mut num_children = 0;
                for _ in tree.children(*r) {
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
        tables.build_index().unwrap();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        while let Some(tree) = tree_iter.next() {
            let mut num_roots = 0;
            for _ in tree.roots() {
                num_roots += 1;
            }
            assert_eq!(num_roots, 0);
        }
    }

    #[test]
    fn test_samples_iterator_error_when_not_tracking_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                match tree.samples(n) {
                    Err(_) => (),
                    _ => panic!("should not be Ok(_) or None"),
                }
            }
        }
    }

    #[test]
    fn test_num_tracked_samples() {
        let treeseq = treeseq_from_small_table_collection();
        assert_eq!(treeseq.num_samples(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(1).unwrap(), 1);
            assert_eq!(tree.num_tracked_samples(0).unwrap(), 2);
        }
    }

    #[should_panic]
    #[test]
    fn test_num_tracked_samples_not_tracking_sample_counts() {
        let treeseq = treeseq_from_small_table_collection();
        assert_eq!(treeseq.num_samples(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::NO_SAMPLE_COUNTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert_eq!(tree.num_tracked_samples(2).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(1).unwrap(), 0);
            assert_eq!(tree.num_tracked_samples(0).unwrap(), 0);
        }
    }

    #[test]
    fn test_iterate_samples() {
        let tables = make_small_table_collection();
        let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        if let Some(tree) = tree_iter.next() {
            assert!(!tree.flags().contains(TreeFlags::NO_SAMPLE_COUNTS));
            assert!(tree.flags().contains(TreeFlags::SAMPLE_LISTS));
            let mut s = vec![];

            if let Ok(iter) = tree.samples(0) {
                for i in iter {
                    s.push(i);
                }
            }
            assert_eq!(s.len(), 2);
            assert_eq!(
                s.len(),
                usize::try_from(tree.num_tracked_samples(0).unwrap()).unwrap()
            );
            assert_eq!(s[0], 1);
            assert_eq!(s[1], 2);

            for u in 1..3 {
                let mut s = vec![];
                if let Ok(iter) = tree.samples(u) {
                    for i in iter {
                        s.push(i);
                    }
                }
                assert_eq!(s.len(), 1);
                assert_eq!(s[0], u);
                assert_eq!(
                    s.len(),
                    usize::try_from(tree.num_tracked_samples(u).unwrap()).unwrap()
                );
            }
        } else {
            panic!("Expected a tree");
        }
    }

    #[test]
    fn test_iterate_samples_two_trees() {
        use super::ll_bindings::tsk_size_t;
        let treeseq = treeseq_from_small_table_collection_two_trees();
        assert_eq!(treeseq.num_trees(), 2);
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
        let expected_number_of_roots = vec![2, 1];
        let mut expected_root_ids = vec![
            vec![NodeId::from(0)],
            vec![NodeId::from(1), NodeId::from(0)],
        ];
        while let Some(tree) = tree_iter.next() {
            let mut num_roots = 0;
            let eroot_ids = expected_root_ids.pop().unwrap();
            for (i, r) in tree.roots().enumerate() {
                num_roots += 1;
                assert_eq!(r, eroot_ids[i]);
            }
            assert_eq!(
                expected_number_of_roots[(tree.current_tree - 1) as usize],
                num_roots
            );
            assert_eq!(tree.roots().count(), eroot_ids.len());
            let mut preoder_nodes = vec![];
            let mut postoder_nodes = vec![];
            for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
                let mut nsamples = 0;
                preoder_nodes.push(n);
                if let Ok(iter) = tree.samples(n) {
                    for _ in iter {
                        nsamples += 1;
                    }
                }
                assert!(nsamples > 0);
                assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
            }
            for n in tree.traverse_nodes(NodeTraversalOrder::Postorder) {
                let mut nsamples = 0;
                postoder_nodes.push(n);
                if let Ok(iter) = tree.samples(n) {
                    for _ in iter {
                        nsamples += 1;
                    }
                }
                assert!(nsamples > 0);
                assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
            }
            assert_eq!(preoder_nodes.len(), postoder_nodes.len());

            // Test our preorder against the tskit functions in 0.99.15
            {
                let mut nodes: Vec<NodeId> = vec![
                    NodeId::NULL;
                    unsafe { ll_bindings::tsk_tree_get_size_bound(tree.as_ptr()) }
                        as usize
                ];
                let mut num_nodes: tsk_size_t = 0;
                let ptr = std::ptr::addr_of_mut!(num_nodes);
                unsafe {
                    ll_bindings::tsk_tree_preorder(
                        tree.as_ptr(),
                        nodes.as_mut_ptr() as *mut tsk_id_t,
                        ptr,
                    );
                }
                assert_eq!(num_nodes as usize, preoder_nodes.len());
                for i in 0..num_nodes as usize {
                    assert_eq!(preoder_nodes[i], nodes[i]);
                }
            }
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

    #[test]
    fn test_dump_tables() {
        let tables = make_small_table_collection_two_trees();
        // Have to make b/c tables will no longer exist after making the treeseq
        let tables_copy = tables.deepcopy().unwrap();
        let ts = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
        let dumped = ts.dump_tables().unwrap();
        assert!(tables_copy.equals(&dumped, crate::TableEqualityOptions::default()));
    }

    #[test]
    fn test_reverse_tree_iteration() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        let mut starts_fwd = vec![];
        let mut stops_fwd = vec![];
        let mut starts_rev = vec![];
        let mut stops_rev = vec![];
        while let Some(tree) = tree_iter.next() {
            let interval = tree.interval();
            starts_fwd.push(interval.0);
            stops_fwd.push(interval.1);
        }
        assert_eq!(stops_fwd.len(), 2);
        assert_eq!(stops_fwd.len(), 2);

        // NOTE: we do NOT need to create a new iterator.
        while let Some(tree) = tree_iter.next_back() {
            let interval = tree.interval();
            starts_rev.push(interval.0);
            stops_rev.push(interval.1);
        }
        assert_eq!(starts_fwd.len(), starts_rev.len());
        assert_eq!(stops_fwd.len(), stops_rev.len());

        starts_rev.reverse();
        assert!(starts_fwd == starts_rev);
        stops_rev.reverse();
        assert!(stops_fwd == stops_rev);
    }

    // FIXME: remove later
    #[test]
    fn test_array_lifetime() {
        let treeseq = treeseq_from_small_table_collection_two_trees();
        let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
        if let Some(tree) = tree_iter.next() {
            let pa = tree.parent_array();
            let mut pc = vec![];
            for i in pa.iter() {
                pc.push(*i);
            }
            for (i, p) in pc.iter().enumerate() {
                assert_eq!(pa[i], *p);
            }
        } else {
            panic!("Expected a tree.");
        }
    }
}

#[cfg(test)]
mod test_treeeseq_send_sync {
    use crate::test_fixtures::treeseq_from_small_table_collection_two_trees;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn build_arc() {
        let t = treeseq_from_small_table_collection_two_trees();
        let a = Arc::new(t);
        let join_handle = thread::spawn(move || a.num_trees());
        let ntrees = join_handle.join().unwrap();
        assert_eq!(ntrees, 2);
    }
}
