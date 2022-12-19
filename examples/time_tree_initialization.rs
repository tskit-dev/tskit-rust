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
    let not_null = left.iter().filter(|x| !x.is_null()).count();
    let not_null_r = right.iter().filter(|x| !x.is_null()).count();
    assert_eq!(not_null, not_null_r);
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
        let mut tree_at = treeseq
            .tree_iterator_at_index(i.into(), &indexes, flags)
            .unwrap();
        let duration = now.elapsed();
        let now = Instant::now();
        let mut tree_at_lib = treeseq
            .tree_iterator_at_index_lib(i.into(), &indexes, flags)
            .unwrap();
        let duration_lib = now.elapsed();
        let now = Instant::now();
        let mut tree_at_jk = treeseq
            .tree_iterator_at_index_jk(i.into(), &indexes, flags)
            .unwrap();
        let duration_jk = now.elapsed();
        println!("indexes {:?} {:?}", i, duration.as_micros(),);
        println!("lib {:?} {:?}", i, duration_lib.as_micros(),);
        println!("jk {:?} {:?}", i, duration_jk.as_micros(),);

        let ttime_at: f64 = tree_at.total_branch_length(false).unwrap().into();
        let ttime_lib: f64 = tree_at_lib.total_branch_length(false).unwrap().into();
        let ttime_jk: f64 = tree_at_jk.total_branch_length(false).unwrap().into();

        // The "liberal" tolerance here is b/c our example
        // data has large ttl times is abs value.
        assert!(
            (ttime_jk - ttime_lib).abs() <= 1e-5,
            "jk vs lib: {} {}",
            ttime_jk,
            ttime_lib
        );
        assert!(
            (ttime_at - ttime_lib).abs() <= 1e-5,
            "at vs lib: {} {}",
            ttime_at,
            ttime_lib
        );

        assert_eq!(tree_at.interval(), tree_at_lib.interval());
        assert_eq!(tree_at_jk.interval(), tree_at_lib.interval());

        // compare(
        //     i,
        //     "parent",
        //     tree_at.parent_array(),
        //     tree_at_lib.parent_array(),
        // );
        compare(
            i,
            "parent",
            tree_at_jk.parent_array(),
            tree_at_lib.parent_array(),
        );

        let mut niterations = 0;
        assert_eq!(unsafe { (*tree_at_lib.as_ptr()).left_index }, unsafe {
            (*tree_at_jk.as_ptr()).left_index
        });
        assert_eq!(unsafe { (*tree_at_lib.as_ptr()).right_index }, unsafe {
            (*tree_at_jk.as_ptr()).right_index
        });
        assert_eq!(unsafe { (*tree_at_lib.as_ptr()).num_edges }, unsafe {
            (*tree_at.as_ptr()).num_edges
        });
        assert_eq!(unsafe { (*tree_at_lib.as_ptr()).num_edges }, unsafe {
            (*tree_at_jk.as_ptr()).num_edges
        });
        while let Some(tree_at_lib) = tree_at_lib.next() {
            let tree_at = tree_at.next().unwrap();
            let tree_at_jk = tree_at_jk.next().unwrap();

            assert_eq!(tree_at_lib.interval(), tree_at.interval());
            assert_eq!(tree_at_lib.interval(), tree_at_jk.interval());
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).index }, unsafe {
                (*tree_at.as_ptr()).index
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).index }, unsafe {
                (*tree_at_jk.as_ptr()).index
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).left_index }, unsafe {
                (*tree_at.as_ptr()).left_index
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).right_index }, unsafe {
                (*tree_at.as_ptr()).right_index
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).num_edges }, unsafe {
                (*tree_at_jk.as_ptr()).num_edges
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).left_index }, unsafe {
                (*tree_at_jk.as_ptr()).left_index
            });
            assert_eq!(unsafe { (*tree_at_lib.as_ptr()).right_index }, unsafe {
                (*tree_at_jk.as_ptr()).right_index
            });
            assert_eq!(tree_at_lib.interval(), tree_at_jk.interval());
            assert_eq!(
                unsafe { (*tree_at_lib.as_ptr()).index },
                unsafe { (*tree_at.as_ptr()).index },
                "tree index = {}",
                i
            );
            //let ttime_lib: f64 = tree_at_lib.total_branch_length(false).unwrap().into();
            //let ttime_jk: f64 = tree_at_jk.total_branch_length(false).unwrap().into();
            //assert!(
            //    (ttime_jk - ttime_lib).abs() <= 1e-6,
            //    "{} {}",
            //    ttime_lib,
            //    ttime_jk
            //);
            compare(
                i,
                "parent",
                tree_at.parent_array(),
                tree_at_lib.parent_array(),
            );
            compare(
                i,
                "parent jk",
                tree_at_jk.parent_array(),
                tree_at_lib.parent_array(),
            );
            niterations += 1;
        }
        println!("{}", niterations);

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
