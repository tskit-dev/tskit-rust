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

#[cfg(feature = "bindings")]
pub use sys::bindings;

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
mod table_iterator;
mod traits;
mod trees;
pub mod types;
mod util;

pub use edge_differences::*;
pub use edge_table::{EdgeTable, EdgeTableRow};
pub use error::TskitError;
pub use individual_table::{IndividualTable, IndividualTableRow};
pub use migration_table::{MigrationTable, MigrationTableRow};
pub use mutation_table::{MutationTable, MutationTableRow};
pub use newtypes::*;
pub use node_table::{NodeDefaults, NodeDefaultsWithMetadata, NodeTable, NodeTableRow};
pub use population_table::{PopulationTable, PopulationTableRow};
pub use site_table::{SiteTable, SiteTableRow};
pub use sys::flags::*;
pub use sys::NodeTraversalOrder;
pub use table_collection::TableCollection;
pub use table_column::{EdgeTableColumn, NodeTableColumn};
pub use traits::IndividualLocation;
pub use traits::TableAccess;
pub use traits::IndividualParents;
pub use trees::{Tree, TreeSequence};

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
