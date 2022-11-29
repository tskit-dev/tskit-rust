use crate::NodeId;
use crate::Position;
use crate::TreeSequence;

use crate::bindings;

#[repr(transparent)]
struct LLEdgeDifferenceIterator(bindings::tsk_diff_iter_t);

impl std::ops::Deref for LLEdgeDifferenceIterator {
    type Target = bindings::tsk_diff_iter_t;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LLEdgeDifferenceIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for LLEdgeDifferenceIterator {
    fn drop(&mut self) {
        unsafe { bindings::tsk_diff_iter_free(&mut self.0) };
    }
}

impl LLEdgeDifferenceIterator {
    pub fn new_from_treeseq(treeseq: &TreeSequence, flags: bindings::tsk_flags_t) -> Option<Self> {
        let mut inner = std::mem::MaybeUninit::<bindings::tsk_diff_iter_t>::uninit();
        match unsafe { bindings::tsk_diff_iter_init(inner.as_mut_ptr(), treeseq.as_ptr(), flags) } {
            x if x < 0 => None,
            _ => Some(Self(unsafe { inner.assume_init() })),
        }
    }
}

/// Marker type for edge insertion.
pub struct Insertion {}

/// Marker type for edge removal.
pub struct Removal {}

mod private {
    pub trait EdgeDifferenceIteration {}

    impl EdgeDifferenceIteration for super::Insertion {}
    impl EdgeDifferenceIteration for super::Removal {}
}

struct LLEdgeList<T: private::EdgeDifferenceIteration> {
    inner: bindings::tsk_edge_list_t,
    marker: std::marker::PhantomData<T>,
}

macro_rules! build_lledgelist {
    ($name: ident, $generic: ty) => {
        type $name = LLEdgeList<$generic>;

        impl Default for $name {
            fn default() -> Self {
                Self {
                    inner: bindings::tsk_edge_list_t {
                        head: std::ptr::null_mut(),
                        tail: std::ptr::null_mut(),
                    },
                    marker: std::marker::PhantomData::<$generic> {},
                }
            }
        }
    };
}

build_lledgelist!(LLEdgeInsertionList, Insertion);
build_lledgelist!(LLEdgeRemovalList, Removal);

/// Concrete type implementing [`Iterator`] over [`EdgeInsertion`] or [`EdgeRemoval`].
/// Created by [`EdgeDifferencesIterator::edge_insertions`] or
/// [`EdgeDifferencesIterator::edge_removals`], respectively.
pub struct EdgeDifferences<'a, T: private::EdgeDifferenceIteration> {
    inner: &'a LLEdgeList<T>,
    current: *mut bindings::tsk_edge_list_node_t,
}

impl<'a, T: private::EdgeDifferenceIteration> EdgeDifferences<'a, T> {
    fn new(inner: &'a LLEdgeList<T>) -> Self {
        Self {
            inner,
            current: std::ptr::null_mut(),
        }
    }
}

/// An edge difference. Edge insertions and removals are differentiated by
/// marker types [`Insertion`] and [`Removal`], respectively.
#[derive(Debug, Copy, Clone)]
pub struct EdgeDifference<T: private::EdgeDifferenceIteration> {
    left: Position,
    right: Position,
    parent: NodeId,
    child: NodeId,
    marker: std::marker::PhantomData<T>,
}

impl<T: private::EdgeDifferenceIteration> EdgeDifference<T> {
    fn new<P: Into<Position>, N: Into<NodeId>>(left: P, right: P, parent: N, child: N) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
            parent: parent.into(),
            child: child.into(),
            marker: std::marker::PhantomData::<T> {},
        }
    }

    pub fn left(&self) -> Position {
        self.left
    }
    pub fn right(&self) -> Position {
        self.right
    }
    pub fn parent(&self) -> NodeId {
        self.parent
    }
    pub fn child(&self) -> NodeId {
        self.child
    }
}

impl<T> std::fmt::Display for EdgeDifference<T>
where
    T: private::EdgeDifferenceIteration,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "left: {}, right: {}, parent: {}, child: {}",
            self.left(),
            self.right(),
            self.parent(),
            self.child()
        )
    }
}

/// Type alias for [`EdgeDifference<Insertion>`]
pub type EdgeInsertion = EdgeDifference<Insertion>;
/// Type alias for [`EdgeDifference<Removal>`]
pub type EdgeRemoval = EdgeDifference<Removal>;

impl<'a, T> Iterator for EdgeDifferences<'a, T>
where
    T: private::EdgeDifferenceIteration,
{
    type Item = EdgeDifference<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            self.current = self.inner.inner.head;
        } else {
            self.current = unsafe { *self.current }.next;
        }
        if self.current.is_null() {
            None
        } else {
            let left = unsafe { (*self.current).edge.left };
            let right = unsafe { (*self.current).edge.right };
            let parent = unsafe { (*self.current).edge.parent };
            let child = unsafe { (*self.current).edge.child };
            Some(Self::Item::new(left, right, parent, child))
        }
    }
}

/// Manages iteration over trees to obtain
/// edge differences.
pub struct EdgeDifferencesIterator {
    inner: LLEdgeDifferenceIterator,
    insertion: LLEdgeInsertionList,
    removal: LLEdgeRemovalList,
    left: f64,
    right: f64,
    advanced: i32,
}

impl EdgeDifferencesIterator {
    // NOTE: will return None if tskit-c cannot
    // allocate memory for internal structures.
    pub(crate) fn new_from_treeseq(
        treeseq: &TreeSequence,
        flags: bindings::tsk_flags_t,
    ) -> Option<Self> {
        LLEdgeDifferenceIterator::new_from_treeseq(treeseq, flags).map(|inner| Self {
            inner,
            insertion: LLEdgeInsertionList::default(),
            removal: LLEdgeRemovalList::default(),
            left: f64::default(),
            right: f64::default(),
            advanced: 0,
        })
    }

    fn advance_tree(&mut self) {
        // SAFETY: our tree sequence is guaranteed
        // to be valid and own its tables.
        self.advanced = unsafe {
            bindings::tsk_diff_iter_next(
                &mut self.inner.0,
                &mut self.left,
                &mut self.right,
                &mut self.removal.inner,
                &mut self.insertion.inner,
            )
        };
    }

    pub fn left(&self) -> Position {
        self.left.into()
    }

    pub fn right(&self) -> Position {
        self.right.into()
    }

    pub fn interval(&self) -> (Position, Position) {
        (self.left(), self.right())
    }

    pub fn edge_removals(&self) -> impl Iterator<Item = EdgeRemoval> + '_ {
        EdgeDifferences::<Removal>::new(&self.removal)
    }

    pub fn edge_insertions(&self) -> impl Iterator<Item = EdgeInsertion> + '_ {
        EdgeDifferences::<Insertion>::new(&self.insertion)
    }
}

impl streaming_iterator::StreamingIterator for EdgeDifferencesIterator {
    type Item = EdgeDifferencesIterator;

    fn advance(&mut self) {
        self.advance_tree()
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.advanced > 0 {
            Some(self)
        } else {
            None
        }
    }
}
