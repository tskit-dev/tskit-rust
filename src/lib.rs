//! A rust interface to [tskit](https://github.com/tskit-dev/tskit).
//!
//! This crate provides a mapping of the `tskit` C API to rust.
//! The result is an interface similar to the `tskit` Python interface,
//! but with all operations implemented using compiled code.
//!
//! # Features
//!
//! ## Interface to the C library
//!
//! * [`TableCollection`] wraps `tsk_table_collection_t`.
//! * [`TreeSequence`] wraps `tsk_treeseq_t`.
//! * [`Tree`] wraps `tsk_tree_t`.
//! * Tree iteration occurs via traits from [streaming_iterator](https://docs.rs/streaming-iterator/).
//! * Errors returned from C map to [`TskitError::ErrorCode`].
//!   Their string messages can be obtained by printing the error type.
//!
//! ## Safety
//!
//! * The types listed above handle all the memory management!
//! * All array accesses are range-checked.
//! * Object lifetimes are clear:
//!     * Creating a tree sequence moves/consumes a table collection.
//!     * Tree lifetimes are tied to that of the parent tree sequence.
//!     * Table objects ([`NodeTable`], etc..) are only represented by non-owning, immutable types.
//!
//! ## Prelude
//!
//! The [`prelude`] module contains definitions that are difficult/annoying to live without.
//! In particuar, this module exports various traits that make it so that client code does
//! not have to `use` them a la carte.
//!
//! We recomment that client code import all symbols from this module:
//!
//! ```
//! use tskit::prelude::*;
//! ```
//!
//! The various documentation examples manually `use` each trait both in order
//! to illustrate which traits are needed and to serve as doc tests.
//!
//! # Optional features
//!
//! Some features are optional, and are activated by requesting them in your `Cargo.toml` file.
//!
//! * `provenance`
//!     * Enables [`provenance`]
//! * `derive` enables the following derive macros:
//!     * [`crate::metadata::MutationMetadata`]
//!     * [`crate::metadata::IndividualMetadata`]
//!     * [`crate::metadata::SiteMetadata`]
//!     * [`crate::metadata::EdgeMetadata`]
//!     * [`crate::metadata::NodeMetadata`]
//!     * [`crate::metadata::MigrationMetadata`]
//!     * [`crate::metadata::PopulationMetadata`]
//!
//!     To see these derive macros in action, take a look
//!     [`here`](metadata).
//!         
//! To add features to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! tskit = {version = "0.2.0", features=["feature0", "feature1"]}
//! ```
//!
//! # What is missing?
//!
//! * A lot of wrappers to the C functions.
//! * Tree sequence statistics!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[allow(deref_nullptr)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod bindings;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_table;
pub mod error;
mod ffi;
mod flags;
mod individual_table;
pub mod metadata;
mod migration_table;
mod mutation_table;
mod node_table;
mod population_table;
pub mod prelude;
mod site_table;
mod table_collection;
mod table_iterator;
mod traits;
mod trees;
pub mod types;
mod util;

// re-export fundamental constants that
// we can't live without
pub use bindings::TSK_NODE_IS_SAMPLE;

// re-export types, too
pub use bindings::tsk_flags_t;

use bindings::tsk_id_t;
use bindings::tsk_size_t;

/// A node ID
///
/// This is an integer referring to a row of a [``NodeTable``].
/// The underlying type is [``tsk_id_t``].
///
/// # Examples
///
/// These examples illustrate using this type as something "integer-like".
///
/// ```
/// use tskit::NodeId;
/// use tskit::bindings::tsk_id_t;
///
/// let x: tsk_id_t = 1;
/// let y: NodeId = NodeId::from(x);
/// assert_eq!(x, y);
/// assert_eq!(y, x);
///
/// assert!(y < x + 1);
/// assert!(y <= x);
/// assert!(x + 1 > y);
/// assert!(x + 1 >= y);
///
/// let z: NodeId = NodeId::from(x);
/// assert_eq!(y, z);
/// ```
///
/// It is also possible to write functions accepting both the `NodeId`
/// and `tsk_id_t`:
///
/// ```
/// use tskit::NodeId;
/// use tskit::bindings::tsk_id_t;
///
/// fn interesting<N: Into<NodeId>>(x: N) -> NodeId {
///     x.into()
/// }
///
/// let x: tsk_id_t = 0;
/// assert_eq!(interesting(x), x);
/// let x: NodeId = NodeId::from(0);
/// assert_eq!(interesting(x), x);
/// ```
///
/// The types also implement `Display`:
///
/// ```
/// use tskit::NodeId;
///
/// let n = NodeId::from(11);
/// assert_eq!(format!("{}", n), "NodeId(11)".to_string());
/// let n = NodeId::from(NodeId::NULL);
/// assert_eq!(format!("{}", n), "NodeId(NULL)".to_string());
/// ```
///
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct NodeId(tsk_id_t);

