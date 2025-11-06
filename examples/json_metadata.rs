#[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
#[serializer("serde_json")]
struct MutationMetadata {
    effect_size: f64,
    dominance: f64,
}

#[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
#[serializer("serde_json")]
struct IndividualMetadata {
    name: String,
    phenotypes: Vec<i32>,
}

fn main() {
    let ts = make_treeseq().unwrap();
    ts.dump("with_json_metadata.trees", 0).unwrap();
}

fn make_tables() -> Result<tskit::TableCollection, tskit::TskitError> {
    let mut tables = tskit::TableCollection::new(100.0)?;
    let pop0 = tables.add_population()?;
    let ind0 = tables.add_individual_with_metadata(
        0,
        None,
        None,
        &IndividualMetadata {
            name: "Jerome".to_string(),
            phenotypes: vec![0, 1, 2, 0],
        },
    )?;
    let node0 = tables.add_node(tskit::NodeFlags::new_sample(), 0.0, pop0, ind0)?;
    let site0 = tables.add_site(50.0, Some("A".as_bytes()))?;
    let _ = tables.add_mutation_with_metadata(
        site0,
        node0,
        tskit::MutationId::NULL,
        1.0,
        Some("G".as_bytes()),
        &MutationMetadata {
            effect_size: -1e-3,
            dominance: 0.1,
        },
    )?;
    tables.build_index()?;
    Ok(tables)
}

fn make_treeseq() -> Result<tskit::TreeSequence, tskit::TskitError> {
    make_tables()?.tree_sequence(0)
}
