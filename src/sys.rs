use crate::bindings;

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

pub fn tree_array_slice<'a, L: Into<bindings::tsk_size_t>>(
    data: *const bindings::tsk_id_t,
    length: L,
) -> &'a [crate::NodeId] {
    assert!(!data.is_null());
    // SAFETY: pointer is not null, length comes from C API
    unsafe { std::slice::from_raw_parts(data.cast::<crate::NodeId>(), length.into() as usize) }
}
