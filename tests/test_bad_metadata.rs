#[path = "./test_fixtures.rs"]
mod test_fixtures;

use test_fixtures::bad_metadata::*;
use tskit::MutationId;

#[test]
fn test_bad_mutation_metadata_roundtrip() {
    let mut tables = tskit::TableCollection::new(1.).unwrap();
    let md = F { x: 1, y: 11 };
    tables
        .add_mutation_with_metadata(0, 0, MutationId::NULL, 0.0, None, &md)
        .unwrap();
    if tables.mutations().metadata::<Ff>(0.into()).is_ok() {
        panic!("expected an error!!");
    }
}
