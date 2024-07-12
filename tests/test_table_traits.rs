#[derive(PartialEq, Debug)]
struct IteratorOutput {
    edges: Vec<tskit::EdgeTableRow>,
    nodes: Vec<tskit::NodeTableRow>,
}

impl IteratorOutput {
    fn new_from_tables(tables: &tskit::TableCollection) -> Self {
        let edges = tables.edges().iter().collect::<Vec<_>>();
        let nodes = tables.nodes().iter().collect::<Vec<_>>();
        Self { edges, nodes }
    }

    fn new_from_treeseq(treeseq: &tskit::TreeSequence) -> Self {
        let edges = treeseq.tables().edges().iter().collect::<Vec<_>>();
        let nodes = treeseq.tables().nodes().iter().collect::<Vec<_>>();
        Self { edges, nodes }
    }

    fn new_from_table_access<T>(access: &T) -> Self
    where
        T: tskit::TableAccess,
    {
        let edges = access.edges().iter().collect::<Vec<_>>();
        let nodes = access.nodes().iter().collect::<Vec<_>>();
        Self { edges, nodes }
    }

    fn new_from_table_iteration<T>(iterator: &T) -> Self
    where
        T: tskit::TableIteration,
    {
        let edges = iterator.edges().iter().collect::<Vec<_>>();
        let nodes = iterator.nodes().iter().collect::<Vec<_>>();
        Self { edges, nodes }
    }

    fn new_from_dyn(dynamic: &dyn tskit::ObjectSafeTableIteration) -> Self {
        let edges = dynamic.edges().iter().collect::<Vec<_>>();
        let nodes = dynamic.nodes().iter().collect::<Vec<_>>();
        Self { edges, nodes }
    }
}

fn validate_output_from_tables(tables: tskit::TableCollection) {
    let tables_output = IteratorOutput::new_from_tables(&tables);
    let access_output = IteratorOutput::new_from_table_access(&tables);
    assert_eq!(tables_output, access_output);
    let iteration_output = IteratorOutput::new_from_table_iteration(&tables);
    assert_eq!(tables_output, iteration_output);
    let boxed = Box::new(tables);
    let dynamic_output = IteratorOutput::new_from_dyn(&boxed);
    assert_eq!(tables_output, dynamic_output);
}

fn validate_output_from_treeseq(treeseq: tskit::TreeSequence) {
    let treeseq_output = IteratorOutput::new_from_treeseq(&treeseq);
    let access_output = IteratorOutput::new_from_table_access(&treeseq);
    assert_eq!(treeseq_output, access_output);
    let iteration_output = IteratorOutput::new_from_table_iteration(&treeseq);
    assert_eq!(treeseq_output, iteration_output);
    let boxed = Box::new(treeseq);
    let dynamic_output = IteratorOutput::new_from_dyn(&boxed);
    assert_eq!(treeseq_output, dynamic_output);
}

fn make_tables() -> tskit::TableCollection {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 0.0, -1, -1)
        .unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 1.0, -1, -1)
        .unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 1.0, -1, -1)
        .unwrap();
    tables.add_edge(0., 50., 1, 0).unwrap();
    tables.add_edge(50., 100., 2, 0).unwrap();
    tables
}

#[test]
fn test_traits_with_table_collection() {
    let tables = make_tables();
    validate_output_from_tables(tables)
}

#[test]
fn test_traits_with_tree_sequence() {
    let mut tables = make_tables();
    tables.full_sort(tskit::TableSortOptions::default()).unwrap();
    tables.build_index().unwrap();
    let treeseq = tskit::TreeSequence::try_from(tables).unwrap();
    validate_output_from_treeseq(treeseq)
}
