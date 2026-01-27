use std::sync::Arc;
use std::thread;

use crate::DoubleEndedStreamingIterator;
use crate::StreamingIterator;

use tskit::prelude::*;
use tskit::NodeFlags;
use tskit::NodeTraversalOrder;
use tskit::SimplificationOptions;
use tskit::TableCollection;
use tskit::TableEqualityOptions;
use tskit::TableSortOptions;
use tskit::TreeFlags;
use tskit::TreeSequence;
use tskit::TreeSequenceFlags;

pub fn make_small_table_collection() -> TableCollection {
    let mut tables = TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(0., 1000., 0, 1).unwrap();
    tables.add_edge(0., 1000., 0, 2).unwrap();
    tables.build_index().unwrap();
    tables
}

pub fn treeseq_from_small_table_collection() -> TreeSequence {
    let tables = make_small_table_collection();
    tables.tree_sequence(TreeSequenceFlags::default()).unwrap()
}

pub fn make_small_table_collection_two_trees() -> TableCollection {
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

    let mut tables = TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 2.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(500., 1000., 0, 1).unwrap();
    tables.add_edge(0., 500., 0, 2).unwrap();
    tables.add_edge(0., 1000., 0, 3).unwrap();
    tables.add_edge(500., 1000., 1, 2).unwrap();
    tables.add_edge(0., 1000., 1, 4).unwrap();
    tables.add_edge(0., 1000., 1, 5).unwrap();
    tables.full_sort(TableSortOptions::default()).unwrap();
    tables.build_index().unwrap();
    tables
}

