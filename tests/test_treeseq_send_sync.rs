#[path = "./test_fixtures.rs"]
mod test_fixtures;

use std::sync::Arc;
use std::thread;
use test_fixtures::treeseq_from_small_table_collection_two_trees;

#[test]
fn build_arc() {
    let t = treeseq_from_small_table_collection_two_trees();
    let a = Arc::new(t);
    let join_handle = thread::spawn(move || a.num_trees());
    let ntrees = join_handle.join().unwrap();
    assert_eq!(ntrees, 2);
}
