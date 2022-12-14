use std::time::Instant;

use clap::Parser;
use tskit::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    treefile: String,
    #[clap(short, long, value_parser, default_value = "10")]
    stepsize: u64,
}

fn compare(tree: u64, name: &str, left: &[NodeId], right: &[NodeId]) {
    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        if *l != *r {
            panic!(
                "tree {}, array: {}, index {}, left {}, right {}",
                tree, name, i, *l, *r
            );
        }
    }
}

fn main() {
    let args = Args::parse();

    assert!(args.stepsize > 0);

    let treeseq = tskit::TreeSequence::load(&args.treefile).unwrap();
    let num_trees: u64 = treeseq.num_trees().into();
    let flags = tskit::TreeFlags::SAMPLE_LISTS;
    let indexes = tskit::TreesIndex::new(&treeseq).unwrap();

    println!("method index time");
    for i in (0..num_trees).step_by(args.stepsize as usize) {
        assert!(i < num_trees);
        let now = Instant::now();
        let tree_at = treeseq
            .tree_iterator_at_index(i.into(), &indexes, flags)
            .unwrap();
        let duration = now.elapsed();
        let now = Instant::now();
        let tree_at_lib = treeseq
            .tree_iterator_at_index_lib(i.into(), &indexes, flags)
            .unwrap();
        let duration_lib = now.elapsed();
        let now = Instant::now();
        let tree_at_jk = treeseq
            .tree_iterator_at_index_jk(i.into(), &indexes, flags)
            .unwrap();
        let duration_jk = now.elapsed();
        println!("indexes {:?} {:?}", i, duration.as_micros(),);
        println!("lib {:?} {:?}", i, duration_lib.as_micros(),);
        println!("jk {:?} {:?}", i, duration_jk.as_micros(),);

        let ttime_at: f64 = tree_at.total_branch_length(false).unwrap().into();
        let ttime_lib: f64 = tree_at_lib.total_branch_length(false).unwrap().into();
        let ttime_jk: f64 = tree_at_jk.total_branch_length(false).unwrap().into();

        assert!((ttime_at - ttime_lib).abs() <= 1e-8);
        assert!((ttime_jk - ttime_lib).abs() <= 1e-8);

        compare(
            i,
            "parent",
            tree_at.parent_array(),
            tree_at_lib.parent_array(),
        );
        compare(
            i,
            "parent",
            tree_at_jk.parent_array(),
            tree_at_lib.parent_array(),
        );

        // The following may not be valid:
        // the different remove/insert ops
        // could change orders of stuff in sub-trees?

        //compare(
        //    i,
        //    "left_child",
        //    tree_at.left_child_array(),
        //    tree_at_lib.left_child_array(),
        //);
        //compare(
        //    i,
        //    "right_child",
        //    tree_at.right_child_array(),
        //    tree_at_lib.right_child_array(),
        //);
        //compare(
        //    i,
        //    "left_sib",
        //    tree_at.left_sib_array(),
        //    tree_at_lib.left_sib_array(),
        //);
        //compare(
        //    i,
        //    "right_sib",
        //    tree_at.right_sib_array(),
        //    tree_at_lib.right_sib_array(),
        //);
    }
}
