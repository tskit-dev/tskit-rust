#[cfg(feature = "derive")]
#[cfg(test)]
macro_rules! build_metadata_registration_test {
    ($modname: ident, $testname: ident, $serializer: tt, $structname: ty, $metadata_marker: ident) => {
        #[cfg(test)]
        mod $modname {

            #[derive(
                Copy,
                Clone,
                Debug,
                Eq,
                PartialEq,
                Ord,
                PartialOrd,
                serde::Serialize,
                serde::Deserialize,
            )]
            struct N(i32);

            #[derive(
                Copy, Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
            )]
            struct F(f64);

            #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::$metadata_marker)]
            #[serializer($serializer)]
            struct GenericMetadata {
                a: N,
                b: Vec<F>,
            }

            impl Default for GenericMetadata {
                fn default() -> Self {
                    GenericMetadata {
                        a: N(-1),
                        b: vec![F(1.0), F(-2.0)],
                    }
                }
            }

            fn dispatch_metadata<T>(_: &T)
            where
                T: tskit::metadata::$metadata_marker,
            {
            }

            #[test]
            fn $testname() {
                dispatch_metadata(&<$structname>::default());
            }

            #[test]
            fn test_roundtrip() {
                use tskit::metadata::MetadataRoundtrip;
                let d = GenericMetadata::default();
                let encoded = d.encode().unwrap();
                let decoded = GenericMetadata::decode(&encoded).unwrap();
                assert_eq!(d.a, decoded.a);
                assert_eq!(d.b.len(), decoded.b.len());
                for (i, j) in d.b.iter().zip(decoded.b.iter()) {
                    match i.partial_cmp(j) {
                        Some(std::cmp::Ordering::Equal) => (),
                        Some(std::cmp::Ordering::Less) => panic!("expected Equal"),
                        Some(std::cmp::Ordering::Greater) => panic!("expected Equal"),
                        None => panic!("expected Equal"),
                    };
                }
            }
        }
    };
}

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_metadata_registration,
    test_register_mutation_metadata,
    "serde_json",
    GenericMetadata,
    MutationMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_node_registration,
    test_register_node_metadata,
    "serde_json",
    GenericMetadata,
    NodeMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_site_registration,
    test_register_site_metadata,
    "serde_json",
    GenericMetadata,
    SiteMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_edge_registration,
    test_register_edge_metadata,
    "serde_json",
    GenericMetadata,
    EdgeMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_individual_registration,
    test_register_individual_metadata,
    "serde_json",
    GenericMetadata,
    IndividualMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_migration_registration,
    test_register_migration_metadata,
    "serde_json",
    GenericMetadata,
    MigrationMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_json_population_registration,
    test_register_population_metadata,
    "serde_json",
    GenericMetadata,
    PopulationMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_metadata_registration,
    test_register_mutation_metadata,
    "bincode",
    GenericMetadata,
    MutationMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_node_registration,
    test_register_node_metadata,
    "bincode",
    GenericMetadata,
    NodeMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_site_registration,
    test_register_site_metadata,
    "bincode",
    GenericMetadata,
    SiteMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_edge_registration,
    test_register_edge_metadata,
    "bincode",
    GenericMetadata,
    EdgeMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_individual_registration,
    test_register_individual_metadata,
    "bincode",
    GenericMetadata,
    IndividualMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_migration_registration,
    test_register_migration_metadata,
    "bincode",
    GenericMetadata,
    MigrationMetadata
);

#[cfg(feature = "derive")]
build_metadata_registration_test!(
    test_bincode_population_registration,
    test_register_population_metadata,
    "bincode",
    GenericMetadata,
    PopulationMetadata
);
