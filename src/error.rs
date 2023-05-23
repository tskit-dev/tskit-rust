//! Error handling

use crate::sys;
use crate::TskReturnValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TskitErrorEnum {
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
        value: crate::metadata::MetadataError,
    },
    /// General error variant
    #[error("{}", *.0)]
    LibraryError(String),
}

impl From<crate::sys::Error> for TskitErrorEnum {
    fn from(error: sys::Error) -> Self {
        match error {
            sys::Error::Message(msg) => TskitErrorEnum::LibraryError(msg),
            sys::Error::Code(code) => TskitErrorEnum::ErrorCode { code },
        }
    }
}

#[derive(Debug)]
pub struct TskitError {
    variant: TskitErrorEnum,
}

impl std::fmt::Display for TskitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.variant {
            TskitErrorEnum::ErrorCode { code: x } => write!(f, "{}", get_tskit_error_message(x)),
            _ => write!(f, "other error variant"),
        }
    }
}

impl From<TskitErrorEnum> for TskitError {
    fn from(variant: TskitErrorEnum) -> Self {
        Self { variant }
    }
}

impl From<sys::Error> for TskitError {
    fn from(value: sys::Error) -> Self {
        let variant = TskitErrorEnum::from(value);
        Self { variant }
    }
}

impl From<crate::metadata::MetadataError> for TskitError {
    fn from(value: crate::metadata::MetadataError) -> Self {
        let variant = TskitErrorEnum::from(value);
        Self { variant }
    }
}

impl std::error::Error for TskitError {}

/// Takes the return code from a tskit
/// function and panics if the code indicates
/// an error.  The error message is included
/// in the panic statement.
///
/// Examples:
///
/// ```
/// let rv = 0;  // All good!
/// tskit::error::panic_on_tskit_error(rv);
/// let rv = 1;  // Probably something like a new node id.
/// tskit::error::panic_on_tskit_error(rv);
/// ```
///
/// This will panic:
///
/// ```should_panic
/// let rv = -202; // "Node out of bounds error"
/// tskit::error::panic_on_tskit_error(rv);
/// ```
pub fn panic_on_tskit_error(code: i32) {
    panic_on_tskit_error!(code);
}

/// Given a return value from low-level tskit function,
/// obtain the corresponding error message.
///
/// tskit returns 0 when there's no error:
/// ```
/// let x = tskit::error::get_tskit_error_message(0);
/// assert_eq!(x, "Normal exit condition. This is not an error!");
/// ```
///
/// Values > 0 are considered errors, but have no known type/cause.
/// tskit never returns error codes > 0 and there should be no attempt
/// to ever do so by client code.
///
/// ```
/// let x = tskit::error::get_tskit_error_message(1);
/// assert_eq!(x, "Unknown error");
/// ```
///
/// Values < 0 may have known causes:
///
/// ```
/// let x = tskit::error::get_tskit_error_message(-207);
/// assert!(x.contains("Individual out of bounds"));
/// ```
///
/// # Panics
///
/// This function must allocate a C string, which may panic
/// if the system runs out of memory.
pub fn get_tskit_error_message(code: i32) -> String {
    sys::get_tskit_error_message(code)
}

/// Given an instance of [``TskReturnValue``](crate::TskReturnValue),
/// obtain the tskit error message if there is indeed an error.
pub fn extract_error_message(x: TskReturnValue) -> Option<String> {
    x.map_or_else(|e: TskitError| Some(format!("{}", e)), |_| None)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_tskit_error_message() {
        let m = get_tskit_error_message(0);
        assert_eq!(m, "Normal exit condition. This is not an error!");
    }

    fn mock_error() -> TskReturnValue {
        handle_tsk_return_value!(-207)
    }

    fn mock_success() -> TskReturnValue {
        handle_tsk_return_value!(0)
    }

    #[test]
    fn test_error_formatting() {
        let x = mock_error();
        let mut s: String = "nope!".to_string();
        x.map_or_else(|e: TskitError| s = format!("{}", e), |_| ());
        assert!(s.contains("Individual out of bounds"));
    }

    #[test]
    fn test_extract_error_message() {
        let x = mock_error();
        match extract_error_message(x) {
            Some(s) => assert!(s.contains("Individual out of bounds")),
            None => panic!(),
        }

        if extract_error_message(mock_success()).is_some() {
            panic!();
        }
    }

    #[test]
    fn test_anyhow_compatability() {
        fn foo() -> anyhow::Result<crate::TableCollection> {
            let tables = crate::TableCollection::new(1.0)?;
            Ok(tables)
        }

        let _ = foo().unwrap();
    }
}
