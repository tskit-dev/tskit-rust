use clap::Parser;
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
        let _steps_to_root = tree.parents(s).count();
    }
}

fn preorder_traversal(tree: &tskit::Tree) {
    // Iterate over nodes.
    // For preorder traversal, this avoids allocation.
    // (But we collect the data for this example, which does allocate.)
    let nodes_from_iter = tree
        .traverse_nodes(tskit::NodeTraversalOrder::Preorder)
        .collect::<Vec<_>>();
    // Get a COPY of all nodes as a boxed slice
    let nodes_as_slice = tree.nodes(tskit::NodeTraversalOrder::Preorder).unwrap();
    assert_eq!(nodes_as_slice.len(), nodes_from_iter.len());
    nodes_from_iter
        .iter()
        .zip(nodes_as_slice.iter())
        .for_each(|(i, j)| assert_eq!(i, j));
}

#[derive(clap::Parser)]
struct Params {
    #[clap(short = 't', long = "treefile", value_parser, help = "Tree file name")]
    treefile: String,
}

fn main() {
    let params = Params::parse();

    let treeseq = tskit::TreeSequence::load(params.treefile).unwrap();

    let mut tree_iterator = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();

    while let Some(tree) = tree_iterator.next() {
        traverse_upwards(tree);
        traverse_upwards_with_iterator(tree);
        preorder_traversal(tree);
    }
}
