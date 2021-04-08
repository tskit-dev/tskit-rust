use clap::{value_t_or_exit, App, Arg};
use streaming_iterator::StreamingIterator; // Required for tree iteration
use tskit;
use tskit::NodeIterator;

// "Manual" traversal from samples to root
fn traverse_upwards(tree: &tskit::Tree) -> () {
    let samples = tree.sample_list();

    for s in samples.iter() {
        let mut u = *s;
        while u != tskit::TSK_NULL {
            u = tree.parent(u).unwrap();
        }
    }
}

// Iterate from each node up to its root.
fn traverse_upwards_with_closure(tree: &tskit::Tree) -> () {
    let samples = tree.sample_list();

    for s in samples.iter() {
        let mut steps_to_root = 0;
        for _ in tree.path_to_root(*s).unwrap() {
            steps_to_root += 1;
        }
    }
}

fn preorder_traversal(tree: &tskit::Tree) {
    for c in tree.nodes(tskit::NodeTraversalOrder::Preorder) {
        println!("{}", c);
    }
}

fn main() {
    let matches = App::new("tree_traversals")
        .arg(
            Arg::with_name("treefile")
                .short("t")
                .long("treefile")
                .help("Tree file name")
                .takes_value(true),
        )
        .get_matches();

    let treefile = value_t_or_exit!(matches.value_of("treefile"), String);

    let treeseq = tskit::TreeSequence::load(&treefile).unwrap();

    let mut tree_iterator = treeseq.tree_iterator().unwrap();

    while let Some(tree) = tree_iterator.next() {
        traverse_upwards(&tree);
        traverse_upwards_with_closure(&tree);
        preorder_traversal(&tree);
    }
}
