fn make_treeseq() -> tskit::TreeSequence {
    let mut tables = tskit::TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 2.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL)
        .unwrap();
    tables
        .add_node(0, 1.0, tskit::PopulationId::NULL, tskit::IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            tskit::NodeFlags::new_sample(),
            0.0,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(500., 1000., 0, 1).unwrap();
    tables.add_edge(0., 500., 0, 2).unwrap();
    tables.add_edge(0., 1000., 0, 3).unwrap();
    tables.add_edge(500., 1000., 1, 2).unwrap();
    tables.add_edge(0., 1000., 1, 4).unwrap();
    tables.add_edge(0., 1000., 1, 5).unwrap();

    tables
        .full_sort(tskit::TableSortOptions::default())
        .unwrap();

    tables.build_index().unwrap();

    tables
        .tree_sequence(tskit::TreeSequenceFlags::default())
        .unwrap()
}

// A fundamental property of iterators is that their Items
// are collectible into objects that are valid to use later.

#[test]
fn test_collected_edge_insertions() {
    let ts = make_treeseq();
    // The ergonomics here seem a bit ugly but it is a corner case?
    let insertions = ts
        .edge_differences_iter()
        .flat_map(|d| d.insertions().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(insertions.len(), ts.edge_insertion_order().len());
    for (i, j) in insertions.iter().zip(ts.edge_insertion_order().iter()) {
        assert_eq!(
            i.parent(),
            ts.tables().edges().parent_column()[j.as_usize()]
        );
        assert_eq!(i.child(), ts.tables().edges().child_column()[j.as_usize()]);
        assert_eq!(i.left(), ts.tables().edges().left_column()[j.as_usize()]);
        assert_eq!(i.right(), ts.tables().edges().right_column()[j.as_usize()]);
    }

    // Better ergonomics
    let mut insertions = vec![];
    for diffs in ts.edge_differences_iter() {
        insertions.extend(diffs.insertions());
    }
    assert_eq!(insertions.len(), ts.edge_insertion_order().len());
    for (i, j) in insertions.iter().zip(ts.edge_insertion_order().iter()) {
        assert_eq!(
            i.parent(),
            ts.tables().edges().parent_column()[j.as_usize()]
        );
        assert_eq!(i.child(), ts.tables().edges().child_column()[j.as_usize()]);
        assert_eq!(i.left(), ts.tables().edges().left_column()[j.as_usize()]);
        assert_eq!(i.right(), ts.tables().edges().right_column()[j.as_usize()]);
    }
}

#[test]
fn test_collect_edge_diff_iterators() {
    let ts = make_treeseq();

    let diffs = ts.edge_differences_iter().collect::<Vec<_>>();

    for (di, dj) in diffs.iter().zip(ts.edge_differences_iter()) {
        for (ri, rj) in di.removals().zip(dj.removals()) {
            assert_eq!(ri.parent(), rj.parent());
            assert_eq!(ri.child(), rj.child());
            assert_eq!(ri.left(), rj.left());
            assert_eq!(ri.right(), rj.right());
        }
    }

    let insertions = diffs
        .iter()
        .flat_map(|d| d.insertions())
        .collect::<Vec<_>>();
    assert_eq!(insertions.len(), ts.edge_insertion_order().len());
    for (i, j) in insertions.iter().zip(ts.edge_insertion_order().iter()) {
        assert_eq!(
            i.parent(),
            ts.tables().edges().parent_column()[j.as_usize()]
        );
        assert_eq!(i.child(), ts.tables().edges().child_column()[j.as_usize()]);
        assert_eq!(i.left(), ts.tables().edges().left_column()[j.as_usize()]);
        assert_eq!(i.right(), ts.tables().edges().right_column()[j.as_usize()]);
    }

    let removals = diffs.iter().flat_map(|d| d.removals()).collect::<Vec<_>>();
    let removal_order = ts.edge_removal_order();
    // Removals have some nuance:
    // The "standard" loop ends when all IMSERTIONS havee
    // been processed, which means that all edges
    // leaving the tree at the "sequence length" are never visited.
    let num_removals_not_at_end = removal_order
        .iter()
        .filter(|r| {
            ts.tables().edges().right_column()[r.as_usize()] != ts.tables().sequence_length()
        })
        .count();
    assert_eq!(removals.len(), num_removals_not_at_end);

    for (i, j) in removals.iter().zip(removal_order.iter()) {
        assert_eq!(
            i.parent(),
            ts.tables().edges().parent_column()[j.as_usize()]
        );
        assert_eq!(i.child(), ts.tables().edges().child_column()[j.as_usize()]);
        assert_eq!(i.left(), ts.tables().edges().left_column()[j.as_usize()]);
        assert_eq!(i.right(), ts.tables().edges().right_column()[j.as_usize()]);
    }
}
