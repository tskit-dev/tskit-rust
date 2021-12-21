use clap::{Arg, Command};
use tskit::prelude::*;

// "Manual" traversal from samples to root
fn traverse_upwards(tree: &tskit::Tree) {
    for &s in tree.sample_nodes() {
        let mut u = s;
        while u != tskit::NodeId::NULL {
            u = tree.parent(u).unwrap();
        }
    }
}

// Iterate from each node up to its root.
fn traverse_upwards_with_iterator(tree: &tskit::Tree) {
    for &s in tree.sample_nodes() {
        // _steps_to_root counts the number of steps,
        // including the starting node s.
        for (_steps_to_root, _) in tree.parents(s).unwrap().enumerate() {}
    }
}

fn preorder_traversal(tree: &tskit::Tree) {
    for _ in tree.traverse_nodes(tskit::NodeTraversalOrder::Preorder) {}
}

fn main() {
    let matches = Command::new("tree_traversals")
        .arg(
            Arg::new("treefile")
                .short('t')
                .long("treefile")
                .help("Tree file name")
                .takes_value(true),
        )
        .get_matches();

    let treefile: String = matches.value_of_t_or_exit("treefile");

    let treeseq = tskit::TreeSequence::load(&treefile).unwrap();

    let mut tree_iterator = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();

    while let Some(tree) = tree_iterator.next() {
        traverse_upwards(tree);
        traverse_upwards_with_iterator(tree);
        preorder_traversal(tree);
    }
}

#[cfg(test)]
mod tests {

    use super::traverse_upwards;
    use tskit::prelude::*;

    pub fn make_small_table_collection_two_trees() -> tskit::TableCollection {
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

        // ANCHOR: init_table
        let mut tables = tskit::TableCollection::new(1000.).unwrap();
        // ANCHOR_END: init_table

        // ANCHOR: init_node_vec
        let mut node_ids = vec![];
        // ANCHOR_END: init_node_vec

        // ANCHOR: add_first_node
        match tables.add_node(
            0,
            tskit::Time::from(2.0),
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        ) {
            Ok(id) => {
                assert_eq!(id, 0);
                node_ids.push(id)
            }
            Err(e) => panic!("{}", e),
        };
        // ANCHOR_END: add_first_node

        // ANCHOR: add_second_node
        let id = tables
            .add_node(0, 1.0, -1, tskit::IndividualId::NULL)
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(id, NodeId::from(1));
        node_ids.push(id);
        // ANCHOR_END: add_second_node

        // ANCHOR: add_sample_nodes
        for i in 2..6 as tskit::bindings::tsk_id_t {
            match tables.add_node(
                tskit::TSK_NODE_IS_SAMPLE,
                0.0,
                tskit::PopulationId::NULL,
                tskit::IndividualId::NULL,
            ) {
                Ok(id) => {
                    assert_eq!(id, i);
                    let n = NodeId::from(i);
                    assert_eq!(id, n);
                    node_ids.push(id)
                }
                Err(e) => panic!("{}", e),
            }
        }
        // ANCHOR_END: add_sample_nodes

        // ANCHOR: add_edges
        tables
            .add_edge(tskit::Position::from(500.), 1000., node_ids[0], node_ids[1])
            .unwrap();
        tables.add_edge(0., 500., 0, 2).unwrap();
        tables.add_edge(0., 1000., 0, 3).unwrap();
        tables.add_edge(500., 1000., 1, 2).unwrap();
        tables.add_edge(0., 1000., 1, 4).unwrap();
        tables.add_edge(0., 1000., 1, 5).unwrap();
        // ANCHOR_END: add_edges

        // ANCHOR: sort_tables
        match tables.full_sort(tskit::TableSortOptions::default()) {
            Ok(rv) => {
                assert_eq!(rv, 0);
            }
            Err(e) => panic!("{}", e),
        }
        // ANCHOR_END: sort_tables

        // ANCHOR: index_tables
        match tables.build_index() {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
        // ANCHOR_END: index_tables

        // ANCHOR: check_integrity_lite
        match tables.check_integrity(tskit::TableIntegrityCheckFlags::default()) {
            Ok(rv) => {
                assert_eq!(rv, 0)
            }
            Err(e) => panic!("{}", e),
        }
        // ANCHOR_END: check_integrity_lite

        // ANCHOR: check_integrity_full
        match tables.check_integrity(tskit::TableIntegrityCheckFlags::all()) {
            Ok(num_trees) => {
                assert_eq!(num_trees, 2)
            }
            Err(e) => panic!("{}", e),
        }
        // ANCHOR_END: check_integrity_full

        tables
    }

    #[test]
    fn test_traverse_upwards() {
        let tables = make_small_table_collection_two_trees();
        let ts = tables
            .tree_sequence(tskit::TreeSequenceFlags::default())
            .unwrap();

        let mut tree_iterator = ts.tree_iterator(tskit::TreeFlags::default()).unwrap();

        while let Some(tree) = tree_iterator.next() {
            traverse_upwards(tree);
        }
    }
}
