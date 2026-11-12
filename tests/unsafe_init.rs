use tskit::bindings::tsk_table_collection_free;
use tskit::bindings::tsk_table_collection_init;
use tskit::bindings::tsk_table_collection_t;

fn setup_empty_tables() -> std::ptr::NonNull<tsk_table_collection_t> {
    let ll = unsafe {
        libc::malloc(std::mem::size_of::<tsk_table_collection_t>()) as *mut tsk_table_collection_t
    };
    let code = unsafe { tsk_table_collection_init(ll, 0) };
    assert_eq!(code, 0);
    let mut nn = std::ptr::NonNull::new(ll).unwrap();
    unsafe { nn.as_mut() }.sequence_length = 100.0;
    nn
}

#[test]
fn owning_tables() {
    let nn = setup_empty_tables();
    let tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
}

#[test]
fn owning_tables_manual_teardown() {
    let nn = setup_empty_tables();
    let tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    let mut nn = tables.into_mut_ptr().unwrap();
    let code = unsafe { tsk_table_collection_free(nn.as_mut()) };
    assert_eq!(code, 0);
    unsafe { libc::free(nn.as_ptr() as *mut libc::c_void) }
}

#[test]
fn owning_tables_manual_teardown_via_ptr() {
    let nn = setup_empty_tables();
    let mut tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    let code = unsafe { tsk_table_collection_free(tables.as_mut_ptr()) };
    assert_eq!(code, 0);
    // We MUST not do the next command, as it results in a double free
    // unsafe { libc::free(tables.as_mut_ptr() as *mut libc::c_void) }
}

#[test]
fn nonowning_tables() {
    let nn = setup_empty_tables();
    let tables = unsafe { tskit::TableCollection::new_borrowed_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    let mut nn = tables.into_mut_ptr().unwrap();
    let code = unsafe { tsk_table_collection_free(nn.as_mut()) };
    assert_eq!(code, 0);
    unsafe { libc::free(nn.as_ptr() as *mut libc::c_void) }
}

#[test]
fn nonowning_tables_teardown_via_ptr() {
    let nn = setup_empty_tables();
    let mut tables = unsafe { tskit::TableCollection::new_borrowed_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
    let code = unsafe { tsk_table_collection_free(tables.as_mut_ptr()) };
    assert_eq!(code, 0);
    unsafe { libc::free(tables.as_mut_ptr() as *mut libc::c_void) }
}
