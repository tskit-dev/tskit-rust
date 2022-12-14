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

fn main() {
    let args = Args::parse();

    assert!(args.stepsize > 0);

    let treeseq = tskit::TreeSequence::load(&args.treefile).unwrap();
    let num_trees: u64 = treeseq.num_trees().into();
    let flags = tskit::TreeFlags::SAMPLE_LISTS;
    let indexes = tskit::TreesIndex::new(&treeseq).unwrap();

    for i in (0..num_trees).step_by(args.stepsize as usize) {
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
        println!(
            "{} {:?} {:?}",
            i,
            duration.as_micros(),
            duration_lib.as_micros()
        );
    }
}
