#[test]
fn test_empty_table_collection() {
    macro_rules! validate_empty_tables {
        ($tables: ident, $table: ident, $table_iter: ident, $row: expr) => {
            assert!($tables.$table().row($row).is_none());
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
            }
        }};
    }

    // NOTE: all functions arguments for adding rows are Into<T>
    // where T is one of our new types.
    // Further, functions taking multiple inputs of T are defined
    // as X: Into<T>, X2: Into<T>, etc., allowing mix-and-match.

    #[test]
    fn test_adding_edge() {
        add_row_without_metadata!(edges, add_edge, 0.1, 0.5, 0, 1); // left, right, parent, child
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
        add_row_without_metadata!(nodes, add_node, 0, 0.1, -1, -1); // flags, time, population,
                                                                    // individual
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
        add_row_without_metadata!(sites, add_site, 2. / 3., None);
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
        add_row_without_metadata!(mutations, add_mutation, 0, 0, -1, 0.0, None);
        add_row_without_metadata!(mutations, add_mutation, 0, 0, -1, 0.0, Some(&[23_u8]));
    }

    #[test]
    fn test_adding_individual() {
        // flags, location, parents
        add_row_without_metadata!(individuals, add_individual, 0, None, None);
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
        add_row_without_metadata!(migrations, add_migration, (0., 1.), 0, (0, 1), 0.0);
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
                        Some(row) => assert!(row.metadata.is_some()),
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
                build_metadata_types!($md);
                let mut tables = tskit::TableCollection::new(10.).unwrap();
                let md = MyMetadata::new();
                let row = tables.$adder(&md);
                match_block_impl!(tables, $table, $adder, row, md)
            }
        }};
        ($table: ident, $adder: ident, $md: ident $(,$payload: expr) + ) => {{
            {
                build_metadata_types!($md);
                let mut tables = tskit::TableCollection::new(10.).unwrap();
                let md = MyMetadata::new();
                let row =  tables.$adder($($payload ), *, &md);
                match_block_impl!(tables, $table, $adder, row, md);
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
