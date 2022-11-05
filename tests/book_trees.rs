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
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
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
                    .filter(|child| child != &node)
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
}
