use tskit::metadata;
use tskit::TableAccess;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Mutation {
    pub effect_size: f64,
    pub dominance: f64,
    pub origin_time: i32,
}

// Implement the metadata trait for our mutation
// type.  Will will use the standard library for the implementation
// details.
impl metadata::MetadataRoundtrip for Mutation {
    fn encode(&self) -> Result<Vec<u8>, metadata::MetadataError> {
        match bincode::serialize(&self) {
            Ok(v) => Ok(v),
            Err(e) => Err(crate::metadata::MetadataError::RoundtripError { value: Box::new(e) }),
        }
    }

    fn decode(md: &[u8]) -> Result<Self, metadata::MetadataError> {
        match bincode::deserialize(md) {
            Ok(x) => Ok(x),
            Err(e) => Err(crate::metadata::MetadataError::RoundtripError { value: Box::new(e) }),
        }
    }
}

pub fn run() {
    let mut tables = tskit::TableCollection::new(1000.).unwrap();
    // The simulation generates a mutation:
    let m = Mutation {
        effect_size: -0.235423,
        dominance: 0.5,
        origin_time: 1,
    };

    // The mutation's data are included as metadata:
    tables
        .add_mutation_with_metadata(0, 0, 0, 0.0, None, Some(&m))
        .unwrap();

    // Decoding requres 2 unwraps:
    // 1. The first is to handle errors.
    // 2. The second is b/c metadata are optional,
    //    so a row may return None.
    let decoded = tables.mutations().metadata::<Mutation>(0).unwrap().unwrap();

    // Check that we've made the round trip:
    assert_eq!(decoded.origin_time, 1);
    assert!((m.effect_size - decoded.effect_size).abs() < f64::EPSILON);
    assert!((m.dominance - decoded.dominance).abs() < f64::EPSILON);
}

#[test]
fn run_test() {
    run();
}

fn main() {
    run();
}
