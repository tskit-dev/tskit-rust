use thiserror::Error;

mod macros;

#[allow(dead_code)]
#[allow(deref_nullptr)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod bindings;

mod edge_table;
pub mod flags;
mod individual_table;
mod migration_table;
mod mutation_table;
pub mod newtypes;
mod node_table;
mod population_table;
#[cfg(feature = "provenance")]
mod provenance_table;
mod site_table;
mod table_collection;
mod trait_impls;
mod traits;
mod tree;
mod treeseq;
mod tskbox;

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub(crate) const TSK_NULL: bindings::tsk_id_t = -1;

pub use edge_table::EdgeTable;
pub use individual_table::IndividualTable;
pub use migration_table::MigrationTable;
pub use mutation_table::MutationTable;
pub use node_table::NodeTable;
pub use population_table::PopulationTable;
#[cfg(feature = "provenance")]
pub use provenance_table::ProvenanceTable;
pub use site_table::SiteTable;
pub use table_collection::*;
pub use tree::LLTree;
pub use tree::NodeTraversalOrder;
pub use treeseq::TreeSequence;

use traits::TskTeardown;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TskitError {
    /// Returned when conversion attempts fail
    #[error("range error: {}", *.0)]
    RangeError(String),
    /// Used when bad input is encountered.
    #[error("we received {} but expected {}",*got, *expected)]
    ValueError { got: String, expected: String },
    /// Used when array access is out of range.
    /// Typically, this is used when accessing
    /// arrays allocated on the C side.
    #[error("Invalid index")]
    IndexError,
    /// Raised when samples are requested from
    /// [`crate::Tree`] objects, but sample lists are
    /// not being updated.
    #[error("Not tracking samples in Trees")]
    NotTrackingSamples,
    /// Wrapper around tskit C API error codes.
    #[error("{}", get_tskit_error_message(*code))]
    ErrorCode { code: i32 },
    /// A redirection of [``crate::metadata::MetadataError``]
    #[error("{value:?}")]
    MetadataError {
        /// The redirected error
        #[from]
        value: MetadataError,
    },
    /// General error variant
    #[error("{}", *.0)]
    LibraryError(String),
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MetadataError {
    /// Error related to types implementing
    /// metadata serialization.
    #[error("{}", *value)]
    RoundtripError {
        #[from]
        value: Box<dyn std::error::Error + Send + Sync>,
    },
}

//#[non_exhaustive]
//#[derive(Error, Debug)]
//pub enum Error {
//    #[error("{}", *.0)]
//    Message(String),
//    #[error("{}", get_tskit_error_message(*.0))]
//    Code(i32),
//}

fn tsk_column_access_detail<R: Into<bindings::tsk_id_t>, L: Into<bindings::tsk_size_t>, T: Copy>(
    row: R,
    column: *const T,
    column_length: L,
) -> Option<T> {
    let row = row.into();
    let column_length = column_length.into();
    if row < 0 || (row as crate::sys::bindings::tsk_size_t) >= column_length {
        None
    } else {
        assert!(!column.is_null());
        // SAFETY: pointer is not null.
        // column_length is assumed to come directly
        // from a table.
        Some(unsafe { *column.offset(row as isize) })
    }
}

pub fn tsk_column_access<
    O: From<T>,
    R: Into<bindings::tsk_id_t>,
    L: Into<bindings::tsk_size_t>,
    T: Copy,
>(
    row: R,
    column: *const T,
    column_length: L,
) -> Option<O> {
    tsk_column_access_detail(row, column, column_length).map(|v| v.into())
}

fn tsk_ragged_column_access_detail<
    R: Into<bindings::tsk_id_t>,
    L: Into<bindings::tsk_size_t>,
    T: Copy,
>(
    row: R,
    column: *const T,
    column_length: L,
    offset: *const bindings::tsk_size_t,
    offset_length: bindings::tsk_size_t,
) -> Option<(*const T, usize)> {
    let row = row.into();
    let column_length = column_length.into();
    if row < 0 || row as bindings::tsk_size_t > column_length || offset_length == 0 {
        None
    } else {
        assert!(!column.is_null());
        assert!(!offset.is_null());
        // SAFETY: pointers are not null
        // and *_length are given by tskit-c
        let index = row as isize;
        let start = unsafe { *offset.offset(index) };
        let stop = if (row as bindings::tsk_size_t) < column_length {
            unsafe { *offset.offset(index + 1) }
        } else {
            offset_length
        };
        if start == stop {
            None
        } else {
            Some((
                unsafe { column.offset(start as isize) },
                stop as usize - start as usize,
            ))
        }
    }
}

pub fn tsk_ragged_column_access<
    'a,
    O,
    R: Into<bindings::tsk_id_t>,
    L: Into<bindings::tsk_size_t>,
    T: Copy,
>(
    row: R,
    column: *const T,
    column_length: L,
    offset: *const bindings::tsk_size_t,
    offset_length: bindings::tsk_size_t,
) -> Option<&'a [O]> {
    // SAFETY: see tsk_ragged_column_access_detail
    tsk_ragged_column_access_detail(row, column, column_length, offset, offset_length)
        .map(|(p, n)| unsafe { std::slice::from_raw_parts(p.cast::<O>(), n) })
}

/// # SAFETY
///
/// * data must not be NULL
/// * length must be a valid offset from data
///   (ideally it comes from the tskit-c API)
pub unsafe fn generate_slice<'a, L: Into<bindings::tsk_size_t>, I, O>(
    data: *const I,
    length: L,
) -> &'a [O] {
    std::slice::from_raw_parts(data.cast::<O>(), length.into() as usize)
}

/// # SAFETY
///
/// * data must not be NULL
/// * length must be a valid offset from data
///   (ideally it comes from the tskit-c API)
pub unsafe fn generate_slice_mut<'a, L: Into<bindings::tsk_size_t>, I, O>(
    data: *mut I,
    length: L,
) -> &'a mut [O] {
    std::slice::from_raw_parts_mut(data.cast::<O>(), length.into() as usize)
}

pub fn get_tskit_error_message(code: i32) -> String {
    let c_str = unsafe { std::ffi::CStr::from_ptr(crate::sys::bindings::tsk_strerror(code)) };
    c_str
        .to_str()
        .expect("failed to convert c_str to &str")
        .to_owned()
}

#[test]
fn test_error_message() {
    fn foo() -> Result<(), TskitError> {
        Err(TskitError::LibraryError("foobar".to_owned()))
    }

    let msg = "foobar".to_owned();
    match foo() {
        Err(TskitError::LibraryError(m)) => assert_eq!(m, msg),
        _ => panic!("unexpected match"),
    }
}

#[test]
fn test_error_code() {
    fn foo() -> Result<(), TskitError> {
        Err(TskitError::ErrorCode { code: -202 })
    }

    match foo() {
        Err(TskitError::ErrorCode { code: x }) => {
            assert_eq!(x, -202);
        }
        _ => panic!("unexpected match"),
    }

    match foo() {
        Err(e) => {
            let m = format!("{}", e);
            assert_eq!(&m, "Node out of bounds. (TSK_ERR_NODE_OUT_OF_BOUNDS)");
        }
        _ => panic!("unexpected match"),
    }
}
