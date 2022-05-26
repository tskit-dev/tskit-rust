// ANCHOR: use_prelude
// The prelude exports traits, etc., that are tedious to live without
use tskit::prelude::*;
// But, not all types are exported.
// Here, we specifically import TableCollection
// to avoid constantly having to refer to the namespace.
use tskit::TableCollection;
// ANCHOR_END: use_prelude

fn initialize() -> tskit::TableCollection {
    #[allow(unused_mut)]
    // ANCHOR: initialization
    let mut tables = match TableCollection::new(1000.0) {
        Ok(t) => t,
        Err(e) => panic!("{e}"),
    };
    // ANCHOR_END: initialization

    tables
}

fn add_rows(tables: &mut TableCollection) -> Vec<NodeId> {
    // ANCHOR: init_node_vec
    let mut node_ids = vec![];
    // ANCHOR_END: init_node_vec
    // ANCHOR: add_first_node
    match tables.add_node(
        0,
        tskit::Time::from(2.0),
        tskit::PopulationId::NULL,
        tskit::IndividualId::NULL,
    ) {
        Ok(id) => {
            assert_eq!(id, 0);
            node_ids.push(id)
        }
        Err(e) => panic!("{}", e),
    };
    // ANCHOR_END: add_first_node

    // ANCHOR: add_second_node
    let id = tables
        .add_node(0, 1.0, -1, tskit::IndividualId::NULL)
        .unwrap();
    assert_eq!(id, 1);
    assert_eq!(id, NodeId::from(1));
    node_ids.push(id);
    // ANCHOR_END: add_second_node

    // ANCHOR: add_sample_nodes
    for i in 2..6_i32 {
        match tables.add_node(
            tskit::TSK_NODE_IS_SAMPLE,
            0.0,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        ) {
            Ok(id) => {
                assert_eq!(id, i);
                let n = NodeId::from(i);
                assert_eq!(id, n);
                node_ids.push(id)
            }
            Err(e) => panic!("{}", e),
        }
    }
    // ANCHOR_END: add_sample_nodes

    // ANCHOR: add_edges
    tables
        .add_edge(tskit::Position::from(500.), 1000., node_ids[0], node_ids[1])
        .unwrap();
    tables.add_edge(0., 500., 0, 2).unwrap();
    tables.add_edge(0., 1000., 0, 3).unwrap();
    tables.add_edge(500., 1000., 1, 2).unwrap();
    tables.add_edge(0., 1000., 1, 4).unwrap();
    tables.add_edge(0., 1000., 1, 5).unwrap();
    // ANCHOR_END: add_edges

    node_ids
}

fn main() {
    {
        let _ = initialize();
    }

    {
        let mut tables = initialize();
        let _ = add_rows(&mut tables);
    }
}
