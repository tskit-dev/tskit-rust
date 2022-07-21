//! Support for table row metadata
//!
//! Metadata refers to data that client code may need to associate
//! with table rows, but the data are not necessary to perform algorithms
//! on tables nor on trees.
//!
//! For complete details, see the data model descriptions
//! [`here`](https://tskit.dev/tskit/docs/stable/)
//!
//! The most straightfoward way to implement metadata
//! is to use the optional `derive` feature of `tskit`.
//! This feature enables derive macros to convert
//! your types to metadata types via [`serde`](https://docs.rs/serde).
//!
//! Note that you will need to add `serde` as a dependency of your
//! package, as you will need its `Serialize` and `Deserialize`
//! derive macros available.
//!
//! Without the derive macros provided by tskit, you must `impl` [`MetadataRoundtrip`]
//! and the approprate table metadata tag marker for your type.
//! An example of such "manual" metadata type registration is shown
//! as the last example below.
//!
//! A technical details section follows the examples
//!
//! # Examples
//!
//! ## Mutation metadata encoded as JSON
//!
//! ```
//! # #[cfg(feature = "derive")] {
//! use tskit::handle_metadata_return;
//! use tskit::TableAccess;
//!
//! #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
//! #[serializer("serde_json")]
//! pub struct MyMutation {
//!     origin_time: i32,
//!     effect_size: f64,
//!     dominance: f64,
//! }
//!
//! let mut tables = tskit::TableCollection::new(100.).unwrap();
//! let mutation = MyMutation{origin_time: 100,
//!     effect_size: -1e-4,
//!     dominance: 0.25};
//!
//! // Add table row with metadata.
//! let id = tables.add_mutation_with_metadata(0, 0, tskit::MutationId::NULL, 100., None,
//!     &mutation).unwrap();
//!
//! // Decode the metadata
//! // The two unwraps are:
//! // 1. Handle Errors vs Option.
//! // 2. Handle the option for the case of no error.
//! let decoded = tables.mutations().metadata::<MyMutation>(id).unwrap().unwrap();
//! assert_eq!(mutation.origin_time, decoded.origin_time);
//! match decoded.effect_size.partial_cmp(&mutation.effect_size) {
//!     Some(std::cmp::Ordering::Greater) => assert!(false),
//!     Some(std::cmp::Ordering::Less) => assert!(false),
//!     Some(std::cmp::Ordering::Equal) => (),
//!     None => panic!("bad comparison"),
//! };
//! match decoded.dominance.partial_cmp(&mutation.dominance) {
//!     Some(std::cmp::Ordering::Greater) => assert!(false),
//!     Some(std::cmp::Ordering::Less) => assert!(false),
//!     Some(std::cmp::Ordering::Equal) => (),
//!     None => panic!("bad comparison"),
//! };
//! # }
//! ```
//! ## Example: individual metadata implemented via newtypes
//!
//! This time, we use [`bincode`](https://docs.rs/bincode/) via `serde`.
//!
//! ```
//! # #[cfg(feature = "derive")] {
//! use tskit::TableAccess;
//! #[derive(serde::Serialize, serde::Deserialize, PartialEq, PartialOrd)]
//! struct GeneticValue(f64);
//!
//! #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
//! #[serializer("bincode")]
//! struct IndividualMetadata {
//!     genetic_value: GeneticValue,
//! }
//! let mut tables = tskit::TableCollection::new(100.).unwrap();
//! let individual = IndividualMetadata {
//!     genetic_value: GeneticValue(0.0),
//! };
//! let id = tables.add_individual_with_metadata(0, &[] as &[tskit::Location], &[tskit::IndividualId::NULL], &individual).unwrap();
//! let decoded = tables.individuals().metadata::<IndividualMetadata>(id).unwrap().unwrap();
//! assert_eq!(decoded.genetic_value.partial_cmp(&individual.genetic_value).unwrap(), std::cmp::Ordering::Equal);
//! # }
//! ```
//!
//! ## Example: manual implementation of all of the traits.
//!
//! Okay, let's do things the hard way.
//! We will use a serializer not supported by `tskit` right now.
//! For fun, we'll use the Python [`pickle`](https://docs.rs/crate/serde-pickle/) format.
//!
//! ```
//! use tskit::TableAccess;
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct Metadata {
//!     data: String,
//! }
//!
//! // Manually implement the metadata round trip trait.
//! // You must propogate any errors back via Box, else
//! // risk a `panic!`.
//! impl tskit::metadata::MetadataRoundtrip for Metadata {
//!     fn encode(&self) -> Result<Vec<u8>, tskit::metadata::MetadataError> {
//!         match serde_pickle::to_vec(self, serde_pickle::SerOptions::default()) {
//!             Ok(v) => Ok(v),
//!             Err(e) => Err(tskit::metadata::MetadataError::RoundtripError{ value: Box::new(e) }),
//!         }
//!     }
//!
//!     fn decode(md: &[u8]) -> Result<Self, tskit::metadata::MetadataError> {
//!         match serde_pickle::from_slice(md, serde_pickle::DeOptions::default()) {
//!             Ok(x) => Ok(x),
//!             Err(e) => Err(tskit::metadata::MetadataError::RoundtripError{ value: Box::new(e) }),
//!         }
//!     }
//! }
//!
//! // If we want this to be, say, node metadata, then we need to mark
//! // it as such:
//! impl tskit::metadata::NodeMetadata for Metadata {}
//!
//! // Ready to rock:
//! let mut tables = tskit::TableCollection::new(1.).unwrap();
//! let id = tables
//!     .add_node_with_metadata(
//!         0,
//!         0.0,
//!         tskit::PopulationId::NULL,
//!         tskit::IndividualId::NULL,
//!         &Metadata {
//!             data: "Bananas".to_string(),
//!         },
//!     )
//!     .unwrap();
//!
//! let decoded = tables.nodes().metadata::<Metadata>(id).unwrap().unwrap();
//! assert_eq!(decoded.data, "Bananas".to_string());
//! ```
//!
//! # Technial details and notes
//!
//! * The derive macros currently support two `serde` methods:
//!   `serde_json` and `bincode`.
//! * A concept like "mutation metadata" is the combination of two traits:
//!   [`MetadataRoundtrip`] plus [`MutationMetadata`].
//!   The latter is a marker trait.
//!   The derive macros handle all of this "boiler plate" for you.
//!
//! ## Limitations/unknowns
//!
//! * We have not yet tested importing metadata encoded using `rust`
//!   into `Python` via the `tskit` `Python API`.

use crate::bindings::{tsk_id_t, tsk_size_t};
use crate::{SizeType, TskitError};
use thiserror::Error;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub extern crate tskit_derive;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use tskit_derive::{
    EdgeMetadata, IndividualMetadata, MigrationMetadata, MutationMetadata, NodeMetadata,
    PopulationMetadata, SiteMetadata,
};

/// Trait marking a type as table metadata
pub trait MetadataRoundtrip {
    /// Encode `self` as bytes
    fn encode(&self) -> Result<Vec<u8>, MetadataError>;
    /// Decond `Self` from bytes
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
/// for the migration table of a [`TableCollection`](crate::TableCollection).
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
            self.encoded.as_ptr().cast::<i8>()
        }
    }

    pub(crate) fn len(&self) -> SizeType {
        self.encoded.len().into()
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

pub(crate) fn char_column_to_slice<T: Sized>(
    _lifetime: &T,
    column: *const libc::c_char,
    column_offset: *const tsk_size_t,
    row: tsk_id_t,
    num_rows: tsk_size_t,
    column_length: tsk_size_t,
) -> Result<Option<&[u8]>, crate::TskitError> {
    let row = match tsk_size_t::try_from(row) {
        Ok(r) => r,
        Err(_) => return Err(TskitError::IndexError),
    };
    if row >= num_rows {
        return Err(crate::TskitError::IndexError {});
    }
    if column_length == 0 {
        return Ok(None);
    }
    let row_isize = match isize::try_from(row) {
        Ok(r) => r,
        Err(_) => {
            return Err(TskitError::RangeError(format!(
                "could not convert u64 value {} to isize",
                stringify!(row)
            )))
        }
    };
    let start = unsafe { *column_offset.offset(row_isize) };
    let stop = if (row as tsk_size_t) < num_rows {
        unsafe { *column_offset.offset(row_isize + 1) }
    } else {
        column_length
    };
    if start >= stop {
        return Ok(None);
    }
    if column_length == 0 {
        return Ok(None);
    }
    let istart = match isize::try_from(start) {
        Ok(v) => v,
        Err(_) => {
            return Err(TskitError::RangeError(format!(
                "cauld not convert value {} to isize",
                stringify!(i)
            )));
        }
    };
    let ustop = match usize::try_from(stop) {
        Ok(v) => v,
        Err(_) => {
            return Err(TskitError::RangeError(format!(
                "cauld not convert value {} to usize",
                stringify!(i)
            )));
        }
    };
    Ok(Some(unsafe {
        std::slice::from_raw_parts(column.offset(istart) as *const u8, ustop - istart as usize)
    }))
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
        for i in 0..usize::try_from(enc.len()).unwrap() {
            d.push(unsafe { *p.add(i) as u8 });
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
        for i in 0..usize::try_from(enc.len()).unwrap() {
            d.push(unsafe { *p.add(i) as u8 });
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
