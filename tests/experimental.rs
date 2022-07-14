#![cfg(feature = "derive")]

mod experimental_features {

    use tskit::TableAccess;

    // Name is not great.
    // We'd like to have this be : tskit::TableAccess,
    // but that's a big ask at this stage.
    trait MutationMetadataExtraction {
        type Item: tskit::metadata::MutationMetadata;

        fn get_mutation_metadata<M: Into<tskit::MutationId>>(
            &self,
            row: M,
        ) -> Result<Option<Self::Item>, tskit::TskitError>;
    }

    #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    #[serializer("serde_json")]
    struct MutationMetadataType {
        effect_size: f64,
    }

    // Goal:
    //
    // A table newtype can let us define tables in terms
    // of their metadata.
    // If we want a table collection that only uses
    // MutationMetadata, then the traits defined here
    // suffice.

    struct MyTableCollection(tskit::TableCollection);

    // Deref'ing for pure convenience
    impl std::ops::Deref for MyTableCollection {
        type Target = tskit::TableCollection;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::DerefMut for MyTableCollection {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl MutationMetadataExtraction for MyTableCollection {
        type Item = MutationMetadataType;
        fn get_mutation_metadata<M: Into<tskit::MutationId>>(
            &self,
            row: M,
        ) -> Result<Option<Self::Item>, tskit::TskitError> {
            self.mutations().metadata::<Self::Item>(row.into())
        }
    }

    #[test]
    fn test_table_collection_newtype() {
        let mut tables = MyTableCollection(tskit::TableCollection::new(1.0).unwrap());
        let md = MutationMetadataType { effect_size: 0.1 };
        tables
            .add_mutation_with_metadata(0, 0, 0, 0.0, None, &md)
            .unwrap();
        let decoded = tables.get_mutation_metadata(0).unwrap().unwrap();
        assert_eq!(decoded.effect_size, 0.10);

        // current API requires
        let decoded = tables
            .mutations()
            .metadata::<MutationMetadataType>(0.into())
            .unwrap()
            .unwrap();
        assert_eq!(decoded.effect_size, 0.10);
    }
}

mod experimental_features_refined {

    // To make further progress from here:
    // AsTables becomes a trait in tskit.
    // Given that, a blanket implementation
    // for TableAccess is trivial boilerplate.
    // Given those two things:
    // * Can define these metadata-centric traits.
    // * proc-macro can implement them "easily"
    //
    // Caveats:
    //
    // * How to apply the same logic to TreeSequence?
    //   - There is currently no .tables() function there.
    //   - but it does implement TableAccess.
    //   - Hmmm...proc macros to the rescue?
    //
    // * On the surface it looks "tempting" to allow
    //   a non-owning TableCollection to hold, but never
    //   Drop, a raw pointer.  But that kinda thing
    //   seems very un-rusty.
    //
    // I may have the logic backwards:
    //
    // * We want to constrain on a trait that it itself
    //   constrained by TableAccess.
    // * What kind of proc-macro can we use to implement that,
    //   again "easily"?

    use tskit::TableAccess;

    trait AsTableCollection {
        fn as_tables(&self) -> &tskit::TableCollection;
    }

    // Name is not great.
    // We'd like to have this be : tskit::TableAccess,
    // but that's a big ask at this stage.
    // See notes above.
    trait MutationMetadataExtraction: AsTableCollection {
        type Item: tskit::metadata::MutationMetadata;

        fn get_mutation_metadata<M: Into<tskit::MutationId>>(
            &self,
            row: M,
        ) -> Result<Option<Self::Item>, tskit::TskitError> {
            self.as_tables()
                .mutations()
                .metadata::<Self::Item>(row.into())
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    #[serializer("serde_json")]
    struct MutationMetadataType {
        effect_size: f64,
    }

    // Goal:
    //
    // A table newtype can let us define tables in terms
    // of their metadata.
    // If we want a table collection that only uses
    // MutationMetadata, then the traits defined here
    // suffice.

    struct MyTableCollection(tskit::TableCollection);

    impl AsTableCollection for MyTableCollection {
        fn as_tables(&self) -> &tskit::TableCollection {
            &self.0
        }
    }

    impl MutationMetadataExtraction for MyTableCollection {
        type Item = MutationMetadataType;
    }

    #[test]
    fn test_table_collection_newtype() {
        let mut tables = MyTableCollection(tskit::TableCollection::new(1.0).unwrap());
        let md = MutationMetadataType { effect_size: 0.1 };
        tables
            .0
            .add_mutation_with_metadata(0, 0, 0, 0.0, None, &md)
            .unwrap();
        let decoded = tables.get_mutation_metadata(0).unwrap().unwrap();
        assert_eq!(decoded.effect_size, 0.10);

        // current API requires
        let decoded = tables
            .0
            .mutations()
            .metadata::<MutationMetadataType>(0.into())
            .unwrap()
            .unwrap();
        assert_eq!(decoded.effect_size, 0.10);
    }
}
