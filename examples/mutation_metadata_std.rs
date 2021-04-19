use tskit::metadata;
use tskit::TableAccess;

pub struct Mutation {
    pub effect_size: f64,
    pub dominance: f64,
    pub origin_time: i32,
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

#[test]
fn run_test() {
    run();
}

fn main() {
    run();
}
