use tskit::TableColumn;

#[test]
fn test_empty_table_collection() {
    macro_rules! validate_empty_tables {
        ($tables: ident, $table: ident, $table_iter: ident, $row: expr) => {
            assert!($tables.$table().row($row).is_none());
            assert!($tables.$table().row_view($row).is_none());
            assert_eq!($tables.$table().num_rows(), 0);
            assert_eq!($tables.$table().iter().count(), 0);
            assert_eq!($tables.$table_iter().count(), 0);
        };
    }
    let tables = tskit::TableCollection::new(10.).unwrap();

    for row in [0, -1, 303] {
        validate_empty_tables!(tables, edges, edges_iter, row);
        validate_empty_tables!(tables, nodes, nodes_iter, row);
        validate_empty_tables!(tables, sites, sites_iter, row);
        validate_empty_tables!(tables, mutations, mutations_iter, row);
        validate_empty_tables!(tables, individuals, individuals_iter, row);
        validate_empty_tables!(tables, populations, populations_iter, row);
        validate_empty_tables!(tables, migrations, migrations_iter, row);
        #[cfg(feature = "provenance")]
        {
            validate_empty_tables!(tables, provenances, provenances_iter, row);
        }
    }
}

// We are not checking column getters here.
// We are not doing integrity checks.
#[cfg(test)]
mod test_adding_rows_without_metadata {
    macro_rules! add_row_without_metadata {
        ($table: ident, $adder: ident, $($payload: expr),* ) => {{
            {
                let mut tables = tskit::TableCollection::new(10.).unwrap();
                match tables.$adder($($payload ), *) {
                    Ok(id) => {
                        assert_eq!(tables.$table().num_rows(), 1);

                        // Rows store metadata as raw bytes here.
                        // (Not decoded.)
                        // The value should be None as the bytes
                        // are held in an Option.
                        match tables.$table().row(id) {
                            Some(row) => {
                                match tables.$table().row_view(id) {
                                    Some(view) => assert_eq!(view, row),
                                    None => panic!("if there is a row, there must be a row view")
                                }
                                assert!(row.metadata.is_none());

                                // A row equals itself
                                let row2 = tables.$table().row(id).unwrap();
                                assert_eq!(row, row2);

                                // create a second row w/identical payload
                                if let Ok(id2) = tables.$adder($($payload),*) {
                                    if let Some(row2) = tables.$table().row(id2) {
                                        // The rows have different id
                                        assert_ne!(row, row2);
                                    } else {
                                         panic!("Expected Some(row2) from {} table", stringify!(table))
                                    }
                                }

                            },
                            None => panic!("Expected Some(row) from {} table", stringify!(table))
                        }
                    },
                    Err(e) => panic!("Err from tables.{}: {:?}", stringify!(adder), e)
                }
                assert_eq!(tables.$table().iter().count(), 2);
                tables
            }
        }};
    }

    macro_rules! compare_column_to_raw_column {
        ($table: expr, $col: ident, $raw: ident) => {
            assert_eq!(
                $table.$col().len(),
                usize::try_from($table.num_rows()).unwrap()
            );
            assert_eq!(
                $table.$raw().len(),
                usize::try_from($table.num_rows()).unwrap()
            );
            assert!($table
                .$col()
                .iter()
                .zip($table.$raw().iter())
                .all(|(a, b)| a == b))
        };
    }

    macro_rules! compare_column_to_row {
        ($table: expr, $col: ident, $target: ident) => {
            assert!($table
                .$col()
                .iter()
                .zip($table.iter())
                .all(|(c, r)| c == &r.$target));
        };
    }

    // NOTE: all functions arguments for adding rows are Into<T>
    // where T is one of our new types.
    // Further, functions taking multiple inputs of T are defined
    // as X: Into<T>, X2: Into<T>, etc., allowing mix-and-match.

