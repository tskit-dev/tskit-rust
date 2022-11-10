#[test]
fn simple_table_collection_creation_with_newtype() {
    // ANCHOR: create_table_collection_with_newtype
    let sequence_length = tskit::Position::from(100.0);
    if let Ok(tables) = tskit::TableCollection::new(sequence_length) {
        assert_eq!(tables.sequence_length(), sequence_length);
        // In tskit, the various newtypes can be compared to
        // the low-level types they wrap.
        assert_eq!(tables.sequence_length(), 100.0);
    } else {
        panic!(
            "TableCollection creation sequence length = {} failed",
            sequence_length
        );
    }
    // ANCHOR_END: create_table_collection_with_newtype
}

#[test]
fn simple_table_collection_creation() {
    // ANCHOR: create_table_collection
    let tables = tskit::TableCollection::new(100.0).unwrap();
    // ANCHOR_END: create_table_collection
    assert_eq!(tables.sequence_length(), 100.0);
}

#[test]
fn add_node_without_metadata() {
    {
        // ANCHOR: add_node_without_metadata
        let mut tables = tskit::TableCollection::new(100.0).unwrap();
        if let Ok(node_id) = tables.add_node(
            0,                         // Node flags
            tskit::Time::from(0.0),    // Birth time
            tskit::PopulationId::NULL, // Population id
            tskit::IndividualId::NULL, // Individual id
        ) {
            assert_eq!(node_id, 0);
        }
        // ANCHOR_END: add_node_without_metadata
    }
    {
        let mut tables = tskit::TableCollection::new(100.0).unwrap();
        // ANCHOR: add_node_without_metadata_using_into
        let node_id = tables.add_node(0, 0.0, -1, -1).unwrap();
        // ANCHOR_END: add_node_without_metadata_using_into
        assert_eq!(node_id, 0);
    }
}

#[test]
fn add_node_handle_error() {
    // ANCHOR: integrity_check
    let mut tables = tskit::TableCollection::new(100.0).unwrap();
    // Everything about this edge is wrong...
    tables.add_edge(-1.0, 110.0, 0, 1).unwrap();
    // ...and we can catch that here
    match tables.check_integrity(tskit::TableIntegrityCheckFlags::default()) {
        Ok(code) => panic!("expected Err(e) but got code: {}", code),
        // tskit::TskitError can be formatted into the same
        // error messages that tskit-c/tskit-python give.
        Err(e) => println!("{}", e),
    }
    // ANCHOR_END: integrity_check
    assert!(tables
        .check_integrity(tskit::TableIntegrityCheckFlags::default())
        .is_err());
}

#[test]
fn get_data_from_edge_table() {
    use rand::distributions::Distribution;
    use tskit::prelude::*;
    let sequence_length = tskit::Position::from(100.0);
    let mut rng = rand::thread_rng();
    let random_pos = rand::distributions::Uniform::new::<f64, f64>(0., sequence_length.into());
    let mut tables = tskit::TableCollection::new(sequence_length).unwrap();
    let child = tables.add_node(0, 0.0, -1, -1).unwrap();
    let parent = tables.add_node(0, 1.0, -1, -1).unwrap();
    let mut left = tskit::Position::from(random_pos.sample(&mut rng));
    let mut right = tskit::Position::from(random_pos.sample(&mut rng));
    if left > right {
        std::mem::swap(&mut left, &mut right);
    }

    // ANCHOR: get_edge_table_columns
    if let Ok(edge_id) = tables.add_edge(left, right, parent, child) {
        // Take a reference to an edge table (& tskit::EdgeTable)
        let edges = tables.edges();
        if let Some(p) = edges.parent(edge_id) {
            assert_eq!(p, parent);
        }
        if let Some(c) = edges.child(edge_id) {
            assert_eq!(c, child);
        }
        if let Some(l) = edges.left(edge_id) {
            assert_eq!(l, left);
        }
        if let Some(r) = edges.right(edge_id) {
            assert_eq!(r, right);
        }
    } else {
        panic!("that should have worked...");
    }
    // ANCHOR_END: get_edge_table_columns

    // ANCHOR: get_edge_table_columns_out_of_range
    assert!(tables.edges().parent(tskit::EdgeId::NULL).is_none());
    // ANCHOR_END: get_edge_table_columns_out_of_range

    let edge_id = tskit::EdgeId::from(0);
    // ANCHOR: get_edge_table_row_by_id
    if let Some(row) = tables.edges().row(edge_id) {
        assert_eq!(row.id, 0);
        assert_eq!(row.left, left);
        assert_eq!(row.right, right);
        assert_eq!(row.parent, parent);
        assert_eq!(row.child, child);
    } else {
        panic!("that should have worked...");
    }
    // ANCHOR_END: get_edge_table_row_by_id

    // ANCHOR: get_edge_table_row_view_by_id
    if let Some(row_view) = tables.edges().row_view(edge_id) {
        assert_eq!(row_view.id, 0);
        assert_eq!(row_view.left, left);
        assert_eq!(row_view.right, right);
        assert_eq!(row_view.parent, parent);
        assert_eq!(row_view.child, child);
    } else {
        panic!("that should have worked...");
    }
    // ANCHOR_END: get_edge_table_row_view_by_id

    // ANCHOR: get_edge_table_rows_by_lending_iterator
    let mut edge_table_lending_iter = tables.edges().lending_iter();
    while let Some(row_view) = edge_table_lending_iter.next() {
        // there is only one row!
        assert_eq!(row_view.id, 0);
        assert_eq!(row_view.left, left);
        assert_eq!(row_view.right, right);
        assert_eq!(row_view.parent, parent);
        assert_eq!(row_view.child, child);
        assert!(row_view.metadata.is_none()); // no metadata in our table
    }
    // ANCHOR_END: get_edge_table_rows_by_lending_iterator

    assert!(tables
        .check_integrity(tskit::TableIntegrityCheckFlags::default())
        .is_ok());

    // ANCHOR: get_edge_table_rows_by_iterator
    for row in tables.edges_iter() {
        // there is only one row!
        assert_eq!(row.id, 0);
        assert_eq!(row.left, left);
        assert_eq!(row.right, right);
        assert_eq!(row.parent, parent);
        assert_eq!(row.child, child);
    }
    // ANCHOR_END: get_edge_table_rows_by_iterator

    assert!(tables
        .check_integrity(tskit::TableIntegrityCheckFlags::default())
        .is_ok());
}
