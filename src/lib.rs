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
//!     * Enables `provenance`
//! * `derive` enables the following derive macros:
//!     * [`crate::metadata::MutationMetadata`]
//!     * [`crate::metadata::IndividualMetadata`]
//!     * [`crate::metadata::SiteMetadata`]
//!     * [`crate::metadata::EdgeMetadata`]
//!     * [`crate::metadata::NodeMetadata`]
//!     * [`crate::metadata::MigrationMetadata`]
//!     * [`crate::metadata::PopulationMetadata`]
//!
//!   To see these derive macros in action, take a look
//!   [`here`](metadata).
//! * `unsafe_init` enables [`crate::TableCollection::new_from_raw`]
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
//!
//! # Manual
//!
//! A manual is [here](https://tskit-dev.github.io/tskit-rust).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]

use std::ffi::c_char;

#[cfg(feature = "bindings")]
pub use sys::bindings;

// We have to cast between raw pointers involving these types when handling metadata.
// These compile-time assertions help prevent undefined behavior in case we run into
// something unexpected on a specific platform.
const _: () = const { assert!(std::mem::size_of::<u8>() == std::mem::size_of::<c_char>()) };
const _: () =
    const { assert!(std::mem::size_of::<u8>() == std::mem::size_of::<std::ffi::c_char>()) };

pub use streaming_iterator::DoubleEndedStreamingIterator;
pub use streaming_iterator::StreamingIterator;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_differences;
mod edge_table;
pub mod error;
mod individual_table;
pub mod metadata;
mod migration_table;
mod mutation_table;
mod newtypes;
mod node_table;
mod population_table;
pub mod prelude;
mod site_table;
mod sys;
mod table_collection;
mod table_column;
mod traits;
mod trees;
pub mod types;

pub use edge_differences::*;
pub use edge_table::EdgeTable;
pub use error::TskitError;
pub use individual_table::IndividualTable;
pub use migration_table::MigrationTable;
pub use mutation_table::MutationTable;
pub use newtypes::*;
pub use node_table::{NodeDefaults, NodeDefaultsWithMetadata, NodeTable};
pub use population_table::PopulationTable;
pub use site_table::SiteTable;
pub use sys::flags::*;
pub use sys::NodeTraversalOrder;
pub use table_collection::TableCollection;
pub use traits::IndividualLocation;
pub use traits::IndividualParents;
pub use traits::TableColumn;
pub use trees::{Tree, TreeSequence};

pub use sys::Edge;
pub use sys::Individual;
pub use sys::Migration;
pub use sys::Mutation;
pub use sys::MutationRef;
pub use sys::Node;
pub use sys::Population;
#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
pub use sys::Provenance;
pub use sys::Site;
pub use sys::SiteRef;

// Optional features
#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
pub mod provenance;

/// Handles return codes from low-level tskit functions.
///
/// When an error from the tskit C API is detected,
/// the error message is stored for diplay.
pub type TskReturnValue = Result<i32, TskitError>;

/// Alias for tsk_flags_t
pub type RawFlags = crate::sys::bindings::tsk_flags_t;

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
    sys::bindings::TSK_VERSION_MAJOR
}

/// C API minor version
pub fn c_api_minor_version() -> u32 {
    sys::bindings::TSK_VERSION_MINOR
}

/// C API patch version
pub fn c_api_patch_version() -> u32 {
    sys::bindings::TSK_VERSION_PATCH
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
