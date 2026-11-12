use tskit::bindings::tsk_table_collection_init;
use tskit::bindings::tsk_table_collection_t;

#[test]
fn owning_tables() {
    let ll = unsafe {
        libc::malloc(std::mem::size_of::<tsk_table_collection_t>()) as *mut tsk_table_collection_t
    };
    let code = unsafe { tsk_table_collection_init(ll, 0) };
    assert_eq!(code, 0);
    let mut nn = std::ptr::NonNull::new(ll).unwrap();
    unsafe { nn.as_mut() }.sequence_length = 100.0;
    let tables = unsafe { tskit::TableCollection::new_from_raw(nn).unwrap() };
    assert_eq!(tables.sequence_length(), 100.);
}
