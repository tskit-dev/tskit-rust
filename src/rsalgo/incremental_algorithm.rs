use crate::NodeId;
use crate::Position;
use crate::TreeSequence;

pub(crate) trait IncrementalAlgorithm {
    fn process_output_edge(&mut self, parent: NodeId, child: NodeId);
    fn process_input_edge(&mut self, parent: NodeId, child: NodeId);
    fn process_interval(&mut self, left: Position, right: Position);
}

#[derive(Default)]
pub(crate) struct IncrementalAlgorithmOptions {
    pub process_remaining_edges: bool,
}

fn update_right<P, E>(right: f64, index: usize, position_slice: &P, diff_slice: &E) -> f64
where
    P: crate::TableColumn<crate::EdgeId, Position>,
    E: crate::TableColumn<crate::EdgeId, crate::EdgeId>,
{
    if index < diff_slice.len() {
        let temp = position_slice[diff_slice[index]];
        if temp < right {
            temp.into()
        } else {
            right
        }
    } else {
        right
    }
}

#[allow(dead_code)]
pub(crate) fn incremental_algorithm<I: IncrementalAlgorithm>(
    ts: &TreeSequence,
    options: IncrementalAlgorithmOptions,
    ia: &mut I,
) {
    let mut left = 0.0;
    let edges_in = ts.edge_insertion_order_column();
    let edges_out = ts.edge_removal_order_column();
    let edges_left = ts.tables().edges().left_column();
    let edges_right = ts.tables().edges().right_column();
    let edges_parent = ts.tables().edges().parent_column();
    let edges_child = ts.tables().edges().child_column();
    let num_edges = ts.edges().num_rows().as_usize();
    let mut i = 0_usize;
    let mut j = 0_usize;
    let mut right = 0.0;
    while i < num_edges && left < ts.tables().sequence_length() {
        left = right;
        while j < num_edges && edges_right[edges_out[j]] == left {
            let parent = edges_parent[edges_out[j]];
            let child = edges_child[edges_out[j]];
            ia.process_output_edge(parent, child);
            j += 1;
        }
        while i < num_edges && edges_left[edges_in[i]] == left {
            let parent = edges_parent[edges_in[i]];
            let child = edges_child[edges_in[i]];
            ia.process_input_edge(parent, child);
            i += 1;
        }
        right = update_right(
            ts.tables().sequence_length().into(),
            i,
            &edges_left,
            &edges_in,
        );
        right = update_right(right, j, &edges_right, &edges_out);
        ia.process_interval(left.into(), right.into());
    }
    if options.process_remaining_edges {
        assert_eq!(right, ts.tables().sequence_length());
        while j < num_edges && edges_right[edges_out[j]] == right {
            let parent = edges_parent[edges_out[j]];
            let child = edges_child[edges_out[j]];
            ia.process_output_edge(parent, child);
            j += 1;
        }
        let right = update_right(
            ts.tables().sequence_length().into(),
            i,
            &edges_left,
            &edges_in,
        );
        let right = update_right(right, j, &edges_right, &edges_out);
        ia.process_interval(left.into(), right.into());
    }
}

#[cfg(test)]
// NOTE: copied from tests/test_trees.rs
pub fn make_small_table_collection_two_trees() -> crate::TableCollection {
    // The two trees are:
    //  0
    // +++
    // | |  1
    // | | +++
    // 2 3 4 5

    //     0
    //   +-+-+
    //   1   |
    // +-+-+ |
    // 2 4 5 3

    let mut tables = crate::TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 2.0, crate::PopulationId::NULL, crate::IndividualId::NULL)
        .unwrap();
    tables
        .add_node(0, 1.0, crate::PopulationId::NULL, crate::IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            crate::NodeFlags::new_sample(),
            0.0,
            crate::PopulationId::NULL,
            crate::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            crate::NodeFlags::new_sample(),
            0.0,
            crate::PopulationId::NULL,
            crate::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            crate::NodeFlags::new_sample(),
            0.0,
            crate::PopulationId::NULL,
            crate::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            crate::NodeFlags::new_sample(),
            0.0,
            crate::PopulationId::NULL,
            crate::IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(500., 1000., 0, 1).unwrap();
    tables.add_edge(0., 500., 0, 2).unwrap();
    tables.add_edge(0., 1000., 0, 3).unwrap();
    tables.add_edge(500., 1000., 1, 2).unwrap();
    tables.add_edge(0., 1000., 1, 4).unwrap();
    tables.add_edge(0., 1000., 1, 5).unwrap();
    tables
        .full_sort(crate::TableSortOptions::default())
        .unwrap();
    tables.build_index().unwrap();
    tables
}

#[cfg(test)]
// NOTE: copied from tests/test_trees.rs
pub fn treeseq_from_small_table_collection_two_trees() -> TreeSequence {
    let tables = make_small_table_collection_two_trees();
    tables
        .tree_sequence(crate::TreeSequenceFlags::default())
        .unwrap()
}

#[cfg(test)]
#[derive(Default)]
struct EdgeCounter {
    input: usize,
    output: usize,
}

#[cfg(test)]
impl IncrementalAlgorithm for EdgeCounter {
    fn process_input_edge(&mut self, _parent: NodeId, _child: NodeId) {
        self.input += 1;
    }

    fn process_output_edge(&mut self, _parent: NodeId, _child: NodeId) {
        self.output += 1;
    }

    fn process_interval(&mut self, left: Position, right: Position) {
        assert_ne!(left, right)
    }
}

#[test]
fn test_default_options() {
    let o = IncrementalAlgorithmOptions::default();
    assert!(!o.process_remaining_edges);
}

#[test]
fn test_number_processed_edges() {
    let ts = treeseq_from_small_table_collection_two_trees();
    let mut ec = EdgeCounter::default();
    incremental_algorithm(&ts, IncrementalAlgorithmOptions::default(), &mut ec);
    assert_eq!(ec.input, ts.edges().num_rows().as_usize());
    assert_eq!(
        ec.output,
        ts.edges()
            .iter()
            .filter(|e| e.right() != ts.tables().sequence_length())
            .count()
    );
}

#[test]
fn test_number_processed_edges_including_final_outgoing() {
    let ts = treeseq_from_small_table_collection_two_trees();
    let mut ec = EdgeCounter::default();
    incremental_algorithm(
        &ts,
        IncrementalAlgorithmOptions {
            process_remaining_edges: true,
        },
        &mut ec,
    );
    assert_eq!(ec.input, ts.edges().num_rows().as_usize());
    assert_eq!(ec.output, ts.edges().num_rows().as_usize());
}
