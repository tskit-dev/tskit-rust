use std::ffi::CString;

use mbox::MBox;
use thiserror::Error;

use crate::bindings;

use bindings::tsk_edge_table_t;
use bindings::tsk_individual_table_t;
use bindings::tsk_migration_table_t;
use bindings::tsk_mutation_table_t;
use bindings::tsk_node_table_t;
use bindings::tsk_population_table_t;
#[cfg(feature = "provenance")]
use bindings::tsk_provenance_table_t;
use bindings::tsk_site_table_t;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("{}", *.0)]
    Message(String),
    #[error("{}", get_tskit_error_message(*.0))]
    Code(i32),
    #[error("NULL pointer encountered")]
    NullPointer,
}

#[derive(Debug)]
pub struct LowLevelPointerManager<T> {
    pointer: *mut T,
    owned: bool,
    tskfree: Option<fn(*mut T) -> i32>,
}

impl<T> LowLevelPointerManager<T> {
    fn new_owning<I>(init: I, free: fn(*mut T) -> i32) -> Result<Self, Error>
    where
        I: Fn(*mut T) -> i32,
    {
        let pointer = unsafe { libc::malloc(std::mem::size_of::<T>()) as *mut T };
        if pointer.is_null() {
            Err(Error::Code(crate::bindings::TSK_ERR_NO_MEMORY))
        } else {
            // The call to setup will leak memory if we don't
            // explicitly match the Ok/Err pathways.
            // Instead, we use RAII via MBox to free our pointer
            // in the case where setup errors.

            // SAFETY: pointer not null
            let mut pointer = unsafe { MBox::from_raw(pointer) };
            Self::setup(pointer.as_mut(), init)?;
            Ok(Self {
                pointer: MBox::into_raw(pointer),
                owned: true,
                tskfree: Some(free),
            })
        }
    }

    fn new_non_owning(pointer: *mut T) -> Result<Self, Error> {
        if pointer.is_null() {
            Err(Error::NullPointer {})
        } else {
            Ok(Self {
                pointer,
                owned: false,
                // In tskit-c, a non-owning pointer does not tear down
                // its data. Doing so is the responsibility
                // of the owning object.
                tskfree: None,
            })
        }
    }

    fn setup<I>(pointer: *mut T, tskinit: I) -> Result<(), Error>
    where
        I: Fn(*mut T) -> i32,
    {
        assert!(!pointer.is_null());
        match tskinit(pointer) {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(()),
        }
    }

    fn teardown(&mut self) -> Result<(), Error> {
        assert!(!self.pointer.is_null());
        self.tskfree.map_or_else(
            || Ok(()),
            |function| match function(self.pointer) {
                code if code < 0 => Err(Error::Code(code)),
                _ => Ok(()),
            },
        )
    }

    // NOTE: the stuff below is boiler-plate-y
    // and we'll want to make that less painful later.

    fn as_mut_ptr(&mut self) -> *mut T {
        self.pointer
    }

    fn as_ptr(&self) -> *const T {
        self.pointer
    }

    // fn as_mut(&mut self) -> &mut T {
    //     assert!(self.pointer.is_null());
    //     // SAFETY: pointer is not null
    //     unsafe { &mut *self.pointer }
    // }

    fn as_ref(&self) -> &T {
        assert!(!self.pointer.is_null());
        // SAFETY: pointer is not null
        unsafe { &*self.pointer }
    }
}

impl<T> Drop for LowLevelPointerManager<T> {
    fn drop(&mut self) {
        // Will not
        self.teardown().unwrap();
        if self.owned {
            assert!(!self.pointer.is_null());
            // SAFETY: pointer is not null and we "own" it,
            // meaning that we malloc'd it.
            unsafe { libc::free(self.pointer.cast::<std::ffi::c_void>()) }
            self.pointer = std::ptr::null_mut();
        }
    }
}

macro_rules! basic_lltable_impl {
    ($lltable: ident, $tsktable: ident, $init: expr, $free: expr, $clear: expr) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $lltable(LowLevelPointerManager<$tsktable>);

        impl $lltable {
            pub fn new_owning(flags: bindings::tsk_flags_t) -> Result<Self, Error> {
                let internal = LowLevelPointerManager::<$tsktable>::new_owning(
                    |x| {
                        assert!(!x.is_null());
                        // SAFETY: pointer is not NULL
                        unsafe { $init(x, flags) }
                    },
                    |x| {
                        assert!(!x.is_null());
                        // SAFETY: pointer is not NULL
                        unsafe { $free(x) }
                    },
                )?;
                Ok(Self(internal))
            }

            pub fn new_non_owning(table: *mut $tsktable) -> Result<Self, Error> {
                let internal = LowLevelPointerManager::<$tsktable>::new_non_owning(table)?;
                Ok(Self(internal))
            }

            pub fn clear(&mut self) -> Result<(), Error> {
                assert!(!self.0.pointer.is_null());
                match unsafe { $clear(self.0.pointer) } {
                    x if x < 0 => Err(Error::Code(x)),
                    _ => Ok(()),
                }
            }

            pub fn as_ref(&self) -> &$tsktable {
                self.0.as_ref()
            }

            pub fn as_ptr(&self) -> *const $tsktable {
                self.0.as_ptr()
            }

            pub fn as_mut_ptr(&mut self) -> *mut $tsktable {
                self.0.as_mut_ptr()
            }
        }
    };
}

