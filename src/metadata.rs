//! Support for table row metadata

use crate::bindings::{tsk_id_t, tsk_size_t};
use thiserror::Error;

/// Enable a type to be used as table metadata
///
/// See [`handle_metadata_return`] for a macro to help implement this trait,
/// and its use in examples below.
///
/// We strongly recommend the use of the [serde](https://serde.rs/) ecosystem
/// for row metadata.
/// For many use cases, we imagine that
/// [bincode](https://crates.io/crates/bincode) will be one of
/// the more useful `serde`-related crates.
///
/// The library provides two macros to facilitate implementing metadata
/// traits:
///
/// * [`serde_json_metadata`]
/// * [`serde_bincode_metadata`]
///
/// These macros are optional features.
/// The feature names are the same as the macro names
///
#[cfg_attr(
    feature = "provenance",
    doc = r##"
# Examples

## Mutation metadata encoded as JSON

```
use tskit::handle_metadata_return;
use tskit::TableAccess;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyMutation {
    origin_time: i32,
    effect_size: f64,
    dominance: f64,
}

// Implement tskit::metadata::MetadataRoundtrip
tskit::serde_json_metadata!(MyMutation);

impl tskit::metadata::MutationMetadata for MyMutation {}

let mut tables = tskit::TableCollection::new(100.).unwrap();
let mutation = MyMutation{origin_time: 100,
    effect_size: -1e-4,
    dominance: 0.25};

// Add table row with metadata.
tables.add_mutation_with_metadata(0, 0, tskit::MutationId::NULL, 100., None,
    &mutation).unwrap();

// Decode the metadata
// The two unwraps are:
// 1. Handle Errors vs Option.
// 2. Handle the option for the case of no error.
//
// The .into() reflects the fact that metadata fetching
// functions only take a strong ID type, and tskit-rust
// adds Into<strong ID type> for i32 for all strong ID types.

let decoded = tables.mutations().metadata::<MyMutation>(0.into()).unwrap().unwrap();
assert_eq!(mutation.origin_time, decoded.origin_time);
match decoded.effect_size.partial_cmp(&mutation.effect_size) {
    Some(std::cmp::Ordering::Greater) => assert!(false),
    Some(std::cmp::Ordering::Less) => assert!(false),
    Some(std::cmp::Ordering::Equal) => (),
    None => panic!("bad comparison"),
};
match decoded.dominance.partial_cmp(&mutation.dominance) {
    Some(std::cmp::Ordering::Greater) => assert!(false),
    Some(std::cmp::Ordering::Less) => assert!(false),
    Some(std::cmp::Ordering::Equal) => (),
    None => panic!("bad comparison"),
};
```
"##
)]
pub trait MetadataRoundtrip {
    fn encode(&self) -> Result<Vec<u8>, MetadataError>;
    fn decode(md: &[u8]) -> Result<Self, MetadataError>
    where
        Self: Sized;
}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the mutation table of a [`TableCollection`](crate::TableCollection).
pub trait MutationMetadata: MetadataRoundtrip {}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the node table of a [`TableCollection`](crate::TableCollection).
pub trait NodeMetadata: MetadataRoundtrip {}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the edge table of a [`TableCollection`](crate::TableCollection).
pub trait EdgeMetadata: MetadataRoundtrip {}
///
/// Marker trait indicating [`MetadataRoundtrip`]
/// for the migratoin table of a [`TableCollection`](crate::TableCollection).
pub trait MigrationMetadata: MetadataRoundtrip {}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the site table of a [`TableCollection`](crate::TableCollection).
pub trait SiteMetadata: MetadataRoundtrip {}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the individual table of a [`TableCollection`](crate::TableCollection).
pub trait IndividualMetadata: MetadataRoundtrip {}

/// Marker trait indicating [`MetadataRoundtrip`]
/// for the population table of a [`TableCollection`](crate::TableCollection).
pub trait PopulationMetadata: MetadataRoundtrip {}

pub(crate) struct EncodedMetadata {
    encoded: Vec<u8>,
}

impl EncodedMetadata {
    pub(crate) fn new<M: MetadataRoundtrip + ?Sized>(md: &M) -> Result<Self, MetadataError> {
        let encoded = md.encode()?;
        Ok(Self { encoded })
    }

