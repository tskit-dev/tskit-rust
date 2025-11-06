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

#[non_exhaustive]
#[derive(Debug)]
pub enum TskitError {
    /// Returned when conversion attempts fail
    RangeError(String),
    /// Used when bad input is encountered.
    ValueError { got: String, expected: String },
    /// Used when array access is out of range.
    /// Typically, this is used when accessing
    /// arrays allocated on the C side.
    IndexError,
    /// Raised when samples are requested from
    /// [`crate::Tree`] objects, but sample lists are
    /// not being updated.
    NotTrackingSamples,
    /// Wrapper around tskit C API error codes.
    ErrorCode { code: i32 },
    /// A redirection of [``crate::metadata::MetadataError``]
    MetadataError {
        /// The redirected error
        value: MetadataError,
    },
    /// General error variant
    LibraryError(String),
}

impl std::fmt::Display for TskitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RangeError(msg) => write!(f, "range error: {}", msg),
            Self::ValueError { got, expected } => {
                write!(f, "we received {} but expected {}", got, expected)
            }
            Self::IndexError => write!(f, "Invalid index"),
            Self::NotTrackingSamples => write!(f, "Not tracking samples in Trees"),
            Self::ErrorCode { code } => write!(f, "{}", get_tskit_error_message(*code)),
            Self::MetadataError { value } => write!(f, "meta data error: {}", value),
            Self::LibraryError(msg) => write!(f, "library error: {msg}"),
        }
    }
}

impl From<MetadataError> for TskitError {
    fn from(value: MetadataError) -> Self {
        Self::MetadataError { value }
    }
}

impl std::error::Error for TskitError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum MetadataError {
    /// Error related to types implementing
    /// metadata serialization.
    RoundtripError {
        value: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoundtripError { value } => write!(f, "metadata round trip error: {value:?}"),
        }
    }
}

impl std::error::Error for MetadataError {}

//#[non_exhaustive]
//#[derive(Error, Debug)]
//pub enum Error {
//    #[error("{}", *.0)]
//    Message(String),
//    #[error("{}", get_tskit_error_message(*.0))]
//    Code(i32),
//}

/// SAFETY:
///
/// * column not null AND column_length is valid
/// * OR column_length == 0
unsafe fn tsk_column_access_detail<
    R: Into<bindings::tsk_id_t>,
    L: Into<bindings::tsk_size_t>,
    T: Copy,
>(
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

unsafe fn tsk_column_access<
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

fn tsk_ragged_column_access_detail<'a, T>(
    row: usize,
    column: &'a [T],
    raw_offset: &'a [bindings::tsk_size_t],
) -> Option<&'a [T]> {
    if row >= raw_offset.len() || raw_offset.is_empty() {
        None
    } else {
        let start = usize::try_from(raw_offset[row]).ok()?;
        let stop = if row < raw_offset.len() - 1 {
            usize::try_from(raw_offset[row + 1]).ok()?
        } else {
            column.len()
        };
        if start == stop {
            None
        } else {
            Some(&column[start..stop])
        }
    }
}

fn tsk_ragged_column_access<'a, O, R: Into<bindings::tsk_id_t>>(
    row: R,
    column: &'a [O],
    raw_offset: &'a [bindings::tsk_size_t],
) -> Option<&'a [O]> {
    let row = row.into();
    let row = usize::try_from(row).ok()?;
    tsk_ragged_column_access_detail(row, column, raw_offset)
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
