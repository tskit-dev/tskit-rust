//! A rust interface to [tskit](https://github.com/tskit-dev/tskit).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod bindings;

mod _macros; // Starts w/_ to be sorted at front by rustfmt!
mod edge_table;
pub mod error;
pub mod ffi;
pub mod metadata;
mod mutation_table;
mod node_table;
mod population_table;
mod site_table;
mod table_collection;
pub mod types;

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

pub use edge_table::EdgeTable;
pub use error::TskitError;
pub use mutation_table::MutationTable;
pub use node_table::NodeTable;
pub use population_table::PopulationTable;
pub use site_table::SiteTable;
pub use table_collection::TableCollection;
/// Handles return codes from low-level tskit functions.
///
/// When an error from the tskit C API is detected,
/// the error message is stored for diplay.
pub type TskReturnValue = Result<i32, TskitError>;

/// Get the tskit version number.
pub fn version() -> &'static str {
    return env!("CARGO_PKG_VERSION");
}

// Testing modules
mod test_tsk_variables;