/// An individual ID
///
/// This is an integer referring to a row of an [``IndividualTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct IndividualId(tsk_id_t);

/// A population ID
///
/// This is an integer referring to a row of an [``PopulationTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct PopulationId(tsk_id_t);

/// A site ID
///
/// This is an integer referring to a row of an [``SiteTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct SiteId(tsk_id_t);

/// A mutation ID
///
/// This is an integer referring to a row of an [``MutationTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct MutationId(tsk_id_t);

/// A migration ID
///
/// This is an integer referring to a row of an [``MigrationTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct MigrationId(tsk_id_t);

/// An edge ID
///
/// This is an integer referring to a row of an [``EdgeTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct EdgeId(tsk_id_t);

impl_id_traits!(NodeId);
impl_id_traits!(IndividualId);
impl_id_traits!(PopulationId);
impl_id_traits!(SiteId);
impl_id_traits!(MutationId);
impl_id_traits!(MigrationId);
impl_id_traits!(EdgeId);

impl_size_type_comparisons_for_row_ids!(NodeId);
impl_size_type_comparisons_for_row_ids!(EdgeId);
impl_size_type_comparisons_for_row_ids!(SiteId);
impl_size_type_comparisons_for_row_ids!(MutationId);
impl_size_type_comparisons_for_row_ids!(PopulationId);
impl_size_type_comparisons_for_row_ids!(MigrationId);

/// Wraps `tsk_size_t`
///
/// This type plays the role of C's `size_t` in the `tskit` C library.
///
/// # Examples
///
/// ```
/// let s = tskit::SizeType::from(1 as tskit::bindings::tsk_size_t);
/// let mut t: tskit::bindings::tsk_size_t = s.into();
/// assert!(t == s);
/// assert!(t == 1);
/// let u = tskit::SizeType::from(s);
/// assert!(u == s);
/// t += 1;
/// assert!(t > s);
/// assert!(s < t);
/// ```
///
/// #[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct SizeType(tsk_size_t);

impl std::fmt::Display for SizeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SizeType({})", self.0)
    }
}

impl From<tsk_size_t> for SizeType {
    fn from(value: tsk_size_t) -> Self {
        Self(value)
    }
}

impl From<SizeType> for tsk_size_t {
    fn from(value: SizeType) -> Self {
        value.0
    }
}

// SizeType is u64, so converstion
// can fail on systems with smaller pointer widths.
impl TryFrom<SizeType> for usize {
    type Error = TskitError;

    fn try_from(value: SizeType) -> Result<Self, Self::Error> {
        match usize::try_from(value.0) {
            Ok(x) => Ok(x),
            Err(_) => Err(TskitError::RangeError(format!(
                "could not convert {} to usize",
                value
            ))),
        }
    }
}

impl From<usize> for SizeType {
    fn from(value: usize) -> Self {
        Self(value as tsk_size_t)
    }
}

impl TryFrom<tsk_id_t> for SizeType {
    type Error = crate::TskitError;

    fn try_from(value: tsk_id_t) -> Result<Self, Self::Error> {
        match tsk_size_t::try_from(value) {
            Ok(v) => Ok(Self(v)),
            Err(_) => Err(crate::TskitError::RangeError(
                stringify!(value.0).to_string(),
            )),
        }
    }
}

impl TryFrom<SizeType> for tsk_id_t {
    type Error = crate::TskitError;

    fn try_from(value: SizeType) -> Result<Self, Self::Error> {
        match tsk_id_t::try_from(value.0) {
            Ok(v) => Ok(v),
            Err(_) => Err(TskitError::RangeError(stringify!(value.0).to_string())),
        }
    }
}

impl PartialEq<SizeType> for tsk_size_t {
    fn eq(&self, other: &SizeType) -> bool {
        *self == other.0
    }
}

