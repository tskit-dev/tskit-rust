use tskit_rust::metadata;
use tskit_rust::*;
mod mutation;

use mutation::Mutation;

impl metadata::MetadataRoundtrip for Mutation {
    fn encode(&self) -> Result<Vec<u8>, metadata::MetadataError> {
        let mut rv = vec![];
        rv.extend(self.effect_size.to_le_bytes().iter().copied());
        rv.extend(self.dominance.to_le_bytes().iter().copied());
        rv.extend(self.origin_time.to_le_bytes().iter().copied());
        Ok(rv)
    }

    fn decode(md: &[u8]) -> Result<Self, metadata::MetadataError> {
        use std::convert::TryInto;
        let (effect_size_bytes, rest) = md.split_at(std::mem::size_of::<f64>());
        let (dominance_bytes, rest) = rest.split_at(std::mem::size_of::<f64>());
        let (origin_time_bytes, _) = rest.split_at(std::mem::size_of::<i32>());
        Ok(Self {
            effect_size: f64::from_le_bytes(effect_size_bytes.try_into().unwrap()),
            dominance: f64::from_le_bytes(dominance_bytes.try_into().unwrap()),
            origin_time: i32::from_le_bytes(origin_time_bytes.try_into().unwrap()),
        })
    }
}

make_mutation_metadata_run!();

#[test]
fn run_bincode() {
    run();
}

fn main() {
    run();
}
