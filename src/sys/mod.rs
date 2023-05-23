#[allow(dead_code)]
#[allow(deref_nullptr)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod bindings;

pub mod flags;
mod tables;
mod tree;
mod treeseq;

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub(crate) const TSK_NULL: bindings::tsk_id_t = -1;

pub use tables::*;
pub use tree::LLTree;
pub use treeseq::LLTreeSeq;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Code(i32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::Message(m) => write!(f, "{}", m),
            Error::Code(c) => write!(f, "{}", get_tskit_error_message(*c)),
        }
    }
}

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

pub fn generate_slice<'a, L: Into<bindings::tsk_size_t>, I, O>(
    data: *const I,
    length: L,
) -> &'a [O] {
    assert!(!data.is_null());
    // SAFETY: pointer is not null, length comes from C API
    unsafe { std::slice::from_raw_parts(data.cast::<O>(), length.into() as usize) }
}

pub fn generate_slice_mut<'a, L: Into<bindings::tsk_size_t>, I, O>(
    data: *mut I,
    length: L,
) -> &'a mut [O] {
    assert!(!data.is_null());
    // SAFETY: pointer is not null, length comes from C API
    unsafe { std::slice::from_raw_parts_mut(data.cast::<O>(), length.into() as usize) }
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
    fn foo() -> Result<(), Error> {
        Err(Error::Message("foobar".to_owned()))
    }

    let msg = "foobar".to_owned();
    match foo() {
        Err(Error::Message(m)) => assert_eq!(m, msg),
        _ => panic!("unexpected match"),
    }
}

#[test]
fn test_error_code() {
    fn foo() -> Result<(), Error> {
        Err(Error::Code(-202))
    }

    match foo() {
        Err(Error::Code(x)) => {
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
