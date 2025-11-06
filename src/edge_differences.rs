use crate::EdgeId;
use crate::NodeId;
use crate::Position;
use crate::TreeSequence;

/// Marker type for edge insertion.
pub struct Insertion {}

/// Marker type for edge removal.
pub struct Removal {}

mod private {
    pub trait EdgeDifferenceIteration {}

    impl EdgeDifferenceIteration for super::Insertion {}
    impl EdgeDifferenceIteration for super::Removal {}
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

/// Manages iteration over trees to obtain
/// edge differences.
pub struct EdgeDifferencesIterator<'ts> {
    edges_left: &'ts [Position],
    edges_right: &'ts [Position],
    edges_parent: &'ts [NodeId],
    edges_child: &'ts [NodeId],
    insertion_order: &'ts [EdgeId],
    removal_order: &'ts [EdgeId],
    left: f64,
    sequence_length: f64,
    insertion_index: usize,
    removal_index: usize,
}

impl<'ts> EdgeDifferencesIterator<'ts> {
    pub(crate) fn new(treeseq: &'ts TreeSequence) -> Self {
        Self {
            edges_left: treeseq.tables().edges().left_slice(),
            edges_right: treeseq.tables().edges().right_slice(),
            edges_parent: treeseq.tables().edges().parent_slice(),
            edges_child: treeseq.tables().edges().child_slice(),
            insertion_order: treeseq.edge_insertion_order(),
            removal_order: treeseq.edge_removal_order(),
            left: 0.,
            sequence_length: treeseq.tables().sequence_length().into(),
            insertion_index: 0,
            removal_index: 0,
        }
    }
}

#[derive(Clone)]
pub struct CurrentTreeEdgeDifferences<'ts> {
    edges_left: &'ts [Position],
    edges_right: &'ts [Position],
    edges_parent: &'ts [NodeId],
    edges_child: &'ts [NodeId],
    insertion_order: &'ts [EdgeId],
    removal_order: &'ts [EdgeId],
    removals: (usize, usize),
    insertions: (usize, usize),
    left: f64,
    right: f64,
}

#[repr(transparent)]
pub struct EdgeRemovalsIterator<'ts>(CurrentTreeEdgeDifferences<'ts>);

#[repr(transparent)]
pub struct EdgeInsertionsIterator<'ts>(CurrentTreeEdgeDifferences<'ts>);

impl<'ts> Iterator for EdgeRemovalsIterator<'ts> {
    type Item = EdgeDifference<Removal>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.removals.0 < self.0.removals.1 {
            let index = self.0.removals.0;
            self.0.removals.0 += 1;
            Some(Self::Item::new(
                self.0.edges_left[self.0.removal_order[index].as_usize()],
                self.0.edges_right[self.0.removal_order[index].as_usize()],
                self.0.edges_parent[self.0.removal_order[index].as_usize()],
                self.0.edges_child[self.0.removal_order[index].as_usize()],
            ))
        } else {
            None
        }
    }
}

impl<'ts> Iterator for EdgeInsertionsIterator<'ts> {
    type Item = EdgeDifference<Insertion>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.insertions.0 < self.0.insertions.1 {
            let index = self.0.insertions.0;
            self.0.insertions.0 += 1;
            Some(Self::Item::new(
                self.0.edges_left[self.0.insertion_order[index].as_usize()],
                self.0.edges_right[self.0.insertion_order[index].as_usize()],
                self.0.edges_parent[self.0.insertion_order[index].as_usize()],
                self.0.edges_child[self.0.insertion_order[index].as_usize()],
            ))
        } else {
            None
        }
    }
}

impl<'ts> CurrentTreeEdgeDifferences<'ts> {
    pub fn removals(&self) -> impl Iterator<Item = EdgeRemoval> + '_ {
        EdgeRemovalsIterator(self.clone())
    }

    pub fn insertions(&self) -> impl Iterator<Item = EdgeInsertion> + '_ {
        EdgeInsertionsIterator(self.clone())
    }

    pub fn interval(&self) -> (Position, Position) {
        (self.left.into(), self.right.into())
    }
}

fn update_right(
    right: f64,
    index: usize,
    position_slice: &[Position],
    diff_slice: &[EdgeId],
) -> f64 {
    if index < diff_slice.len() {
        let temp = position_slice[diff_slice[index].as_usize()];
        if temp < right {
            temp.into()
        } else {
            right
        }
    } else {
        right
    }
}

impl<'ts> Iterator for EdgeDifferencesIterator<'ts> {
    type Item = CurrentTreeEdgeDifferences<'ts>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.insertion_index < self.insertion_order.len() && self.left < self.sequence_length {
            let removals_start = self.removal_index;
            while self.removal_index < self.removal_order.len()
                && self.edges_right[self.removal_order[self.removal_index].as_usize()] == self.left
            {
                self.removal_index += 1;
            }
            let insertions_start = self.insertion_index;
            while self.insertion_index < self.insertion_order.len()
                && self.edges_left[self.insertion_order[self.insertion_index].as_usize()]
                    == self.left
            {
                self.insertion_index += 1;
            }
            let right = update_right(
                self.sequence_length,
                self.insertion_index,
                self.edges_left,
                self.insertion_order,
            );
            let right = update_right(
                right,
                self.removal_index,
                self.edges_right,
                self.removal_order,
            );
            let diffs = CurrentTreeEdgeDifferences {
                edges_left: self.edges_left,
                edges_right: self.edges_right,
                edges_parent: self.edges_parent,
                edges_child: self.edges_child,
                insertion_order: self.insertion_order,
                removal_order: self.removal_order,
                removals: (removals_start, self.removal_index),
                insertions: (insertions_start, self.insertion_index),
                left: self.left,
                right,
            };
            self.left = right;
            Some(diffs)
        } else {
            None
        }
    }
}
