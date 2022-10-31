#[test]
fn test_empty_table_collection() {
    use tskit::TableAccess;

    let tables = tskit::TableCollection::new(10.).unwrap();

    assert!(tables.edges().row(-1).is_none());
    assert!(tables.edges().row(0).is_none());
    assert!(tables.nodes().row(-1).is_none());
    assert!(tables.nodes().row(0).is_none());
    assert!(tables.sites().row(-1).is_none());
    assert!(tables.sites().row(0).is_none());
    assert!(tables.mutations().row(-1).is_none());
    assert!(tables.mutations().row(0).is_none());
    assert!(tables.individuals().row(-1).is_none());
    assert!(tables.individuals().row(0).is_none());
    assert!(tables.populations().row(-1).is_none());
    assert!(tables.populations().row(0).is_none());
    assert!(tables.migrations().row(-1).is_none());
    assert!(tables.migrations().row(0).is_none());

    #[cfg(feature = "provenance")]
    {
        assert!(tables.provenances().row(-1).is_none());
        assert!(tables.provenances().row(0).is_none());
    }
}
