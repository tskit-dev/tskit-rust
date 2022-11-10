#[cfg(feature = "derive")]
#[test]
fn book_mutation_metadata() {
    use streaming_iterator::StreamingIterator;
    use tskit::metadata::MetadataRoundtrip;

    // ANCHOR: metadata_derive
    #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    #[serializer("serde_json")]
    struct MutationMetadata {
        effect_size: f64,
        dominance: f64,
    }
    // ANCHOR_END: metadata_derive

    // ANCHOR: add_mutation_table_row_with_metadata
    let mut tables = tskit::TableCollection::new(50.0).unwrap();

    let md = MutationMetadata {
        effect_size: 1e-3,
        dominance: 1.0,
    };

    let mut_id_0 = tables
        .add_mutation_with_metadata(
            0,    // site id
            0,    // node id
            -1,   // mutation parent id
            0.0,  // time
            None, // derived state is Option<&[u8]>
            &md,  // metadata for this row
        )
        .unwrap();
    // ANCHOR_END: add_mutation_table_row_with_metadata

    // ANCHOR: add_mutation_table_row_without_metadata
    let mut_id_1 = tables
        .add_mutation(
            0,    // site id
            0,    // node id
            -1,   // mutation parent id
            0.0,  // time
            None, // derived state is Option<&[u8]>
        )
        .unwrap();
    // ANCHOR_END: add_mutation_table_row_without_metadata

    // ANCHOR: validate_metadata_row_contents
    assert_eq!(
        tables
            .mutations_iter()
            .filter(|m| m.metadata.is_some())
            .count(),
        1
    );
    assert_eq!(
        tables
            .mutations_iter()
            .filter(|m| m.metadata.is_none())
            .count(),
        1
    );
    // ANCHOR_END: validate_metadata_row_contents

    // ANCHOR: metadata_retrieval
    let fetched_md = match tables.mutations().metadata::<MutationMetadata>(mut_id_0) {
        Some(Ok(m)) => m,
        Some(Err(e)) => panic!("metadata decoding failed: {:?}", e),
        None => panic!(
            "hmmm...row {} should have been a valid row with metadata...",
            mut_id_0
        ),
    };

    assert_eq!(md.effect_size, fetched_md.effect_size);
    assert_eq!(md.dominance, fetched_md.dominance);
    // ANCHOR_END: metadata_retrieval

    // ANCHOR: metadata_retrieval_none
    // There is no metadata at row 1, so
    // you get None back
    assert!(tables
        .mutations()
        .metadata::<MutationMetadata>(mut_id_1)
        .is_none());

    // There is also no metadata at row 2,
    // because that row does not exist, so
    // you get None back
    assert!(tables
        .mutations()
        .metadata::<MutationMetadata>(2.into())
        .is_none());
    // ANCHOR_END: metadata_retrieval_none

    // ANCHOR: metadata_bulk_decode_lending_iter
    let mut mutation_row_lending_iterator = tables.mutations().lending_iter();
    let mut decoded_md = vec![];
    while let Some(row_view) = mutation_row_lending_iterator.next() {
        match row_view.metadata {
            Some(slice) => decoded_md.push(Some(MutationMetadata::decode(slice).unwrap())),
            None => decoded_md.push(None),
        }
    }
    // ANCHOR_END: metadata_bulk_decode_lending_iter

    // ANCHOR: metadata_bulk_decode_lending_iter_with_filter
    let mut mutation_row_lending_iterator = tables.mutations().lending_iter();
    let mut decoded_md = vec![];
    while let Some(row_view) = mutation_row_lending_iterator
        .next()
        .filter(|rv| rv.metadata.is_some())
    {
        decoded_md.push((
            row_view.id,
            // The unwrap will never panic because of our filter
            MutationMetadata::decode(row_view.metadata.unwrap()).unwrap(),
        ));
    }
    // ANCHOR_END: metadata_bulk_decode_lending_iter_with_filter
}
