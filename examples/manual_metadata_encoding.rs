use core::str;

use tskit::metadata::MetadataRoundtrip;

struct MutationMetadata {
    effect_size: f64,
    dominance: f64,
}

impl MetadataRoundtrip for MutationMetadata {
    fn encode(&self) -> Result<Vec<u8>, tskit::metadata::MetadataError> {
        let mut rv = vec![];
        rv.extend_from_slice(&self.effect_size.to_le_bytes());
        rv.extend_from_slice(&self.dominance.to_le_bytes());
        Ok(rv)
    }

    fn decode(md: &[u8]) -> Result<Self, tskit::metadata::MetadataError>
    where
        Self: Sized,
    {
        let slice: [u8; 8] = md[0..8].try_into().unwrap();
        let effect_size = f64::from_le_bytes(slice);
        let slice: [u8; 8] = md[8..].try_into().unwrap();
        let dominance = f64::from_le_bytes(slice);
        Ok(Self {
            effect_size,
            dominance,
        })
    }
}

impl tskit::metadata::MutationMetadata for MutationMetadata {}

struct IndividualMetadata {
    name: String,
    phenotypes: Vec<i32>,
}

impl MetadataRoundtrip for IndividualMetadata {
    fn encode(&self) -> Result<Vec<u8>, tskit::metadata::MetadataError> {
        let mut rv = vec![];
        rv.extend_from_slice(self.name.len().to_le_bytes().as_slice());
        rv.extend_from_slice(self.name.as_bytes());
        rv.extend_from_slice(self.phenotypes.len().to_le_bytes().as_slice());
        for i in self.phenotypes.iter() {
            rv.extend_from_slice(i.to_le_bytes().as_slice());
        }
        Ok(rv)
    }
    fn decode(md: &[u8]) -> Result<Self, tskit::metadata::MetadataError>
    where
        Self: Sized,
    {
        let size: [u8; std::mem::size_of::<usize>()] =
            md[0..std::mem::size_of::<usize>()].try_into().unwrap();
        let size = usize::from_le_bytes(size);
        let md = &md[std::mem::size_of::<usize>()..];
        let name = str::from_utf8(&md[0..size]).unwrap().to_string();
        let md = &md[size..];
        let md = &md[std::mem::size_of::<usize>()..];
        let mut phenotypes = vec![];
        // NOTE: production code would want to validate that
        // the remaining number of bytes are correct
        let chunks = md.chunks_exact(std::mem::size_of::<i32>());
        for c in chunks {
            // Unwrap b/c the conversion cannot fail b/c the chunk size is correct!
            let a: [u8; std::mem::size_of::<i32>()] = c.try_into().unwrap();
            phenotypes.push(i32::from_le_bytes(a));
        }
        Ok(Self { name, phenotypes })
    }
}

impl tskit::metadata::IndividualMetadata for IndividualMetadata {}

fn main() {
    let ts = make_treeseq().unwrap();
    ts.dump("with_manual_metadata.trees", 0).unwrap();
}

fn make_tables() -> anyhow::Result<tskit::TableCollection> {
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

fn make_treeseq() -> anyhow::Result<tskit::TreeSequence> {
    Ok(make_tables()?.tree_sequence(0)?)
}

#[test]
fn test_mutation_metadata_roundtrip() {
    let md = MutationMetadata {
        effect_size: 0.1,
        dominance: 0.25,
    };
    let encoded = md.encode().unwrap();
    let decoded = MutationMetadata::decode(&encoded).unwrap();
    assert_eq!(md.effect_size, decoded.effect_size);
    assert_eq!(md.dominance, decoded.dominance);
}

#[test]
fn test_individual_metadata_roundtrip() {
    let md = IndividualMetadata {
        name: "Jerome".to_string(),
        phenotypes: vec![10, 9],
    };
    let encoded = md.encode().unwrap();
    let decoded = IndividualMetadata::decode(&encoded).unwrap();
    assert_eq!(md.name, decoded.name);
    assert_eq!(md.phenotypes, decoded.phenotypes);
}
