#[path = "./test_fixtures.rs"]
mod test_fixtures;

use crate::NodeTraversalOrder;
use streaming_iterator::DoubleEndedStreamingIterator;
use streaming_iterator::StreamingIterator;
use test_fixtures::{
    make_small_table_collection, make_small_table_collection_two_trees,
    treeseq_from_small_table_collection, treeseq_from_small_table_collection_two_trees,
};

#[test]
fn test_create_treeseq_new_from_tables() {
    let tables = make_small_table_collection();
    let treeseq = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
    let samples = treeseq.sample_nodes();
    assert_eq!(samples.len(), 2);
    for i in 1..3 {
        assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));
    }
}

#[test]
fn test_create_treeseq_from_tables() {
    let tables = make_small_table_collection();
    let _treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
}

#[test]
fn test_iterate_tree_seq_with_one_tree() {
    let tables = make_small_table_collection();
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    let mut ntrees = 0;
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    while let Some(tree) = tree_iter.next() {
        ntrees += 1;
        assert_eq!(tree.current_tree, ntrees);
        let samples = tree.sample_nodes();
        assert_eq!(samples.len(), 2);
        for i in 1..3 {
            assert_eq!(samples[i - 1], NodeId::from(i as tsk_id_t));

            let mut nsteps = 0;
            for _ in tree.parents(samples[i - 1]).unwrap() {
                nsteps += 1;
            }
            assert_eq!(nsteps, 2);
        }
        let roots = tree.roots_to_vec();
        for r in roots.iter() {
            let mut num_children = 0;
            for _ in tree.children(*r).unwrap() {
                num_children += 1;
            }
            assert_eq!(num_children, 2);
        }
    }
    assert_eq!(ntrees, 1);
}

#[test]
fn test_iterate_no_roots() {
    let mut tables = TableCollection::new(100.).unwrap();
    tables.build_index().unwrap();
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    while let Some(tree) = tree_iter.next() {
        let mut num_roots = 0;
        for _ in tree.roots() {
            num_roots += 1;
        }
        assert_eq!(num_roots, 0);
    }
}

#[should_panic]
#[test]
fn test_samples_iterator_error_when_not_tracking_samples() {
    let tables = make_small_table_collection();
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    if let Some(tree) = tree_iter.next() {
        for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            for _ in tree.samples(n).unwrap() {}
        }
    }
}