    #[test]
    fn test_adding_edge() {
        {
            let tables = add_row_without_metadata!(edges, add_edge, 0.1, 0.5, 0, 1); // left, right, parent, child
            compare_column_to_raw_column!(tables.edges(), left_slice, left_slice_raw);
            compare_column_to_raw_column!(tables.edges(), right_slice, right_slice_raw);
            compare_column_to_raw_column!(tables.edges(), parent_slice, parent_slice_raw);
            compare_column_to_raw_column!(tables.edges(), child_slice, child_slice_raw);

            compare_column_to_row!(tables.edges(), left_slice, left);
            compare_column_to_row!(tables.edges(), right_slice, right);
            compare_column_to_row!(tables.edges(), parent_slice, parent);
            compare_column_to_row!(tables.edges(), child_slice, child);
        }
        add_row_without_metadata!(edges, add_edge, tskit::Position::from(0.1), 0.5, 0, 1); // left, right, parent, child
        add_row_without_metadata!(edges, add_edge, 0.1, tskit::Position::from(0.5), 0, 1); // left, right, parent, child
        add_row_without_metadata!(
            edges,
            add_edge,
            0.1,
            0.5,
            tskit::NodeId::from(0),
            tskit::NodeId::from(1)
        ); // left, right, parent, child
        add_row_without_metadata!(
            edges,
            add_edge,
            tskit::Position::from(0.1),
            tskit::Position::from(0.5),
            tskit::NodeId::from(0),
            tskit::NodeId::from(1)
        ); // left, right, parent, child
    }

    #[test]
    fn test_adding_node() {
        {
            let tables = add_row_without_metadata!(
                nodes,
                add_node,
                tskit::NodeFlags::new_sample(),
                0.1,
                -1,
                -1
            ); // flags, time, population,
               // individual
            assert!(tables
                .nodes()
                .flags_slice()
                .iter()
                .zip(tables.nodes().flags_slice_raw().iter())
                .all(|(a, b)| a.bits() == *b));
            compare_column_to_raw_column!(tables.nodes(), time_slice, time_slice_raw);
            compare_column_to_raw_column!(tables.nodes(), population_slice, population_slice_raw);
            compare_column_to_raw_column!(tables.nodes(), individual_slice, individual_slice_raw);

            assert!(tables
                .nodes()
                .flags_slice()
                .iter()
                .zip(tables.nodes().iter())
                .all(|(c, r)| c == &r.flags));
            compare_column_to_row!(tables.nodes(), time_slice, time);
            compare_column_to_row!(tables.nodes(), population_slice, population);
            compare_column_to_row!(tables.nodes(), individual_slice, individual);
        }
        add_row_without_metadata!(
            nodes,
            add_node,
            tskit::NodeFlags::default(),
            tskit::Time::from(0.1),
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL
        );
    }

    #[test]
    fn test_adding_site() {
        // No ancestral state
        {
            let tables = add_row_without_metadata!(sites, add_site, 2. / 3., None);
            compare_column_to_raw_column!(tables.sites(), position_slice, position_slice_raw);
            compare_column_to_row!(tables.sites(), position_slice, position);
        }
        add_row_without_metadata!(sites, add_site, tskit::Position::from(2. / 3.), None);
        add_row_without_metadata!(sites, add_site, 2. / 3., Some(&[1_u8]));
        add_row_without_metadata!(
            sites,
            add_site,
            tskit::Position::from(2. / 3.),
            Some(&[1_u8])
        );
    }

    #[test]
    fn test_adding_mutation() {
        // site, node, parent mutation, time, derived_state
        // Each value is a different Into<T> so we skip doing
        // permutations
        {
            let tables = add_row_without_metadata!(mutations, add_mutation, 0, 0, -1, 0.0, None);
            compare_column_to_raw_column!(tables.mutations(), node_slice, node_slice_raw);
            compare_column_to_raw_column!(tables.mutations(), time_slice, time_slice_raw);
            compare_column_to_raw_column!(tables.mutations(), site_slice, site_slice_raw);
            compare_column_to_raw_column!(tables.mutations(), parent_slice, parent_slice_raw);

            compare_column_to_row!(tables.mutations(), node_slice, node);
            compare_column_to_row!(tables.mutations(), time_slice, time);
            compare_column_to_row!(tables.mutations(), site_slice, site);
            compare_column_to_row!(tables.mutations(), parent_slice, parent);
        }

        add_row_without_metadata!(mutations, add_mutation, 0, 0, -1, 0.0, Some(&[23_u8]));
    }

