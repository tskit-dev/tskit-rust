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
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[allow(deref_nullptr)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod bindings;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_differences;
mod edge_table;
pub mod error;
mod flags;
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
mod table_iterator;
pub mod table_views;
mod traits;
mod tree_interface;
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

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub(crate) const TSK_NULL: tsk_id_t = -1;

pub use edge_differences::*;
pub use edge_table::{EdgeTable, EdgeTableRow, OwningEdgeTable};
pub use error::TskitError;
pub use flags::*;
pub use individual_table::{IndividualTable, IndividualTableRow, OwningIndividualTable};
pub use migration_table::{MigrationTable, MigrationTableRow, OwningMigrationTable};
pub use mutation_table::{MutationTable, MutationTableRow, OwningMutationTable};
pub use newtypes::*;
pub use node_table::{NodeTable, NodeTableRow, OwningNodeTable};
pub use population_table::{OwningPopulationTable, PopulationTable, PopulationTableRow};
pub use site_table::{OwningSiteTable, SiteTable, SiteTableRow};
pub use table_collection::TableCollection;
pub use traits::IndividualLocation;
pub use traits::IndividualParents;
pub use tree_interface::{NodeTraversalOrder, TreeInterface};
pub use trees::{Tree, TreeSequence};

// Optional features
#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
pub mod provenance;

#[cfg(feature = "edgebuffer")]
mod edgebuffer;

#[cfg(feature = "edgebuffer")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "edgebuffer")))]
pub use edgebuffer::EdgeBuffer;

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

    #[test]
    fn test_c_api_version() {
        let _ = c_api_version();
    }
}

// Testing modules
mod test_fixtures;
mod test_simplification;
mod test_tsk_variables;