#[test]
fn test_num_tracked_samples() {
    let treeseq = treeseq_from_small_table_collection();
    assert_eq!(treeseq.num_samples(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    if let Some(tree) = tree_iter.next() {
        assert_eq!(tree.num_tracked_samples(2.into()).unwrap(), 1);
        assert_eq!(tree.num_tracked_samples(1.into()).unwrap(), 1);
        assert_eq!(tree.num_tracked_samples(0.into()).unwrap(), 2);
    }
}

#[should_panic]
#[test]
fn test_num_tracked_samples_not_tracking_samples() {
    let treeseq = treeseq_from_small_table_collection();
    assert_eq!(treeseq.num_samples(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::NO_SAMPLE_COUNTS).unwrap();
    if let Some(tree) = tree_iter.next() {
        assert_eq!(tree.num_tracked_samples(2.into()).unwrap(), 0);
        assert_eq!(tree.num_tracked_samples(1.into()).unwrap(), 0);
        assert_eq!(tree.num_tracked_samples(0.into()).unwrap(), 0);
    }
}

#[test]
fn test_iterate_samples() {
    let tables = make_small_table_collection();
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

    let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
    if let Some(tree) = tree_iter.next() {
        assert!(!tree.flags().contains(TreeFlags::NO_SAMPLE_COUNTS));
        assert!(tree.flags().contains(TreeFlags::SAMPLE_LISTS));
        let mut s = vec![];
        for i in tree.samples(0.into()).unwrap() {
            s.push(i);
        }
        assert_eq!(s.len(), 2);
        assert_eq!(
            s.len(),
            usize::try_from(tree.num_tracked_samples(0.into()).unwrap()).unwrap()
        );
        assert_eq!(s[0], 1);
        assert_eq!(s[1], 2);

        for u in 1..3 {
            let mut s = vec![];
            for i in tree.samples(u.into()).unwrap() {
                s.push(i);
            }
            assert_eq!(s.len(), 1);
            assert_eq!(s[0], u);
            assert_eq!(
                s.len(),
                usize::try_from(tree.num_tracked_samples(u.into()).unwrap()).unwrap()
            );
        }
    } else {
        panic!("Expected a tree");
    }
}

#[test]
fn test_iterate_samples_two_trees() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    assert_eq!(treeseq.num_trees(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
    let expected_number_of_roots = vec![2, 1];
    let mut expected_root_ids = vec![
        vec![NodeId::from(0)],
        vec![NodeId::from(1), NodeId::from(0)],
    ];
    while let Some(tree) = tree_iter.next() {
        let mut num_roots = 0;
        let eroot_ids = expected_root_ids.pop().unwrap();
        for (i, r) in tree.roots().enumerate() {
            num_roots += 1;
            assert_eq!(r, eroot_ids[i]);
        }
        assert_eq!(
            expected_number_of_roots[(tree.current_tree - 1) as usize],
            num_roots
        );
        assert_eq!(tree.roots().count(), eroot_ids.len());
        let mut preoder_nodes = vec![];
        let mut postoder_nodes = vec![];
        for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            let mut nsamples = 0;
            preoder_nodes.push(n);
            for _ in tree.samples(n).unwrap() {
                nsamples += 1;
            }
            assert!(nsamples > 0);
            assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
        }
        for n in tree.traverse_nodes(NodeTraversalOrder::Postorder) {
            let mut nsamples = 0;
            postoder_nodes.push(n);
            for _ in tree.samples(n).unwrap() {
                nsamples += 1;
            }
            assert!(nsamples > 0);
            assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
        }
        assert_eq!(preoder_nodes.len(), postoder_nodes.len());

        // Test our preorder against the tskit functions in 0.99.15
        {
            let mut nodes: Vec<NodeId> = vec![
                NodeId::NULL;
                unsafe { ll_bindings::tsk_tree_get_size_bound(tree.as_ptr()) }
                    as usize
            ];
            let mut num_nodes: tsk_size_t = 0;
            let ptr = std::ptr::addr_of_mut!(num_nodes);
            unsafe {
                ll_bindings::tsk_tree_preorder(
                    tree.as_ptr(),
                    nodes.as_mut_ptr() as *mut tsk_id_t,
                    ptr,
                );
            }
            assert_eq!(num_nodes as usize, preoder_nodes.len());
            for i in 0..num_nodes as usize {
                assert_eq!(preoder_nodes[i], nodes[i]);
            }
        }
    }
}

#[test]
fn test_kc_distance_naive_test() {
    let ts1 = treeseq_from_small_table_collection();
    let ts2 = treeseq_from_small_table_collection();

    let kc = ts1.kc_distance(&ts2, 0.0).unwrap();
    assert!(kc.is_finite());
    assert!((kc - 0.).abs() < f64::EPSILON);
}

#[test]
fn test_dump_tables() {
    let tables = make_small_table_collection_two_trees();
    // Have to make b/c tables will no longer exist after making the treeseq
    let tables_copy = tables.deepcopy().unwrap();
    let ts = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    let dumped = ts.dump_tables().unwrap();
    assert!(tables_copy.equals(&dumped, crate::TableEqualityOptions::default()));
}

#[test]
fn test_reverse_tree_iteration() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    let mut starts_fwd = vec![];
    let mut stops_fwd = vec![];
    let mut starts_rev = vec![];
    let mut stops_rev = vec![];
    while let Some(tree) = tree_iter.next() {
        let interval = tree.interval();
        starts_fwd.push(interval.0);
        stops_fwd.push(interval.1);
    }
    assert_eq!(stops_fwd.len(), 2);
    assert_eq!(stops_fwd.len(), 2);

    // NOTE: we do NOT need to create a new iterator.
    while let Some(tree) = tree_iter.next_back() {
        let interval = tree.interval();
        starts_rev.push(interval.0);
        stops_rev.push(interval.1);
    }
    assert_eq!(starts_fwd.len(), starts_rev.len());
    assert_eq!(stops_fwd.len(), stops_rev.len());

    starts_rev.reverse();
    assert!(starts_fwd == starts_rev);
    stops_rev.reverse();
    assert!(stops_fwd == stops_rev);
}

// FIXME: remove later
#[test]
fn test_array_lifetime() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    if let Some(tree) = tree_iter.next() {
        let pa = tree.parent_array();
        let mut pc = vec![];
        for i in pa.iter() {
            pc.push(*i);
        }
        for (i, p) in pc.iter().enumerate() {
            assert_eq!(pa[i], *p);
        }
    } else {
        panic!("Expected a tree.");
    }
}