    #[test]
    fn test_adding_individual() {
        // flags, location, parents
        {
            let tables = add_row_without_metadata!(individuals, add_individual, 0, None, None);
            assert!(tables
                .individuals()
                .flags_slice()
                .iter()
                .zip(tables.individuals().flags_slice_raw().iter())
                .all(|(a, b)| a.bits() == *b));
            assert!(tables
                .individuals()
                .flags_slice()
                .iter()
                .zip(tables.individuals().iter())
                .all(|(c, r)| c == &r.flags));
        }
        add_row_without_metadata!(
            individuals,
            add_individual,
            tskit::IndividualFlags::default(),
            None,
            None
        );
        add_row_without_metadata!(individuals, add_individual, 0, &[0.2, 0.2], None);
        add_row_without_metadata!(
            individuals,
            add_individual,
            0,
            &[tskit::Location::from(0.2), tskit::Location::from(0.2)],
            None
        );
        add_row_without_metadata!(individuals, add_individual, 0, None, &[0, 1]);
        add_row_without_metadata!(
            individuals,
            add_individual,
            0,
            None,
            &[tskit::IndividualId::from(0), tskit::IndividualId::from(1)]
        );
    }

    #[test]
    fn test_adding_population() {
        // population table
        add_row_without_metadata!(populations, add_population,);
    }

    #[test]
    fn test_adding_migration() {
        // migration table
        // (left, right), node, (source, dest), time
        {
            let tables =
                add_row_without_metadata!(migrations, add_migration, (0., 1.), 0, (0, 1), 0.0);
            compare_column_to_raw_column!(tables.migrations(), left_slice, left_slice_raw);
            compare_column_to_raw_column!(tables.migrations(), right_slice, right_slice_raw);
            compare_column_to_raw_column!(tables.migrations(), node_slice, node_slice_raw);
            compare_column_to_raw_column!(tables.migrations(), time_slice, time_slice_raw);
            compare_column_to_raw_column!(tables.migrations(), source_slice, source_slice_raw);
            compare_column_to_raw_column!(tables.migrations(), dest_slice, dest_slice_raw);

            compare_column_to_row!(tables.migrations(), left_slice, left);
            compare_column_to_row!(tables.migrations(), right_slice, right);
            compare_column_to_row!(tables.migrations(), node_slice, node);
            compare_column_to_row!(tables.migrations(), time_slice, time);
            compare_column_to_row!(tables.migrations(), source_slice, source);
            compare_column_to_row!(tables.migrations(), dest_slice, dest);
        }
        add_row_without_metadata!(
            migrations,
            add_migration,
            (tskit::Position::from(0.), 1.),
            0,
            (0, 1),
            0.0
        );
        add_row_without_metadata!(
            migrations,
            add_migration,
            (0., tskit::Position::from(1.)),
            0,
            (0, 1),
            0.0
        );
        add_row_without_metadata!(
            migrations,
            add_migration,
            (0., 1.),
            tskit::NodeId::from(0),
            (0, 1),
            0.0
        );
        add_row_without_metadata!(
            migrations,
            add_migration,
            (0., 1.),
            0,
            (0, 1),
            tskit::Time::from(0.0)
        );
        add_row_without_metadata!(
            migrations,
            add_migration,
            (0., 1.),
            0,
            (tskit::PopulationId::from(0), 1),
            0.0
        );
        add_row_without_metadata!(
            migrations,
            add_migration,
            (0., 1.),
            0,
            (0, tskit::PopulationId::from(1)),
            0.0
        );
    }
}

