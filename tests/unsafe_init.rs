//! This file tests the unsafe_init cargo feature,
//! which allows one to make table collections
//! and tree sequences from raw pointers.
//! This kind of thing is quite unsafe.
//!
//! The tests here attempt to illustrate what it takes
//! to uphold rust's expectations of memory safety.

use tskit::bindings::tsk_table_collection_free;
use tskit::bindings::tsk_table_collection_init;
use tskit::bindings::tsk_table_collection_t;

fn setup_empty_tables() -> std::ptr::NonNull<tsk_table_collection_t> {
    let ll = unsafe {
        libc::malloc(std::mem::size_of::<tsk_table_collection_t>()) as *mut tsk_table_collection_t
    };
    assert!(!ll.is_null());
    let code = unsafe { tsk_table_collection_init(ll, 0) };
    assert_eq!(code, 0);
    let mut nn = std::ptr::NonNull::new(ll).unwrap();
    unsafe { nn.as_mut() }.sequence_length = 100.0;
    nn
}

fn touch_tables(tables: &tskit::TableCollection) {
    let _ = tables.nodes();
    let _ = tables.edges();
    let _ = tables.sites();
    let _ = tables.mutations();
    let _ = tables.individuals();
}

#[test]
fn owning_tables() {
    let nn = setup_empty_tables();
    // This TableCollection assumes ownership of the pointer.
    // Since the pointer was allocated by malloc, it is valid
    // to let tables drop "naturally" out of scope.
    let tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    touch_tables(&tables);
}

#[test]
fn owning_tables_manual_teardown() {
    // For this test, pretend that nn contains a pointer allocated by something
    // like PyMem_Malloc (or really any allocator other than the system malloc).
    let nn = setup_empty_tables();
    let tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    touch_tables(&tables);
    // If the underlying pointer was allocated by anything other than malloc,
    // then we need to get access to the raw pointer...
    let mut nn = tables.into_mut_ptr().unwrap();
    // ...then tear it down using the tskit C API...
    let code = unsafe { tsk_table_collection_free(nn.as_mut()) };
    assert_eq!(code, 0);
    // ... and finally return the memory to the system
    unsafe { libc::free(nn.as_ptr() as *mut libc::c_void) }
}

#[test]
fn owning_tables_manual_teardown_via_ptr() {
    // For this test, pretend that nn contains a pointer allocated by something
    // like PyMem_Malloc (or really any allocator other than the system malloc).
    let nn = setup_empty_tables();
    let mut tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    touch_tables(&tables);
    // Note that we can access the pointer w/o consuming tables as the
    // above examples do!
    let code = unsafe { tsk_table_collection_free(tables.as_mut_ptr()) };
    assert_eq!(code, 0);
    // We MUST not do the next command, as it results in a double free
    // then tables drops out of scope!!!
    // unsafe { libc::free(tables.as_mut_ptr() as *mut libc::c_void) }
}

#[test]
fn nonowning_tables() {
    let nn = setup_empty_tables();
    let tables = unsafe { tskit::TableCollection::new_borrowed_from_raw(nn).unwrap() };
    // Here, tables does NOT OWN the pointer!
    // Therefore we need to do manual work to teardown data structures and free memory!
    assert_eq!(tables.sequence_length(), 100.);
    touch_tables(&tables);
    let mut nn = tables.into_mut_ptr().unwrap();
    let code = unsafe { tsk_table_collection_free(nn.as_mut()) };
    assert_eq!(code, 0);
    unsafe { libc::free(nn.as_ptr() as *mut libc::c_void) }
}

#[test]
fn nonowning_tables_teardown_via_ptr() {
    let nn = setup_empty_tables();
    let mut tables = unsafe { tskit::TableCollection::new_borrowed_from_raw(nn).unwrap() };
    // Here, tables does NOT OWN the pointer!
    // Therefore we need to do manual work to teardown data structures and free memory!
    assert_eq!(tables.sequence_length(), 100.);
    touch_tables(&tables);
    let code = unsafe { tsk_table_collection_free(tables.as_mut_ptr()) };
    assert_eq!(code, 0);
    unsafe { libc::free(tables.as_mut_ptr() as *mut libc::c_void) }
}