    pub(crate) fn as_ptr(&self) -> *const libc::c_char {
        if self.encoded.is_empty() {
            std::ptr::null()
        } else {
            self.encoded.as_ptr() as *const libc::c_char
        }
    }

    pub(crate) fn len(&self) -> tsk_size_t {
        self.encoded.len() as tsk_size_t
    }
}

#[derive(Error, Debug)]
pub enum MetadataError {
    /// Error related to types implementing
    /// [``MetadataRoundtrip``]
    #[error("{}", *value)]
    RoundtripError {
        #[from]
        value: Box<dyn std::error::Error>,
    },
}

pub(crate) fn char_column_to_vector(
    column: *const libc::c_char,
    column_offset: *const tsk_size_t,
    row: tsk_id_t,
    num_rows: tsk_size_t,
    column_length: tsk_size_t,
) -> Result<Option<Vec<u8>>, crate::TskitError> {
    if row < 0 || (row as tsk_size_t) >= num_rows {
        return Err(crate::TskitError::IndexError {});
    }
    if column_length == 0 {
        return Ok(None);
    }
    let start = unsafe { *column_offset.offset(row as isize) };
    let stop = if (row as tsk_size_t) < num_rows {
        unsafe { *column_offset.offset((row + 1) as isize) }
    } else {
        column_length
    };
    if start >= stop {
        return Ok(None);
    }
    if column_length == 0 {
        return Ok(None);
    }
    let mut buffer = vec![];
    for i in start..stop {
        buffer.push(unsafe { *column.offset(i as isize) } as u8);
    }
    Ok(Some(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec8_cast_to_c_string() {
        let v: Vec<u8> = vec![0, 1, b'\0', 2, 3];
        let c = v.as_ptr() as *const libc::c_char;
        for (i, vi) in v.iter().enumerate() {
            assert_eq!(*vi as i8, unsafe { *c.add(i) });
        }

        let _ = match Some(&v) {
            Some(x) => x.as_ptr() as *const libc::c_char,
            None => std::ptr::null(),
        };
    }

    struct F {
        x: i32,
        y: u32,
    }

    impl MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            let mut rv = vec![];
            rv.extend(self.x.to_le_bytes().iter().copied());
            rv.extend(self.y.to_le_bytes().iter().copied());
            Ok(rv)
        }
        fn decode(md: &[u8]) -> Result<Self, MetadataError> {
            use std::convert::TryInto;
            let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
            let (y_int_bytes, _) = rest.split_at(std::mem::size_of::<u32>());
            Ok(Self {
                x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
                y: u32::from_le_bytes(y_int_bytes.try_into().unwrap()),
            })
        }
    }

    impl MutationMetadata for F {}

    #[test]
    fn test_metadata_round_trip() {
        let f = F { x: -3, y: 42 };
        let v = f.encode().unwrap();
        let c = v.as_ptr() as *const libc::c_char;
        let mut d = vec![];
        for i in 0..v.len() {
            d.push(unsafe { *c.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }

    #[test]
    fn test_encoded_metadata_roundtrip() {
        let f = F { x: -3, y: 42 };
        let enc = EncodedMetadata::new(&f).unwrap();
        let p = enc.as_ptr();
        let mut d = vec![];
        for i in 0..enc.len() {
            d.push(unsafe { *p.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use crate::test_fixtures::bad_metadata::*;

    #[test]
    fn test_metadata_round_trip() {
        let f = F { x: -3, y: 42 };
        let v = f.encode().unwrap();
        let c = v.as_ptr() as *const libc::c_char;
        let mut d = vec![];
        for i in 0..v.len() {
            d.push(unsafe { *c.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }

    #[test]
    fn test_encoded_metadata_roundtrip() {
        let f = F { x: -3, y: 42 };
        let enc = EncodedMetadata::new(&f).unwrap();
        let p = enc.as_ptr();
        let mut d = vec![];
        for i in 0..enc.len() {
            d.push(unsafe { *p.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }

    #[test]
    fn test_metadata_round_trip_wrong_type() {
        let f = F { x: -3, y: 42 };
        let v = f.encode().unwrap();
        let c = v.as_ptr() as *const libc::c_char;
        let mut d = vec![];
        for i in 0..v.len() {
            d.push(unsafe { *c.add(i as usize) as u8 });
        }
        if crate::test_fixtures::bad_metadata::Ff::decode(&d).is_ok() {
            panic!("expected an error!!");
        }
    }
}