#[cfg(test)]
#[cfg(feature = "derive")]
mod test_metadata_round_trips {
    macro_rules! build_metadata_types {
        ($md: ident) => {
            #[derive(
                serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, tskit::metadata::$md,
            )]
            #[serializer("serde_json")]
            struct MyMetadata {
                value: i32,
            }

            impl MyMetadata {
                fn new() -> Self {
                    Self { value: 42 }
                }
            }

            // This is the limitation of the current API:
            // A different type with the same layout can be
            // used -- serde just doens't care.
            #[derive(
                serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, tskit::metadata::$md,
            )]
            #[serializer("serde_json")]
            struct SameMetadataLayoutDifferentType {
                value: i32,
            }

            #[derive(
                serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, tskit::metadata::$md,
            )]
            #[serializer("serde_json")]
            struct InvalidMetadataType {
                value: String,
            }
        };
    }

    macro_rules! match_block_impl {
        ($tables: ident, $table: ident, $adder: ident, $row: ident, $md: ident) => {
            match $row {
                Ok(id) => {
                    assert_eq!($tables.$table().num_rows(), 1);

                    match $tables.$table().row(id) {
                        Some(row) => {
                            assert!(row.metadata.is_some());
                            match $tables.$table().row_view(id) {
                                Some(view) => assert_eq!(row, view),
                                None => panic!("if there is a row, there must be a view!"),
                            }
                        }
                        None => panic!("Expected Some(row) from {} table", stringify!(table)),
                    }

                    match $tables.$table().metadata::<MyMetadata>(id) {
                        Some(Ok(value)) => assert_eq!(value, $md),
                        _ => panic!(
                            "expected Some(Ok(_)) from tables.{}()::metadata::<_>(row)",
                            stringify!($table)
                        ),
                    }

                    match $tables.$table().metadata::<InvalidMetadataType>(id) {
                        Some(Err(_)) => (),
                        _ => panic!(
                            "expected Some(Err(_)) from tables.{}()::metadata::<_>(row)",
                            stringify!($table)
                        ),
                    }

                    // This is the limitation:
                    // We can record type A as metadata and get type B
                    // back out so long as A and B (de)serialize the same way.
                    match $tables
                        .$table()
                        .metadata::<SameMetadataLayoutDifferentType>(id)
                    {
                        Some(Ok(_)) => (),
                        _ => panic!(
                            "expected Some(Ok(_)) from tables.{}()::metadata::<_>(row)",
                            stringify!($table)
                        ),
                    }
                }
                Err(e) => panic!("Err from tables.{}: {:?}", stringify!(adder), e),
            }
        };
    }

    macro_rules! add_row_with_metadata {
        ($table: ident, $adder: ident, $md: ident) => {{
            {
                use tskit::prelude::*;
                use tskit::metadata::MetadataRoundtrip;
                build_metadata_types!($md);
                let mut tables = tskit::TableCollection::new(10.).unwrap();
                let md = MyMetadata::new();
                let row = tables.$adder(&md);
                match_block_impl!(tables, $table, $adder, row, md);
                let mut lending_iter = tables.$table().lending_iter();
                let mut iter = tables.$table().iter();
                while let Some(row) = lending_iter.next() {
                    if let Some(row_from_iter) = iter.next() {
                        assert_eq!(row, &row_from_iter);
                        assert_eq!(&row_from_iter, row);
                    }
                    if let Some(metadata) = row.metadata {
                        assert_eq!(MyMetadata::decode(metadata).unwrap(), md);
                    }else {
                        panic!("expected Some(metadata)");
                    }
                }
            }
        }};
        ($table: ident, $adder: ident, $md: ident $(,$payload: expr) + ) => {{
            {
                use tskit::prelude::*;
                use tskit::metadata::MetadataRoundtrip;
                build_metadata_types!($md);
                let mut tables = tskit::TableCollection::new(10.).unwrap();
                let md = MyMetadata::new();
                let row =  tables.$adder($($payload ), *, &md);
                match_block_impl!(tables, $table, $adder, row, md);
                let mut lending_iter = tables.$table().lending_iter();
                let mut iter = tables.$table().iter();
                while let Some(row) = lending_iter.next() {
                    if let Some(row_from_iter) = iter.next() {
                        assert_eq!(row, &row_from_iter);
                        assert_eq!(&row_from_iter, row);
                    }
                    if let Some(metadata) = row.metadata {
                        assert_eq!(MyMetadata::decode(metadata).unwrap(), md);
                    }else {
                        panic!("expected Some(metadata)");
                    }
                }
            }
        }};
    }

    #[test]
    fn test_edge_metadata() {
        add_row_with_metadata!(edges, add_edge_with_metadata, EdgeMetadata, 0.1, 0.5, 0, 1);
    }

    #[test]
    fn test_adding_node() {
        add_row_with_metadata!(nodes, add_node_with_metadata, NodeMetadata, 0, 0.1, -1, -1);
    }

    #[test]
    fn test_adding_site() {
        add_row_with_metadata!(sites, add_site_with_metadata, SiteMetadata, 2. / 3., None);
    }

    #[test]
    fn test_adding_mutation() {
        add_row_with_metadata!(
            mutations,
            add_mutation_with_metadata,
            MutationMetadata,
            0,
            0,
            -1,
            0.0,
            None
        );
    }
    #[test]
    fn test_adding_individual() {
        add_row_with_metadata!(
            individuals,
            add_individual_with_metadata,
            IndividualMetadata,
            0,
            None,
            None
        );
    }

    #[test]
    fn test_adding_population() {
        add_row_with_metadata!(
            populations,
            add_population_with_metadata,
            PopulationMetadata
        );
    }

    #[test]
    fn test_adding_migration() {
        add_row_with_metadata!(
            migrations,
            add_migration_with_metadata,
            MigrationMetadata,
            (0., 1.),
            0,
            (0, 1),
            0.0
        );
    }
}

