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
}
