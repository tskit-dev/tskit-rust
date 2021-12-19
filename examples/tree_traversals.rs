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

        let mut tables = tskit::TableCollection::new(1000.).unwrap();
        tables
            .add_node(0, 2.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL)
            .unwrap();
        tables
            .add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL)
            .unwrap();
        tables
            .add_node(
                tskit::TSK_NODE_IS_SAMPLE,
                0.0,
                tskit::PopulationId::NULL,
                tskit::IndividualId::NULL,
            )
            .unwrap();
        tables
            .add_node(
                tskit::TSK_NODE_IS_SAMPLE,
                0.0,
                tskit::PopulationId::NULL,
                tskit::IndividualId::NULL,
            )
            .unwrap();
        tables
            .add_node(
                tskit::TSK_NODE_IS_SAMPLE,
                0.0,
                tskit::PopulationId::NULL,
                tskit::IndividualId::NULL,
            )
            .unwrap();
        tables
            .add_node(
                tskit::TSK_NODE_IS_SAMPLE,
                0.0,
                tskit::PopulationId::NULL,
                tskit::IndividualId::NULL,
            )
            .unwrap();
        tables.add_edge(500., 1000., 0, 1).unwrap();
        tables.add_edge(0., 500., 0, 2).unwrap();
        tables.add_edge(0., 1000., 0, 3).unwrap();
        tables.add_edge(500., 1000., 1, 2).unwrap();
        tables.add_edge(0., 1000., 1, 4).unwrap();
        tables.add_edge(0., 1000., 1, 5).unwrap();
        tables
            .full_sort(tskit::TableSortOptions::default())
            .unwrap();
        tables.build_index().unwrap();
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
