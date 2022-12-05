use crate::{bindings, TskitError};
use bindings::tsk_edge_table_t;
use bindings::tsk_individual_table_t;
use bindings::tsk_migration_table_t;
use bindings::tsk_mutation_table_t;
use bindings::tsk_node_table_t;
use bindings::tsk_population_table_t;
use bindings::tsk_site_table_t;
use std::ptr::NonNull;

#[cfg(feature = "provenance")]
use bindings::tsk_provenance_table_t;

macro_rules! basic_lltableref_impl {
    ($lltable: ident, $tsktable: ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $lltable(NonNull<bindings::$tsktable>);

        impl $lltable {
            pub fn new_from_table(table: *mut $tsktable) -> Result<Self, TskitError> {
                let internal = NonNull::new(table).ok_or_else(|| {
                    let msg = format!("null pointer to {}", stringify!($tsktable));
                    TskitError::LibraryError(msg)
                })?;
                Ok(Self(internal))
            }

            pub fn as_ref(&self) -> &$tsktable {
                // SAFETY: we cannot get this far w/o
                // going through new_from_table and that
                // fn protects us from null ptrs
                unsafe { self.0.as_ref() }
            }
        }
    };
}

basic_lltableref_impl!(LLEdgeTableRef, tsk_edge_table_t);
basic_lltableref_impl!(LLNodeTableRef, tsk_node_table_t);
basic_lltableref_impl!(LLMutationTableRef, tsk_mutation_table_t);
basic_lltableref_impl!(LLSiteTableRef, tsk_site_table_t);
basic_lltableref_impl!(LLMigrationTableRef, tsk_migration_table_t);
basic_lltableref_impl!(LLPopulationTableRef, tsk_population_table_t);
basic_lltableref_impl!(LLIndividualTableRef, tsk_individual_table_t);

#[cfg(feature = "provenance")]
basic_lltableref_impl!(LLProvenanceTableRef, tsk_provenance_table_t);

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
