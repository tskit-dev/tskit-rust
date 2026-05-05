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

#[cfg(feature = "bindings")]
fn compare_preorder_to_c_api(tree: &tskit::Tree, node: NodeId, expected: &[NodeId]) {
    let mut nodes: Vec<NodeId> = vec![
        NodeId::NULL;
        unsafe { tskit::bindings::tsk_tree_get_size_bound(tree.as_ll_ref()) }
            as usize
    ];
    let mut num_nodes: tskit::bindings::tsk_size_t = 0;
    let ptr = std::ptr::addr_of_mut!(num_nodes);
    unsafe {
        tskit::bindings::tsk_tree_preorder_from(
            tree.as_ll_ref(),
            if node == tree.virtual_root() {
                -1
            } else {
                node.into()
            },
            nodes.as_mut_ptr() as *mut tskit::bindings::tsk_id_t,
            ptr,
        );
    }
    assert_eq!(num_nodes as usize, expected.len());
    assert_eq!(expected, &nodes[0..num_nodes as usize]);
}

#[cfg(not(feature = "bindings"))]
fn compare_preorder_to_c_api(_tree: &tskit::Tree, _node: NodeId, _expected: &[NodeId]) {}

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
    let mut tree_iter = treeseq
        .tree_iterator(TreeFlags::default().sample_lists())
        .unwrap();
    while let Some(tree) = tree_iter.next() {
        ntrees += 1;
        let samples = tree.samples_iter().unwrap().collect::<Vec<_>>();
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

        for r in tree.roots() {
            let mut num_children = 0;
            for _ in tree.children(r) {
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
        assert!(s
            .iter()
            .all(|&n| treeseq.nodes().flags(n).unwrap().is_sample()));
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
        let mut preorder_nodes = vec![];
        let mut postoder_nodes = vec![];
        for n in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            let mut nsamples = 0;
            preorder_nodes.push(n);
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
        assert_eq!(preorder_nodes.len(), postoder_nodes.len());

        let mut postorder_from_roots = vec![];
        for root in tree.roots() {
            for node in tree
                .traverse_nodes_from_root(root, NodeTraversalOrder::Postorder)
                .unwrap()
            {
                postorder_from_roots.push(node);
            }
        }
        assert_eq!(postorder_from_roots, postoder_nodes);

        compare_preorder_to_c_api(tree, tree.virtual_root(), &preorder_nodes);

        // For each root, traverse its subtree with a preorder
        // traversal, collecting outputs as we go.
        // Then, compare to what the C API preorder fn outputs
        let mut nodes_from_roots = vec![];
        for root in tree.roots() {
            for node in tree
                .traverse_nodes_from_root(root, NodeTraversalOrder::Preorder)
                .unwrap()
            {
                nodes_from_roots.push(node);
            }
        }
        for &node in &nodes_from_roots {
            assert!(preorder_nodes.contains(&node));
        }
        // This assert checks that we get the same order as the
        // tskit-c preorder fn.  We need to take a slice of the
        // vec where we store the tskit output b/c its allocation
        // may be larger than the number of nodes in the tree.
        assert_eq!(nodes_from_roots, preorder_nodes);
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
    for (i, row) in tables.node_iter().enumerate() {
        if row.flags().contains(NodeFlags::IS_SAMPLE) {
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

#[test]
fn test_iterate_mutations_at_site() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let n0 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let n1 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let n2 = tables.add_node(0, 2.0, -1, -1).unwrap();
    let _ = tables.add_edge(0., 100., n1, n0).unwrap();
    let _ = tables.add_edge(0., 100., n2, n1).unwrap();
    let s0 = tables.add_site(10.0, None).unwrap();
    let m0 = tables.add_mutation(s0, n1, -1, 1.0, None).unwrap();
    let m1 = tables.add_mutation(s0, n0, -1, 0.0, None).unwrap();
    let s1 = tables.add_site(11.0, None).unwrap();
    let _ = tables.add_mutation(s1, n0, -1, 0.0, None).unwrap();
    tables.build_index().unwrap();
    tables
        .compute_mutation_parents(tskit::MutationParentsFlags::default())
        .unwrap();
    let ts = tables.tree_sequence(0).unwrap();
    let muts_at_site = ts
        .site(s0)
        .unwrap()
        .mutation_iter()
        .map(|m| m.id())
        .collect::<Vec<_>>();
    assert_eq!(muts_at_site.len(), 2);
    assert!(muts_at_site.contains(&m0));
    assert!(muts_at_site.contains(&m1));

    for site in ts.site_iter() {
        let muts = site.mutation_iter().collect::<Vec<_>>();
        let rmuts = site.mutation_iter().rev().collect::<Vec<_>>();
        for (i, j) in muts.iter().rev().zip(rmuts.iter()) {
            assert_eq!(i, j)
        }
    }
}
#[test]
fn test_site_mutation_iterators_nth() {
    let mut tables = tskit::TableCollection::new(100.0).unwrap();
    let n0 = tables
        .add_node(tskit::NodeFlags::IS_SAMPLE, 0.0, -1, -1)
        .unwrap();
    let s0 = tables.add_site(50., None).unwrap();
    let m0 = tables.add_mutation(s0, n0, -1, 20.0, None).unwrap();
    let m1 = tables.add_mutation(s0, n0, -1, 10.0, None).unwrap();
    let m2 = tables.add_mutation(s0, n0, -1, 5.0, None).unwrap();
    let m3 = tables.add_mutation(s0, n0, -1, 2.0, None).unwrap();

    tables.full_sort(0).unwrap();
    tables.build_index().unwrap();
    tables
        .compute_mutation_parents(tskit::MutationParentsFlags::default())
        .unwrap();
    let ts = tables.deepcopy().unwrap().tree_sequence(0).unwrap();

    for (index, mutid) in [m0, m1, m2, m3].iter().enumerate() {
        assert_eq!(
            ts.site_iter()
                .next()
                .unwrap()
                .mutation_iter()
                .nth(index)
                .unwrap()
                .id(),
            mutid
        );
    }
    assert!(ts
        .site_iter()
        .next()
        .unwrap()
        .mutation_iter()
        .nth(100)
        .is_none());
}

#[test]
fn test_site_mutation_iterators_meet_in_middle() {
    let mut tables = tskit::TableCollection::new(100.0).unwrap();
    let n0 = tables
        .add_node(tskit::NodeFlags::IS_SAMPLE, 0.0, -1, -1)
        .unwrap();
    let s0 = tables.add_site(50., None).unwrap();
    let m0 = tables.add_mutation(s0, n0, -1, 20.0, None).unwrap();
    let m1 = tables.add_mutation(s0, n0, -1, 10.0, None).unwrap();
    let m2 = tables.add_mutation(s0, n0, -1, 5.0, None).unwrap();
    let m3 = tables.add_mutation(s0, n0, -1, 2.0, None).unwrap();

    tables.full_sort(0).unwrap();
    tables.build_index().unwrap();
    tables
        .compute_mutation_parents(tskit::MutationParentsFlags::default())
        .unwrap();
    let ts = tables.deepcopy().unwrap().tree_sequence(0).unwrap();

    // The following makes sure that our Iterator and DoubleEndedIterator
    // over mutations do not result in infinite loops
    let mut collected = vec![];
    for site in ts.site_iter() {
        let mut iterator = site.mutation_iter();
        while let Some(mutation) = iterator.next() {
            collected.push(mutation.id());
            if let Some(mback) = iterator.next_back() {
                collected.push(mback.id());
            }
        }
    }
    assert_eq!(&collected, &[m0, m3, m1, m2]);
}

#[test]
fn test_site_mutation_co_iteration() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let n0 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let n1 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let n2 = tables.add_node(0, 2.0, -1, -1).unwrap();
    let _ = tables.add_edge(0., 100., n1, n0).unwrap();
    let _ = tables.add_edge(0., 100., n2, n1).unwrap();
    let s0 = tables.add_site(5.0, None).unwrap();
    let m0 = tables.add_mutation(s0, n1, -1, 1.0, None).unwrap();
    let m1 = tables.add_mutation(s0, n0, -1, 0.0, None).unwrap();
    // Site with no mutations!
    let _ = tables.add_site(10.0, None).unwrap();
    let s1 = tables.add_site(11.0, None).unwrap();
    let m2 = tables.add_mutation(s1, n0, -1, 0.0, None).unwrap();
    tables.build_index().unwrap();
    assert_eq!(tables.sites().num_rows(), 3);
    assert_eq!(tables.mutations().num_rows(), 3);
    tables
        .compute_mutation_parents(tskit::MutationParentsFlags::default())
        .unwrap();
    let ts = tables.tree_sequence(0).unwrap();

    let contents = ts
        .site_iter()
        .flat_map(|site| {
            // we take id() here...
            let id = site.id();
            // ...because we consume site here, making
            // site.id() inaccesible via the closure
            site.mutation_iter().map(move |m| (id, m.id()))
        })
        .collect::<Vec<_>>();
    assert_eq!(contents.len(), 3, "{contents:?}");
    for t in [(s0, m0), (s0, m1), (s1, m2)] {
        assert!(contents.contains(&t), "{contents:?} does not contain {t:?}")
    }
    let contents = ts
        .site_iter()
        .skip(1)
        .flat_map(|site| {
            // we take id() here...
            let id = site.id();
            // ...because we consume site here, making
            // site.id() inaccesible via the closure
            site.mutation_iter().map(move |m| (id, m.id()))
        })
        .collect::<Vec<_>>();
    assert_eq!(contents.len(), 1, "{contents:?}");
    for t in [(s1, m2)] {
        assert!(contents.contains(&t), "{contents:?} does not contain {t:?}")
    }

    for site in ts.site_iter() {
        assert!(site.ancestral_state().is_none());
        assert!(site.metadata().is_none());
    }

    for m in ts.site_iter().flat_map(|site| site.mutation_iter()) {
        assert!(m.metadata().is_none());
        assert!(m.inherited_state().is_none());
        assert!(!m.edge().is_null());
        assert!(!m.site().is_null());
        assert!(m.time() >= ts.nodes().time(m.node()).unwrap());
    }
}

#[test]
fn test_site_mutation_co_iteration_fully_loaded() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let parent = tables.add_node(0, 1., -1, -1).unwrap();
    let child = tables
        .add_node(tskit::NodeFlags::IS_SAMPLE, 0., -1, -1)
        .unwrap();
    let e = tables.add_edge(0., 100., parent, child).unwrap();
    struct RawMetadata {
        md: Vec<u8>,
    }

    impl tskit::metadata::MetadataRoundtrip for RawMetadata {
        fn encode(&self) -> Result<Vec<u8>, tskit::metadata::MetadataError> {
            Ok(self.md.clone())
        }
        fn decode(md: &[u8]) -> Result<Self, tskit::metadata::MetadataError>
        where
            Self: Sized,
        {
            Ok(Self { md: md.to_vec() })
        }
    }
    impl tskit::metadata::SiteMetadata for RawMetadata {}
    impl tskit::metadata::MutationMetadata for RawMetadata {}
    let s = tables
        .add_site_with_metadata(
            10.,
            Some("ancestral".as_bytes()),
            &RawMetadata {
                md: "site_md".as_bytes().to_vec(),
            },
        )
        .unwrap();
    let _m0 = tables
        .add_mutation_with_metadata(
            s,
            child,
            -1,
            0.5,
            Some("derived_state".as_bytes()),
            &RawMetadata {
                md: "site_md".as_bytes().to_vec(),
            },
        )
        .unwrap();
    let _m1 = tables
        .add_mutation_with_metadata(
            s,
            child,
            -1,
            0.5,
            Some("another_derived_state".as_bytes()),
            &RawMetadata {
                md: "site_md".as_bytes().to_vec(),
            },
        )
        .unwrap();
    tables.full_sort(0).unwrap();
    tables.build_index().unwrap();
    let ts = tables
        .tree_sequence(tskit::TreeSequenceFlags::default().compute_mutation_parents())
        .unwrap();
    for site in ts.site_iter() {
        assert!(site.ancestral_state().is_some());
        assert!(site.metadata().is_some());
    }

    for m in ts.site_iter().flat_map(move |site| site.mutation_iter()) {
        assert!(m.metadata().is_some());
        assert!(m.inherited_state().is_some());
        assert_eq!(m.edge(), e);
        assert_eq!(m.site(), s);
        assert!(m.time() >= ts.nodes().time(m.node()).unwrap());
    }
}

#[test]
fn test_tree_site_iter() {
    let mut tables = make_small_table_collection_two_trees();
    let s0 = tables.add_site(250., Some("A".as_bytes())).unwrap();
    let m0 = tables
        .add_mutation(s0, 2, -1, 0.1, Some("G".as_bytes()))
        .unwrap();
    let m1 = tables
        .add_mutation(s0, 3, -1, 0.1, Some("C".as_bytes()))
        .unwrap();
    let s1 = tables.add_site(750., Some("T".as_bytes())).unwrap();
    let m2 = tables
        .add_mutation(s1, 1, -1, 1.1, Some("G".as_bytes()))
        .unwrap();
    tables.full_sort(0).unwrap();
    tables.build_index().unwrap();
    let ts = tables.tree_sequence(0).unwrap();
    let mut tree_iter = ts.tree_iterator(0).unwrap();
    let mut sites = vec![];
    let mut mutations = vec![];
    while let Some(tree) = tree_iter.next() {
        for s in tree.site_iter() {
            sites.push((tree.index(), s.id()));
            for m in s.mutation_iter() {
                mutations.push((tree.index(), m.id()));
            }
        }
    }
    assert_eq!(sites.len(), 2);
    assert_eq!(mutations.len(), 3);
    assert_eq!(mutations.iter().filter(|(t, _)| t == &0).count(), 2);
    assert!(mutations
        .iter()
        .filter(|(t, _)| t == &0)
        .any(|(_, m)| m == m0));
    assert!(mutations
        .iter()
        .filter(|(t, _)| t == &0)
        .any(|(_, m)| m == m1));
    assert!(mutations
        .iter()
        .filter(|(t, _)| t == &1)
        .any(|(_, m)| m == m2));
}

#[test]
fn test_tree_samples_iter() {
    let tables = make_small_table_collection_two_trees();
    let ts = tables.tree_sequence(0).unwrap();
    let mut tree_iter = ts
        .tree_iterator(tskit::TreeFlags::default().sample_lists())
        .unwrap();
    while let Some(tree) = tree_iter.next() {
        let samples = tree.samples_iter().unwrap().collect::<Vec<_>>();
        assert_eq!(samples.len(), 4);
        for i in [2, 3, 4, 5] {
            assert!(samples.contains(&(i.into())))
        }
    }
}

#[test]
fn test_treeseq_individual_iter() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let i0 = tables.add_individual(0, [1., 2.], None).unwrap();
    let i1 = tables.add_individual(0, [1., 2.], None).unwrap();
    let i2 = tables.add_individual(0, [3., 4.], [i0, i1]).unwrap();

    let n0 = tables
        .add_node(tskit::NodeFlags::default(), 1., -1, i0)
        .unwrap();
    let n1 = tables
        .add_node(tskit::NodeFlags::default(), 1., -1, i0)
        .unwrap();
    let n2 = tables
        .add_node(tskit::NodeFlags::default(), 1., -1, i1)
        .unwrap();
    let n3 = tables
        .add_node(tskit::NodeFlags::default(), 1., -1, i1)
        .unwrap();
    let n4 = tables
        .add_node(tskit::NodeFlags::IS_SAMPLE, 0., -1, i2)
        .unwrap();
    let n5 = tables
        .add_node(tskit::NodeFlags::IS_SAMPLE, 0., -1, i2)
        .unwrap();

    tables.add_edge(0., 50., n0, n4).unwrap();
    tables.add_edge(50., 100., n1, n4).unwrap();
    tables.add_edge(0., 25., n2, n5).unwrap();
    tables.add_edge(25., 100., n3, n5).unwrap();

    tables.full_sort(0).unwrap();
    tables.topological_sort_individuals(0).unwrap();

    let ts = tables
        .tree_sequence(tskit::TreeSequenceFlags::default().build_indexes())
        .unwrap();
    for ind in ts.individual_iter() {
        if let Some(parents) = ind.parents() {
            for p in [i0, i1] {
                assert!(parents.contains(&p))
            }
            let location = ind.location().unwrap();
            assert_eq!(location, [3., 4.]);
        } else {
            let location = ind.location().unwrap();
            assert_eq!(location, [1., 2.]);
        }
    }
    for (i, ind) in ts.individual_iter().enumerate() {
        assert_eq!(ts.individual_iter().nth(i).unwrap(), ind);
        assert_eq!(ts.individual(i32::try_from(i).unwrap()).unwrap(), ind);
    }
}

#[test]
fn test_site_iterator_double_ended() {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    let s0 = tables.add_site(10., None).unwrap();
    let s1 = tables.add_site(20., None).unwrap();
    let s2 = tables.add_site(30., None).unwrap();
    let s3 = tables.add_site(40., None).unwrap();

    let ts = tables
        .tree_sequence(tskit::TreeSequenceFlags::default().build_indexes())
        .unwrap();
    let mut sites = vec![];
    let mut iter = ts.site_iter();
    while let Some(site) = iter.next() {
        sites.push(site.id());
        if let Some(site) = iter.next_back() {
            sites.push(site.id())
        }
    }
    assert_eq!(&sites, &[s0, s3, s1, s2])
}

#[test]
fn test_subtrees_from_non_root_nodes() {
    let treeseq = treeseq_from_small_table_collection_two_trees();
    let mut iter = treeseq.tree_iterator(tskit::TreeFlags::default()).unwrap();
    let tree = iter.next().unwrap();
    let pre = tree
        .traverse_nodes_from_root(1.into(), tskit::NodeTraversalOrder::Preorder)
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(pre, &[1, 4, 5]);

    compare_preorder_to_c_api(tree, 1.into(), &pre);

    let tree = iter.next().unwrap();
    let pre = tree
        .traverse_nodes_from_root(1.into(), tskit::NodeTraversalOrder::Preorder)
        .unwrap()
        .collect::<Vec<_>>();
    let mut expected = vec![1];
    expected.extend(tree.children(1).map(i32::from));
    assert_eq!(pre, expected);

    compare_preorder_to_c_api(tree, 1.into(), &pre);

    let pre = tree
        .traverse_nodes_from_root(3.into(), tskit::NodeTraversalOrder::Preorder)
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(pre, &[3]);
    compare_preorder_to_c_api(tree, 3.into(), &pre);
}

// The following tests are lifted
// from another crate that identified
// the bug in the impl of nth
#[cfg(test)]
mod from_popgen_oxide {
    //   Tree for [0, 50)
    //   --6-- <- time 30
    //   |   |
    //  -5-- | <- time 20
    //  |  | |
    // -4- | | <- time 10
    // | | | |
    // 0 1 2 3 <- time 0
    // Tree for [50, 100)
    //  --6---
    //  |  | |
    // -5- | |
    // | | | |
    // 4 | | |
    // | | | |
    // 0 1 2 3
    fn make_two_different_four_sample_trees() -> tskit::TableCollection {
        let mut tables = tskit::TableCollection::new(100.0).unwrap();

        for _ in 0..4 {
            tables
                .add_node(
                    tskit::NodeFlags::new_sample(),
                    0.0,
                    tskit::PopulationId::NULL,
                    tskit::IndividualId::NULL,
                )
                .unwrap();
        }

        for (_, time) in (0..3).zip([10.0, 20.0, 30.0]) {
            tables
                .add_node(
                    tskit::NodeFlags::default(),
                    time,
                    tskit::PopulationId::NULL,
                    tskit::IndividualId::NULL,
                )
                .unwrap();
        }

        tables.add_edge(0., 50., 4, 0).unwrap();
        tables.add_edge(0., 50., 4, 1).unwrap();
        tables.add_edge(0., 50., 5, 4).unwrap();
        tables.add_edge(0., 50., 5, 2).unwrap();
        tables.add_edge(0., 50., 6, 5).unwrap();
        tables.add_edge(0., 50., 6, 3).unwrap();

        tables.add_edge(50., 100., 6, 5).unwrap();
        tables.add_edge(50., 100., 6, 2).unwrap();
        tables.add_edge(50., 100., 6, 3).unwrap();
        tables.add_edge(50., 100., 5, 4).unwrap();
        tables.add_edge(50., 100., 5, 1).unwrap();
        tables.add_edge(50., 100., 4, 0).unwrap();

        tables
    }

    struct MutationData {
        node: tskit::NodeId,
        time: tskit::Time,
        derived_state: Vec<u8>,
    }

    impl MutationData {
        fn new<N, T>(node: N, time: T, derived_state: &str) -> Self
        where
            N: Into<tskit::NodeId>,
            T: Into<tskit::Time>,
        {
            Self {
                node: node.into(),
                time: time.into(),
                derived_state: derived_state.as_bytes().to_owned(),
            }
        }
    }

    struct SiteData {
        position: tskit::Position,
        ancestral_state: Vec<u8>,
        mutations: Vec<MutationData>,
    }

    impl SiteData {
        fn new<P, M>(position: P, ancestral_state: &str, mutations: M) -> Self
        where
            P: Into<tskit::Position>,
            M: IntoIterator<Item = MutationData>,
        {
            Self {
                position: position.into(),
                ancestral_state: ancestral_state.as_bytes().to_owned(),
                mutations: mutations.into_iter().collect::<Vec<_>>(),
            }
        }
    }
    fn add_sites_and_mutations<S>(tables: &mut tskit::TableCollection, data: S)
    where
        S: IntoIterator<Item = SiteData>,
    {
        for s in data {
            let site = tables
                .add_site(s.position, Some(&s.ancestral_state))
                .unwrap();
            for m in &s.mutations {
                tables
                    .add_mutation(
                        site,
                        m.node,
                        tskit::MutationId::NULL,
                        m.time,
                        Some(&m.derived_state),
                    )
                    .unwrap();
            }
        }
    }

    fn make_test_data<F, S>(make_tables: F, data: S) -> tskit::TreeSequence
    where
        F: Fn() -> tskit::TableCollection,
        S: IntoIterator<Item = SiteData>,
    {
        let mut tables = make_tables();
        add_sites_and_mutations(&mut tables, data);
        tables
            .full_sort(tskit::TableSortOptions::default())
            .unwrap();
        tables.build_index().unwrap();
        assert!(!tables.as_mut_ptr().is_null());
        tables
            .compute_mutation_parents(tskit::MutationParentsFlags::default())
            .unwrap();
        tables
            .tree_sequence(tskit::TreeSequenceFlags::default())
            .unwrap()
    }

    #[test]
    fn test_7() {
        use tskit::StreamingIterator;

        let site0 = SiteData::new(
            60.0,
            "G",
            vec![
                MutationData::new(5, 20.1, "A"),
                MutationData::new(4, 10.1, "G"),
                MutationData::new(1, 0.1, "C"),
            ],
        );
        let site1 = SiteData::new(
            40.0,
            "T",
            vec![
                MutationData::new(3, 20.1, "T"),
                MutationData::new(2, 0.1, "G"),
                MutationData::new(4, 10.1, "G"),
            ],
        );
        let ts = make_test_data(make_two_different_four_sample_trees, vec![site0, site1]);
        let mut tree_iter = ts.tree_iterator(0).unwrap();
        let mut site = 0_usize;
        while let Some(tree) = tree_iter.next() {
            for _ in ts
                .site_iter()
                .skip(site)
                .take_while(|s| s.position() < tree.interval().1)
            {
                site += 1;
            }
        }
        assert_eq!(site, ts.sites().num_rows().as_usize())
    }
}
