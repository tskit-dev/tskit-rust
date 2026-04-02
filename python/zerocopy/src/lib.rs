#[test]
fn tables_roundtrip() {
    use pyo3::prelude::*;

    use tskit::bindings::tsk_table_collection_free;
    use tskit::bindings::tsk_table_collection_init;
    use tskit::bindings::tsk_table_collection_t;

    Python::attach(|_py| {
        let tables_ptr = unsafe {
            pyo3::ffi::PyMem_Malloc(std::mem::size_of::<tsk_table_collection_t>())
                .cast::<tsk_table_collection_t>()
        };
        assert!(!tables_ptr.is_null());

        // SAFETY: ptr is not null
        let rv = unsafe { tsk_table_collection_init(tables_ptr, 0) };
        assert_eq!(rv, 0);
        let tables_ptr = std::ptr::NonNull::new(tables_ptr).unwrap();
        // Not null and initialized w/o error
        let mut tables = unsafe { tskit::TableCollection::new_from_raw(tables_ptr) }.unwrap();
        let _ = tables.add_node(0, 0.0, -1, -1).unwrap();

        let tables_ptr = tables.into_mut_ptr().unwrap();
        // Not null (the NonNull unwrapped on the previous line)
        let rv = unsafe { tsk_table_collection_free(tables_ptr.as_ptr()) };
        assert_eq!(rv, 0);

        // The pointer was allocated with PyMem_Malloc, so this is the correct free fn
        unsafe { pyo3::ffi::PyMem_Free(tables_ptr.as_ptr().cast::<std::ffi::c_void>()) };
    });
}

#[test]
fn treeseq_roundtrip() {
    use pyo3::prelude::*;

    use tskit::bindings::tsk_table_collection_free;
    use tskit::bindings::tsk_table_collection_init;
    use tskit::bindings::tsk_table_collection_t;

    Python::attach(|_py| {
        let tables_ptr = unsafe {
            pyo3::ffi::PyMem_Malloc(std::mem::size_of::<tsk_table_collection_t>())
                .cast::<tsk_table_collection_t>()
        };
        assert!(!tables_ptr.is_null());

        // SAFETY: ptr is not null
        let rv = unsafe { tsk_table_collection_init(tables_ptr, 0) };
        assert_eq!(rv, 0);
        let tables_ptr = std::ptr::NonNull::new(tables_ptr).unwrap();
        // Not null
        unsafe { (*tables_ptr.as_ptr()).sequence_length = 100.0 };
        // Not null and initialized w/o error
        let mut tables = unsafe { tskit::TableCollection::new_from_raw(tables_ptr) }.unwrap();
        let _ = tables
            .add_node(tskit::NodeFlags::IS_SAMPLE, 0.0, -1, -1)
            .unwrap();

        tables.build_index().unwrap();

        let treeseq = tables.tree_sequence(0).unwrap();

        let tables = treeseq.dump_tables().unwrap();

        let tables_ptr = tables.into_mut_ptr().unwrap();
        // Not null (the NonNull unwrapped on the previous line)
        let rv = unsafe { tsk_table_collection_free(tables_ptr.as_ptr()) };
        assert_eq!(rv, 0);

        // The pointer was allocated with PyMem_Malloc, so this is the correct free fn
        unsafe { pyo3::ffi::PyMem_Free(tables_ptr.as_ptr().cast::<std::ffi::c_void>()) };
    });
}
