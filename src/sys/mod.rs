use std::ffi::CString;
use thiserror::Error;

#[allow(dead_code)]
#[allow(deref_nullptr)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod bindings;

mod tables;

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
/// "Null" identifier value.
pub(crate) const TSK_NULL: bindings::tsk_id_t = -1;

pub use tables::*;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("{}", *.0)]
    Message(String),
    #[error("{}", get_tskit_error_message(*.0))]
    Code(i32),
}

#[repr(transparent)]
pub struct LLTreeSeq(bindings::tsk_treeseq_t);

impl LLTreeSeq {
    pub fn new(
        tables: *mut bindings::tsk_table_collection_t,
        flags: bindings::tsk_flags_t,
    ) -> Result<Self, Error> {
        let mut inner = std::mem::MaybeUninit::<bindings::tsk_treeseq_t>::uninit();
        let mut flags = flags;
        flags |= bindings::TSK_TAKE_OWNERSHIP;
        match unsafe { bindings::tsk_treeseq_init(inner.as_mut_ptr(), tables, flags) } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(Self(unsafe { inner.assume_init() })),
        }
    }

    pub fn as_ref(&self) -> &bindings::tsk_treeseq_t {
        &self.0
    }

    pub fn as_ptr(&self) -> *const bindings::tsk_treeseq_t {
        &self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut bindings::tsk_treeseq_t {
        &mut self.0
    }

    pub fn simplify(
        &self,
        samples: &[bindings::tsk_id_t],
        options: bindings::tsk_flags_t,
        idmap: *mut bindings::tsk_id_t,
    ) -> Result<Self, Error> {
        // The output is an UNINITIALIZED treeseq,
        // else we leak memory.
        let mut ts = std::mem::MaybeUninit::<bindings::tsk_treeseq_t>::uninit();
        // SAFETY: samples is not null, idmap is allowed to be.
        // self.as_ptr() is not null
        let rv = unsafe {
            bindings::tsk_treeseq_simplify(
                self.as_ptr(),
                samples.as_ptr(),
                samples.len() as bindings::tsk_size_t,
                options,
                ts.as_mut_ptr(),
                idmap,
            )
        };
        let init = unsafe { ts.assume_init() };
        if rv < 0 {
            // SAFETY: the ptr is not null
            // and tsk_treeseq_free uses safe methods
            // to clean up.
            unsafe { bindings::tsk_treeseq_free(ts.as_mut_ptr()) };
            Err(Error::Code(rv))
        } else {
            Ok(Self(init))
        }
    }

    pub fn dump(&self, filename: CString, options: bindings::tsk_flags_t) -> Result<i32, Error> {
        // SAFETY: self pointer is not null
        match unsafe { bindings::tsk_treeseq_dump(self.as_ptr(), filename.as_ptr(), options) } {
            code if code < 0 => Err(Error::Code(code)),
            code => Ok(code),
        }
    }

    pub fn num_trees(&self) -> bindings::tsk_size_t {
        // SAFETY: self pointer is not null
        unsafe { bindings::tsk_treeseq_get_num_trees(self.as_ptr()) }
    }

    pub fn kc_distance(&self, other: &Self, lambda: f64) -> Result<f64, Error> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        // SAFETY: self pointer is not null
        match unsafe {
            bindings::tsk_treeseq_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(kc),
        }
    }

    pub fn num_samples(&self) -> bindings::tsk_size_t {
        unsafe { bindings::tsk_treeseq_get_num_samples(self.as_ptr()) }
    }

    fn free(&mut self) -> Result<(), Error> {
        match unsafe { bindings::tsk_treeseq_free(self.as_mut_ptr()) } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(()),
        }
    }
}

impl Drop for LLTreeSeq {
    fn drop(&mut self) {
        match self.free() {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
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
