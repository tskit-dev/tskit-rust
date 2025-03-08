#[test]
fn initialize_from_table_collection() {
    // ANCHOR: build_tables
    use tskit::prelude::*;
    use tskit::TableCollection;
    use tskit::TableSortOptions;
    use tskit::TreeFlags;
    use tskit::TreeSequenceFlags;

    let mut tables = TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 2.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
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
    // ANCHOR_END: build_tables

    // ANCHOR: sort_tables
    tables.full_sort(TableSortOptions::default()).unwrap();
    // ANCHOR_END: sort_tables

    // ANCHOR: index_tables
    tables.build_index().unwrap();
    // ANCHOR_END: index_tables

    // ANCHOR: create_tree_sequence
    let treeseq = tables.tree_sequence(TreeSequenceFlags::default()).unwrap();
    // ANCHOR_END: create_tree_sequence

    // ANCHOR: iterate_trees
    let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();

    while let Some(_tree) = tree_iterator.next() {
        // _tree is a tskit::Tree
    }
    // ANCHOR_END: iterate_trees

    let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    // ANCHOR: iterate_node_siblings
    // This is an enum defining supported
    // traversal orders through a Tree.
    use tskit::NodeTraversalOrder;
    while let Some(tree) = tree_iterator.next() {
        for node in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            if let Some(parent) = tree.parent(node) {
                // Collect the siblings of node into a Vec
                // The children function returns another iterator
                let _siblings = tree
                    .children(parent)
                    .filter(|child| child != node)
                    .collect::<Vec<_>>();
            }
        }
    }
    // ANCHOR_END: iterate_node_siblings

    let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    // ANCHOR: iterate_node_siblings_via_arrays
    while let Some(tree) = tree_iterator.next() {
        let parents = tree.parent_array();
        let rsibs = tree.right_sib_array();
        let lchildren = tree.left_child_array();
        for node in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            let mut siblings = vec![];
            assert!(!node.is_null());
            if let Some(parent) = parents.get(usize::try_from(node).unwrap()) {
                if !parent.is_null() {
                    if let Some(child) = lchildren.get(usize::try_from(*parent).unwrap()) {
                        let mut u = *child;
                        while !u.is_null() {
                            if u != node {
                                siblings.push(u);
                            }
                            if let Some(sib) = rsibs.get(usize::try_from(u).unwrap()) {
                                u = *sib;
                            }
                        }
                    }
                }
            }
        }
    }
    // ANCHOR_END: iterate_node_siblings_via_arrays

    let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    // ANCHOR: iterate_node_siblings_via_array_getters
    while let Some(tree) = tree_iterator.next() {
        for node in tree.traverse_nodes(NodeTraversalOrder::Preorder) {
            let mut siblings = vec![];
            if let Some(parent) = tree.parent(node) {
                if let Some(child) = tree.left_child(parent) {
                    let mut u = child;
                    while !u.is_null() {
                        if u != node {
                            siblings.push(u);
                        }
                        if let Some(sib) = tree.right_sib(u) {
                            u = sib;
                        }
                    }
                }
            }
        }
    }
    // ANCHOR_END: iterate_node_siblings_via_array_getters

    // let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    // let mut total_branch_lengths = vec![];
    // while let Some(tree) = tree_iterator.next() {
    //     total_branch_lengths.push(tree.total_branch_length(false).unwrap());
    // }

    // let mut tree_iterator = treeseq.tree_iterator(TreeFlags::default()).unwrap();
    // let mut total_branch_lengths_ll = vec![];
    // let mut x = 0.0;

    // while let Some(tree) = tree_iterator.next() {
    //     let l =
    //         unsafe { tskit::bindings::tsk_tree_get_total_branch_length(tree.as_ptr(), -1, &mut x) };
    //     assert!(l >= 0);
    //     total_branch_lengths_ll.push(x);
    // }

    // for (i, j) in total_branch_lengths
    //     .iter()
    //     .zip(total_branch_lengths_ll.iter())
    // {
    //     assert_eq!(i, j, "{} {}", i, j);
    // }

    // ANCHOR: iterate_edge_differences
    if let Ok(mut edge_diff_iterator) = treeseq.edge_differences_iter() {
        while let Some(diffs) = edge_diff_iterator.next() {
            for edge_removal in diffs.edge_removals() {
                println!("{}", edge_removal);
            }
            for edge_insertion in diffs.edge_insertions() {
                println!("{}", edge_insertion);
            }
        }
    } else {
        panic!("creating edge diffs iterator failed");
    }
    // ANCHOR_END: iterate_edge_differences

    // ANCHOR: iterate_edge_differences_update_parents
    let num_nodes = treeseq.nodes().num_rows().as_usize();
    // num_nodes + 1 to reflect a "virtual root" present in
    // the tree arrays
    let mut parents = vec![NodeId::NULL; num_nodes + 1];
    match treeseq.edge_differences_iter() {
        Ok(mut ediff_iter) => match treeseq.tree_iterator(0) {
            Ok(mut tree_iter) => {
                while let Some(diffs) = ediff_iter.next() {
                    let tree = tree_iter.next().unwrap();
                    for edge_out in diffs.edge_removals() {
                        let c = edge_out.child();
                        parents[c.as_usize()] = NodeId::NULL;
                    }
                    for edge_in in diffs.edge_insertions() {
                        let c = edge_in.child();
                        parents[c.as_usize()] = edge_in.parent();
                    }
                    assert_eq!(tree.parent_array(), &parents);
                }
            }
            Err(e) => panic!("error creating tree iter: {:?}", e),
        },
        Err(e) => panic!("error creating edge diff iter: {:?}", e),
    }
    // ANCHOR_END: iterate_edge_differences_update_parents
}