#[test]
fn test_node_table_column_access() {
    let mut t = tskit::NodeTable::new().unwrap();
    let node = t
        .add_row(tskit::NodeFlags::new_sample(), 0.0, -1, -1)
        .unwrap();
    {
        let individual = t.individual_column();
        assert_eq!(individual[node], tskit::IndividualId::NULL);
        assert_eq!(
            individual.get_with_id(node).unwrap(),
            &tskit::IndividualId::NULL
        );
        assert!(individual.get_with_size_type(t.num_rows()).is_none());
    }
    {
        let population = t.population_column();
        assert_eq!(population[node], tskit::PopulationId::NULL);
        assert_eq!(
            population.get_with_id(node).unwrap(),
            &tskit::PopulationId::NULL
        );
    }
    {
        let time = t.time_column();
        assert_eq!(time[node], 0.0);
        assert_eq!(time.get_with_id(node).unwrap(), &0.0);
    }
    {
        let flags = t.flags_column();
        assert_eq!(flags[node], tskit::NodeFlags::IS_SAMPLE);
        assert_eq!(
            flags.get_with_id(node).unwrap(),
            &tskit::NodeFlags::IS_SAMPLE
        );
    }
}

#[test]
fn test_edge_table_column_access() {
    let mut table = tskit::EdgeTable::default();
    let edge = table.add_row(0., 10., 1, 0).unwrap();

    {
        let column = table.left_column();
        assert_eq!(column[edge], 0.0);
        assert_eq!(column[edge], tskit::Position::from(0.));
    }

    {
        let column = table.right_column();
        assert_eq!(column[edge], 10.0);
        assert_eq!(column[edge], tskit::Position::from(10.0));
    }

    {
        let column = table.parent_column();
        assert_eq!(column[edge], 1);
        assert_eq!(column[edge], tskit::NodeId::from(1));
        let _: Vec<tskit::NodeId> = table.parent_column().iter().cloned().collect::<Vec<_>>();
        let _ = table.parent_column().iter().cloned().collect::<Vec<_>>();
    }

    {
        let column = table.child_column();
        assert_eq!(column[edge], 0);
        assert_eq!(column[edge], tskit::NodeId::from(0));
    }
}
