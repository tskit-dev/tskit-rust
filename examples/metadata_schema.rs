use tskit::prelude::*;
use tskit::TableCollection;

#[derive(serde::Serialize, serde::Deserialize, tskit::metadata::PopulationMetadata)]
#[serializer("serde_json")]
struct PopulationMetadata {
    name: String,
}

fn main() {
    let from_fp11 = r#"
    {
        "codec": "json",
        "type": "object",
        "name": "Population metadata",
        "properties": {"name": {"type": "string"}}
    }"#;

    let mut tables = TableCollection::new(10.0).unwrap();
    tables
        .add_population_with_metadata(&PopulationMetadata {
            name: "YRB".to_string(),
        })
        .unwrap();
    tables
        .set_json_metadata_schema_from_str(tskit::MetadataSchema::Populations, from_fp11)
        .unwrap();
    tables.dump("testit.trees", 0).unwrap();
}
