use tskit_rust::metadata;
use tskit_rust::*;
mod mutation;

use mutation::Mutation;

// Implement the metadata trait for our mutation
// type.  Will will use the standard library for the implementation
// details.
impl metadata::MetadataRoundtrip for Mutation {
    fn encode(&self) -> Result<Vec<u8>, metadata::MetadataError> {
        Ok(bincode::serialize(&self).unwrap())
    }

    fn decode(md: &[u8]) -> Result<Self, metadata::MetadataError> {
        Ok(bincode::deserialize(md).unwrap())
    }
}

make_mutation_metadata_run!();

#[test]
fn run_std() {
    run();
}

fn main() {
    run();
}