pub fn treeseq_from_small_table_collection_two_trees() -> TreeSequence {
    let tables = make_small_table_collection_two_trees();
    tables.tree_sequence(TreeSequenceFlags::default()).unwrap()
}
#[test]
fn test_create_treeseq_new_from_tables() {
    let tables = make_small_table_collection();
    let treeseq = TreeSequence::new(tables, TreeSequenceFlags::default()).unwrap();
    let samples = treeseq.sample_nodes();
    assert_eq!(samples.len(), 2);
    for i in 1_i32..3 {
        assert_eq!(samples[(i - 1) as usize], NodeId::from(i));
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
        let samples = tree.sample_nodes();
        assert_eq!(samples.len(), 2);
        for i in 1_i32..3 {
            assert_eq!(samples[(i - 1) as usize], NodeId::from(i));

            let mut nsteps = 0;
            for _ in tree.parents(samples[(i - 1) as usize]) {
                nsteps += 1;
            }
            assert_eq!(nsteps, 2);
        }

        // These nodes are all out of range
        for i in 100..110 {
            let mut nsteps = 0;
            for _ in tree.parents(i) {
                nsteps += 1;
            }
            assert_eq!(nsteps, 0);
        }

        assert_eq!(tree.parents(-1_i32).count(), 0);
        assert_eq!(tree.children(-1_i32).count(), 0);

        let roots = tree.roots_to_vec();
        for r in roots.iter() {
            let mut num_children = 0;
            for _ in tree.children(*r) {
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

#[test]
fn test_samples_iterator_error_when_not_tracking_samples() {
    let tables = make_small_table_collection();
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();

    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    if let Some(tree) = tree_iter.next() {
        for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            match tree.samples(n) {
                Err(_) => (),
                _ => panic!("should not be Ok(_) or None"),
            }
        }
    }
}

#[test]
fn test_num_tracked_samples() {
    let treeseq = treeseq_from_small_table_collection();
    assert_eq!(treeseq.num_samples(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    if let Some(tree) = tree_iter.next() {
        assert_eq!(tree.num_tracked_samples(2).unwrap(), 1);
        assert_eq!(tree.num_tracked_samples(1).unwrap(), 1);
        assert_eq!(tree.num_tracked_samples(0).unwrap(), 2);
    }
}

#[should_panic]
#[test]
fn test_num_tracked_samples_not_tracking_sample_counts() {
    let treeseq = treeseq_from_small_table_collection();
    assert_eq!(treeseq.num_samples(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::NO_SAMPLE_COUNTS).unwrap();
    if let Some(tree) = tree_iter.next() {
        assert_eq!(tree.num_tracked_samples(2).unwrap(), 0);
        assert_eq!(tree.num_tracked_samples(1).unwrap(), 0);
        assert_eq!(tree.num_tracked_samples(0).unwrap(), 0);
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

        if let Ok(iter) = tree.samples(0) {
            for i in iter {
                s.push(i);
            }
        }
        assert_eq!(s.len(), 2);
        assert_eq!(
            s.len(),
            usize::try_from(tree.num_tracked_samples(0).unwrap()).unwrap()
        );
        assert_eq!(s[0], 1);
        assert_eq!(s[1], 2);

        for u in 1..3 {
            let mut s = vec![];
            if let Ok(iter) = tree.samples(u) {
                for i in iter {
                    s.push(i);
                }
            }
            assert_eq!(s.len(), 1);
            assert_eq!(s[0], u);
            assert_eq!(
                s.len(),
                usize::try_from(tree.num_tracked_samples(u).unwrap()).unwrap()
            );
        }
    } else {
        panic!("Expected a tree");
    }
}

#[cfg(feature = "bindings")]
#[test]
fn test_iterate_samples_two_trees() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    assert_eq!(treeseq.num_trees(), 2);
    let mut tree_iter = treeseq.tree_iterator(TreeFlags::SAMPLE_LISTS).unwrap();
    let expected_number_of_roots = [2, 1];
    let mut expected_root_ids = vec![
        vec![NodeId::from(0)],
        vec![NodeId::from(1), NodeId::from(0)],
    ];
    let mut current_tree: usize = 0;
    while let Some(tree) = tree_iter.next() {
        let mut num_roots = 0;
        let eroot_ids = expected_root_ids.pop().unwrap();
        for (i, r) in tree.roots().enumerate() {
            num_roots += 1;
            assert_eq!(r, eroot_ids[i]);
        }
        assert_eq!(expected_number_of_roots[current_tree], num_roots);
        assert_eq!(tree.roots().count(), eroot_ids.len());
        let mut preoder_nodes = vec![];
        let mut postoder_nodes = vec![];
        for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            let mut nsamples = 0;
            preoder_nodes.push(n);
            if let Ok(iter) = tree.samples(n) {
                for _ in iter {
                    nsamples += 1;
                }
            }
            assert!(nsamples > 0);
            assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
        }
        for n in tree.traverse_nodes(NodeTraversalOrder::Postorder) {
            let mut nsamples = 0;
            postoder_nodes.push(n);
            if let Ok(iter) = tree.samples(n) {
                for _ in iter {
                    nsamples += 1;
                }
            }
            assert!(nsamples > 0);
            assert_eq!(nsamples, tree.num_tracked_samples(n).unwrap());
        }
        assert_eq!(preoder_nodes.len(), postoder_nodes.len());

        // Test our preorder against the tskit functions in 0.99.15
        {
            let mut nodes: Vec<NodeId> = vec![
                NodeId::NULL;
                unsafe { tskit::bindings::tsk_tree_get_size_bound(tree.as_ll_ref()) }
                    as usize
            ];
            let mut num_nodes: tskit::bindings::tsk_size_t = 0;
            let ptr = std::ptr::addr_of_mut!(num_nodes);
            unsafe {
                tskit::bindings::tsk_tree_preorder(
                    tree.as_ll_ref(),
                    nodes.as_mut_ptr() as *mut tskit::bindings::tsk_id_t,
                    ptr,
                );
            }
            assert_eq!(num_nodes as usize, preoder_nodes.len());
            for i in 0..num_nodes as usize {
                assert_eq!(preoder_nodes[i], nodes[i]);
            }
        }
        current_tree += 1;
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
    assert!(tables_copy.equals(&dumped, TableEqualityOptions::default()));
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

#[test]
fn test_tree_iteration_at_position() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    let mut tree_iter = treeseq
        .tree_iterator_at_position(TreeFlags::default(), 502.)
        .unwrap();
    let interval = tree_iter.interval();
    assert!(502. >= interval.0 && 502. < interval.1);
    assert!(tree_iter.next().is_none())
}

#[test]
fn test_tree_iteration_at_invalid_position() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    assert!(treeseq
        .tree_iterator_at_position(TreeFlags::default(), -1.)
        .is_err());
    assert!(treeseq
        .tree_iterator_at_position(TreeFlags::default(), 1001.)
        .is_err());
}

#[test]
fn test_tree_iteration_at_index() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    let mut tree_iter = treeseq
        .tree_iterator_at_index(TreeFlags::default(), 1)
        .unwrap();
    let interval = tree_iter.interval();
    assert!(502. >= interval.0 && 502. < interval.1);
    assert!(tree_iter.next().is_none())
}

#[test]
fn test_tree_iteration_at_invalid_index() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    assert!(treeseq
        .tree_iterator_at_index(TreeFlags::default(), -1)
        .is_err());
    assert!(treeseq
        .tree_iterator_at_index(TreeFlags::default(), 2)
        .is_err());
}

#[test]
fn build_arc() {
    let t = treeseq_from_small_table_collection_two_trees();
    let a = Arc::new(t);
    let join_handle = thread::spawn(move || a.num_trees());
    let ntrees = join_handle.join().unwrap();
    assert_eq!(ntrees, 2);
}

#[test]
fn test_simplify_tables() {
    let mut tables = make_small_table_collection_two_trees();
    let mut samples: Vec<NodeId> = vec![];
    for (i, row) in tables.nodes_iter().enumerate() {
        if row.flags.contains(NodeFlags::IS_SAMPLE) {
            samples.push((i as i32).into());
        }
    }
    let idmap_option = tables
        .simplify(&samples, SimplificationOptions::default(), true)
        .unwrap();
    assert!(idmap_option.is_some());
    let idmap = idmap_option.unwrap();
    for i in samples.iter() {
        assert_ne!(idmap[i.as_usize()], NodeId::NULL);
        assert!(!idmap[i.as_usize()].is_null());
    }
}

#[test]
fn test_simplify_treeseq() {
    let ts = treeseq_from_small_table_collection_two_trees();
    let samples = ts.sample_nodes();
    let (_, idmap_option) = ts
        .simplify(samples, SimplificationOptions::default(), true)
        .unwrap();
    assert!(idmap_option.is_some());
    let idmap = idmap_option.unwrap();
    for &i in samples {
        assert_ne!(idmap[usize::try_from(i).unwrap()], NodeId::NULL);
    }
}


#[test]
fn test_need_mutation_parents() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let n0 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let n1 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let n2 = tables.add_node(0, 2.0, -1, -1).unwrap();
    let _ = tables.add_edge(0., 100., n1, n0).unwrap();
    let _ = tables.add_edge(0., 100., n2, n1).unwrap();
    tables.build_index().unwrap();
    assert!(tables.deepcopy().unwrap().tree_sequence(0).is_ok());
    let s0 = tables.add_site(10.0, None).unwrap();
    let _ = tables.add_mutation(s0, n1, -1, 1.0, None).unwrap();
    let _ = tables.add_mutation(s0, n0, -1, 0.0, None).unwrap();
    tables.build_index().unwrap();
    // Error code 511 is the "bad mutation parents error".
    // (Yes, testing against internal constants is ugly,
    // but we want to make sure we hit this case.)
    match tables.deepcopy().unwrap().tree_sequence(0) {
        Ok(_) => panic!("expected Error"),
        Err(e) => assert!(matches!(e, tskit::TskitError::ErrorCode { code: -511 })),
    }
    tables
        .compute_mutation_parents(tskit::MutationParentsFlags::default())
        .unwrap();
    assert!(tables.tree_sequence(0).is_ok());
}