basic_lltable_impl!(
    LLEdgeTable,
    tsk_edge_table_t,
    bindings::tsk_edge_table_init,
    bindings::tsk_edge_table_free,
    bindings::tsk_edge_table_clear
);
basic_lltable_impl!(
    LLNodeTable,
    tsk_node_table_t,
    bindings::tsk_node_table_init,
    bindings::tsk_node_table_free,
    bindings::tsk_node_table_clear
);
basic_lltable_impl!(
    LLSiteTable,
    tsk_site_table_t,
    bindings::tsk_site_table_init,
    bindings::tsk_site_table_free,
    bindings::tsk_site_table_clear
);
basic_lltable_impl!(
    LLMutationTable,
    tsk_mutation_table_t,
    bindings::tsk_mutation_table_init,
    bindings::tsk_mutation_table_free,
    bindings::tsk_mutation_table_clear
);
basic_lltable_impl!(
    LLIndividualTable,
    tsk_individual_table_t,
    bindings::tsk_individual_table_init,
    bindings::tsk_individual_table_free,
    bindings::tsk_individual_table_clear
);
basic_lltable_impl!(
    LLPopulationTable,
    tsk_population_table_t,
    bindings::tsk_population_table_init,
    bindings::tsk_population_table_free,
    bindings::tsk_population_table_clear
);
basic_lltable_impl!(
    LLMigrationTable,
    tsk_migration_table_t,
    bindings::tsk_migration_table_init,
    bindings::tsk_migration_table_free,
    bindings::tsk_migration_table_clear
);
#[cfg(feature = "provenance")]
basic_lltable_impl!(
    LLProvenanceTable,
    tsk_provenance_table_t,
    bindings::tsk_provenance_table_init,
    bindings::tsk_provenance_table_free,
    bindings::tsk_provenance_table_clear
);

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
    if row < 0 || (row as crate::tsk_size_t) >= column_length {
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
    let c_str = unsafe { std::ffi::CStr::from_ptr(crate::bindings::tsk_strerror(code)) };
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

// NOTE: these are design phase tests

#[test]
fn test_low_level_table_collection_pointer_manager_owning() {
    let flags: bindings::tsk_flags_t = 0;
    let mut x = LowLevelPointerManager::<bindings::tsk_table_collection_t>::new_owning(
        |x| {
            assert!(!x.is_null());
            // SAFETY: pointer is not NULL
            unsafe { bindings::tsk_table_collection_init(x, flags) }
        },
        |x| {
            assert!(!x.is_null());
            // SAFETY: pointer is not NULL
            unsafe { bindings::tsk_table_collection_free(x) }
        },
    )
    .unwrap();
    assert!(x.owned);
    assert!(!x.as_ptr().is_null());
    assert!(!x.as_mut_ptr().is_null());
}

#[test]
fn test_low_level_table_collection_pointer_manager_non_owning() {
    let raw = unsafe {
        libc::malloc(std::mem::size_of::<bindings::tsk_table_collection_t>())
            as *mut bindings::tsk_table_collection_t
    };
    let mut x =
        LowLevelPointerManager::<bindings::tsk_table_collection_t>::new_non_owning(raw).unwrap();
    assert!(!x.owned);
    assert!(!x.as_ptr().is_null());
    assert!(!x.as_mut_ptr().is_null());
    unsafe { libc::free(raw as *mut libc::c_void) };
}

#[cfg(test)]
mod soundness_tests {
    use super::*;

    #[test]
    fn test_non_owning_table_runtime_soundness() {
        let raw = unsafe {
            libc::malloc(std::mem::size_of::<bindings::tsk_table_collection_t>())
                as *mut bindings::tsk_table_collection_t
        };
        let mut x = LowLevelPointerManager::<bindings::tsk_table_collection_t>::new_non_owning(raw)
            .unwrap();
        let rv = unsafe { bindings::tsk_table_collection_init(x.as_mut_ptr(), 0) };
        assert_eq!(rv, 0);
        let mut n = unsafe { LLNodeTable::new_non_owning(&mut (*x.as_mut_ptr()).nodes).unwrap() };
        let rv = unsafe { bindings::tsk_table_collection_free(x.as_mut_ptr()) };
        assert_eq!(rv, 0);
        drop(x);
        unsafe { libc::free(raw as *mut libc::c_void) };
        assert!(raw.is_null());
        assert!(n.as_ptr().is_null());
        n.as_ref();
    }
}

// NOTE: design phase 2 tests

#[test]
fn test_lledgetable() {
    let _ = LLEdgeTable::new_owning(0).unwrap();
}
