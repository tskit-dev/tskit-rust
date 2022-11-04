#![cfg(feature = "derive")]

mod experimental_features {

    // Goal: proc macro this up
    // Design notes for future:
    // * We can probably drop this trait.
    // * We want a derive macro based on XMetadataRetrieval,
    //   where X is a table type.
    // * So hopefully we can start there.
    trait MetadataRetrieval<R> {
        type Metadata: tskit::metadata::MetadataRoundtrip;
        fn metadata(&self, r: impl Into<R>) -> Option<Result<Self::Metadata, tskit::TskitError>>;
    }

    // Specific traits cover the various row id types
    trait MutationMetadataRetrieval: MetadataRetrieval<tskit::MutationId> {
        fn mutation_metadata(
            &self,
            row: impl Into<tskit::MutationId>,
        ) -> Option<
            Result<<Self as MetadataRetrieval<tskit::MutationId>>::Metadata, tskit::TskitError>,
        >
        where
            <Self as MetadataRetrieval<tskit::MutationId>>::Metadata:
                tskit::metadata::MutationMetadata;
    }

    // Blanket implementations are possible given the above
    // two defnitions, putting all boiler plate out of sight!
    impl<T> MutationMetadataRetrieval for T
    where
        T: MetadataRetrieval<tskit::MutationId>,
        <Self as MetadataRetrieval<tskit::MutationId>>::Metadata: tskit::metadata::MutationMetadata,
    {
        fn mutation_metadata(
            &self,
            row: impl Into<tskit::MutationId>,
        ) -> Option<
            Result<<Self as MetadataRetrieval<tskit::MutationId>>::Metadata, tskit::TskitError>,
        > {
            self.metadata(row)
        }
    }

    trait IndividualMetadataRetrieval: MetadataRetrieval<tskit::IndividualId> {
        fn individual_metadata(
            &self,
            row: impl Into<tskit::IndividualId>,
        ) -> Option<
            Result<<Self as MetadataRetrieval<tskit::IndividualId>>::Metadata, tskit::TskitError>,
        >
        where
            <Self as MetadataRetrieval<tskit::IndividualId>>::Metadata:
                tskit::metadata::MutationMetadata;
    }

    impl<T> IndividualMetadataRetrieval for T
    where
        T: MetadataRetrieval<tskit::IndividualId>,
        <Self as MetadataRetrieval<tskit::IndividualId>>::Metadata:
            tskit::metadata::IndividualMetadata,
    {
        fn individual_metadata(
            &self,
            row: impl Into<tskit::IndividualId>,
        ) -> Option<
            Result<<Self as MetadataRetrieval<tskit::IndividualId>>::Metadata, tskit::TskitError>,
        > {
            self.metadata(row)
        }
    }

    trait MutationMetadataExtraction {
        type Item: tskit::metadata::MutationMetadata;

        fn get_mutation_metadata<M: Into<tskit::MutationId>>(
            &self,
            row: M,
        ) -> Option<Result<Self::Item, tskit::TskitError>>;
    }

    #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    #[serializer("serde_json")]
    struct MutationMetadataType {
        effect_size: f64,
    }

    #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    #[serializer("serde_json")]
    struct IndividualMetadataType {
        fitness: f64,
        location: [f64; 3],
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
        ) -> Option<Result<Self::Item, tskit::TskitError>> {
            self.mutations().metadata::<Self::Item>(row.into())
        }
    }

    impl MetadataRetrieval<tskit::MutationId> for MyTableCollection {
        type Metadata = MutationMetadataType;
        fn metadata(
            &self,
            row: impl Into<tskit::MutationId>,
        ) -> Option<Result<MutationMetadataType, tskit::TskitError>> {
            self.mutations()
                .metadata::<MutationMetadataType>(row.into())
        }
    }

    impl MetadataRetrieval<tskit::IndividualId> for MyTableCollection {
        type Metadata = IndividualMetadataType;
        fn metadata(
            &self,
            row: impl Into<tskit::IndividualId>,
        ) -> Option<Result<IndividualMetadataType, tskit::TskitError>> {
            self.individuals()
                .metadata::<IndividualMetadataType>(row.into())
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

        // More ergonomic here...
        // NOTE: this can no longer compile b/c we've
        // got the pattern in place for > 1 trait.
        // let decoded = tables.metadata(0).unwrap().unwrap();
        // assert_eq!(decoded.effect_size, 0.10);

        // ...but not here, which is how it would normally be called...
        let decoded =
            <MyTableCollection as MetadataRetrieval<tskit::MutationId>>::metadata(&tables, 0)
                .unwrap()
                .unwrap();
        assert_eq!(decoded.effect_size, 0.10);

        // ... but blanket impl may be a path to glory.
        let decoded = tables.mutation_metadata(0).unwrap().unwrap();
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

    trait AsTableCollection {
        fn as_tables(&self) -> &tskit::TableCollection;
    }

    // Name is not great.
    // See notes above.
    trait MutationMetadataExtraction: AsTableCollection {
        type Item: tskit::metadata::MutationMetadata;

        fn get_mutation_metadata<M: Into<tskit::MutationId>>(
            &self,
            row: M,
        ) -> Option<Result<Self::Item, tskit::TskitError>> {
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
