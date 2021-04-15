//! A rust interface to [tskit](https://github.com/tskit-dev/tskit).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod bindings;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_table;
pub mod error;
pub mod ffi;
mod individual_table;
pub mod metadata;
mod migration_table;
mod mutation_table;
mod node_table;
mod population_table;
mod site_table;
mod table_collection;
mod table_iterator;
mod trees;
pub mod types;
mod util;

// re-export fundamental constants that
// we can't live without
pub use bindings::TSK_NODE_IS_SAMPLE;
pub use bindings::TSK_NO_BUILD_INDEXES;
pub use bindings::TSK_SAMPLE_LISTS;

// re-export types, too
pub use bindings::tsk_flags_t;
pub use bindings::tsk_id_t;
pub use bindings::tsk_size_t;

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub const TSK_NULL: tsk_id_t = -1;

pub use edge_table::{EdgeTable, EdgeTableRow};
pub use error::TskitError;
pub use individual_table::{IndividualTable, IndividualTableRow};
pub use migration_table::{MigrationTable, MigrationTableRow};
pub use mutation_table::{MutationTable, MutationTableRow};
pub use node_table::{NodeTable, NodeTableRow};
pub use population_table::{PopulationTable, PopulationTableRow};
pub use site_table::{SiteTable, SiteTableRow};
pub use table_collection::TableCollection;
pub use trees::{NodeIterator, NodeTraversalOrder, Tree, TreeFlags, TreeSequence};

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
mod test_tsk_variables;