impl PartialEq<tsk_size_t> for SizeType {
    fn eq(&self, other: &tsk_size_t) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<tsk_size_t> for SizeType {
    fn partial_cmp(&self, other: &tsk_size_t) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<SizeType> for tsk_size_t {
    fn partial_cmp(&self, other: &SizeType) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

/// A newtype for the concept of time.
/// A `Time` value can represent either a point in time
/// or the output of arithmetic involving time.
///
/// Wraps [`f64`].
///
/// # Examples
///
/// ```
/// let t0 = tskit::Time::from(2.0);
/// let t1 = tskit::Time::from(10.0);
///
/// let mut sum = t0 + t1;
///
/// match sum.partial_cmp(&12.0) {
///    Some(std::cmp::Ordering::Equal) => (),
///    _ => assert!(false),
/// };
///
/// sum /= tskit::Time::from(2.0);
///
/// match sum.partial_cmp(&6.0) {
///    Some(std::cmp::Ordering::Equal) => (),
///    _ => assert!(false),
/// };
/// ```
///
/// # Notes
///
/// The current implementation of [`PartialOrd`] is based on
/// the underlying implementation for [`f64`].
///
/// A `Time` can be multiplied and divided by a [`Position`]
///
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Time(f64);

/// A newtype for the concept of "genomic position".
/// A `Position` can represent either a locus or a
/// distance between loci.
///
/// Wraps [`f64`].
///
/// For examples, see [`Time`].
///
/// This type can be multiplied and divided by [`Time`].
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Position(f64);

/// A newtype for the concept of location.
/// A `Location` may represent a location or the
/// output of arithmetic involving locations.
///
/// Wraps [`f64`].
///
/// For examples, see [`Time`].
///
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Location(f64);

impl_f64_newtypes!(Time);
impl_f64_newtypes!(Position);
impl_f64_newtypes!(Location);

// It is natural to be able to * and / times and positions
impl_time_position_arithmetic!(Time, Position);
impl_time_position_arithmetic!(Position, Time);

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub(crate) const TSK_NULL: tsk_id_t = -1;

pub use edge_table::{EdgeTable, EdgeTableRow};
pub use error::TskitError;
pub use flags::*;
pub use individual_table::{IndividualTable, IndividualTableRow};
pub use migration_table::{MigrationTable, MigrationTableRow};
pub use mutation_table::{MutationTable, MutationTableRow};
pub use node_table::{NodeTable, NodeTableRow};
pub use population_table::{PopulationTable, PopulationTableRow};
pub use site_table::{SiteTable, SiteTableRow};
pub use table_collection::TableCollection;
pub use traits::IndividualLocation;
pub use traits::IndividualParents;
pub use traits::NodeListGenerator;
pub use traits::TableAccess;
pub use traits::TskitTypeAccess;
pub use trees::{NodeTraversalOrder, Tree, TreeSequence};

// Optional features
#[cfg(any(feature = "provenance", doc))]
pub mod provenance;

#[cfg(any(feature = "provenance", doc))]
/// A provenance ID
///
/// This is an integer referring to a row of a [``provenance::ProvenanceTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct ProvenanceId(tsk_id_t);

#[cfg(feature = "provenance")]
impl_id_traits!(ProvenanceId);

/// Handles return codes from low-level tskit functions.
///
/// When an error from the tskit C API is detected,
/// the error message is stored for diplay.
pub type TskReturnValue = Result<i32, TskitError>;

/// Alias for tsk_flags_t
pub type RawFlags = crate::bindings::tsk_flags_t;

/// Version of the rust crate.
///
/// To get the C API version, see:
/// * [`c_api_major_version`]
/// * [`c_api_minor_version`]
/// * [`c_api_patch_version`]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// C API major version
pub fn c_api_major_version() -> u32 {
    bindings::TSK_VERSION_MAJOR
}

/// C API minor version
pub fn c_api_minor_version() -> u32 {
    bindings::TSK_VERSION_MINOR
}

/// C API patch version
pub fn c_api_patch_version() -> u32 {
    bindings::TSK_VERSION_PATCH
}

/// The C API version in MAJOR.MINOR.PATCH format
pub fn c_api_version() -> String {
    format!(
        "{}.{}.{}",
        c_api_major_version(),
        c_api_minor_version(),
        c_api_patch_version()
    )
}

#[cfg(test)]
mod tests {
    use super::c_api_version;
    use super::Location;
    use super::Position;
    use super::Time;

    #[test]
    fn test_c_api_version() {
        let _ = c_api_version();
    }

    #[test]
    fn test_f64_newtype_Display() {
        let x = Position::from(1.0);
        let mut output = String::new();
        std::fmt::write(&mut output, format_args!("{}", x))
            .expect("Error occurred while trying to write in String");
        assert_eq!(output, "Position(1)".to_string());
        let x = Time::from(1.0);
        let mut output = String::new();
        std::fmt::write(&mut output, format_args!("{}", x))
            .expect("Error occurred while trying to write in String");
        assert_eq!(output, "Time(1)".to_string());
        let x = Location::from(1.0);
        let mut output = String::new();
        std::fmt::write(&mut output, format_args!("{}", x))
            .expect("Error occurred while trying to write in String");
        assert_eq!(output, "Location(1)".to_string());
    }
}

// Testing modules
mod test_fixtures;
mod test_simplification;
mod test_tsk_variables;
