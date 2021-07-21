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
pub mod bindings;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_table;
pub mod error;
pub mod ffi;
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
pub use bindings::tsk_id_t;
pub use bindings::tsk_size_t;

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
/// use tskit::tsk_id_t;
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
/// use tskit::tsk_id_t;
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

impl_id_traits!(NodeId);
impl_id_traits!(IndividualId);
impl_id_traits!(PopulationId);

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub const TSK_NULL: tsk_id_t = -1;

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
pub use traits::IdIsNull;
pub use traits::NodeListGenerator;
pub use traits::TableAccess;
pub use traits::TskitTypeAccess;
pub use trees::{NodeTraversalOrder, Tree, TreeSequence};

// Optional features
#[cfg(any(doc, feature = "provenance"))]
pub mod provenance;

/// Handles return codes from low-level tskit functions.
///
/// When an error from the tskit C API is detected,
/// the error message is stored for diplay.
pub type TskReturnValue = Result<i32, TskitError>;

/// Version of the rust crate.
///
/// To get the C API version, see:
/// * [`c_api_major_version`]
/// * [`c_api_minor_version`]
/// * [`c_api_patch_version`]
pub fn version() -> &'static str {
    return env!("CARGO_PKG_VERSION");
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

    #[test]
    fn test_c_api_version() {
        let _ = c_api_version();
    }
}

// Testing modules
mod test_fixtures;
mod test_simplification;
mod test_tsk_variables;
