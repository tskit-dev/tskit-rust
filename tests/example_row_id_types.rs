use tskit::prelude::*;

fn row_id_examples() {
    // ANCHOR: create_node_id
    let n = NodeId::from(0);

    // The ID type can be compared to the "raw" integer type
    assert_eq!(n, 0_i32);
    // The ID types have some associated functions.
    // For example, to check if an ID is null:
    assert!(!n.is_null());
    // ANCHOR_END: create_node_id

    // ANCHOR: create_null_node_id
    let n = NodeId::NULL;
    assert_eq!(n, -1_i32);
    assert!(n.is_null());
    // ANCHOR_END: create_null_node_id
}

fn vector_row_id_examples() -> Vec<NodeId> {
    // ANCHOR: create_vec_node_id
    let mut v = vec![];

    for i in 0..5_i32 {
        v.push(NodeId::from(i));
    }
    // ANCHOR_END: create_vec_node_id
    v
}

// ANCHOR: mock_rust_fn
fn process_vec_node_id(_: &[NodeId]) {}
// ANCHOR_END: mock_rust_fn

// ANCHOR: mock_tsk_fn
extern "C" fn tsk_foo(_: *const tskit::bindings::tsk_id_t, _: tskit::bindings::tsk_size_t) {}
// ANCHOR_END: mock_tsk_fn

// ANCHOR: mock_tsk_fn_mut
extern "C" fn tsk_foo2(_: *mut tskit::bindings::tsk_id_t, _: tskit::bindings::tsk_size_t) {}
// ANCHOR_END: mock_tsk_fn_mut

#[test]
fn test_row_id_examples() {
    row_id_examples();
}

#[test]
fn test_vector_row_id_examples() {
    let mut v = vector_row_id_examples();

    // ANCHOR: call_mock_rust_fn
    process_vec_node_id(&v);
    // ANCHOR_END: call_mock_rust_fn

    // ANCHOR: call_mock_tsk_fn
    tsk_foo(
        v.as_ptr() as *const tskit::bindings::tsk_id_t,
        v.len() as tskit::bindings::tsk_size_t,
    );
    // ANCHOR_END: call_mock_tsk_fn

    // ANCHOR: call_mock_tsk_fn_mut
    tsk_foo2(
        v.as_mut_ptr() as *mut tskit::bindings::tsk_id_t,
        v.len() as tskit::bindings::tsk_size_t,
    );
    // ANCHOR_END: call_mock_tsk_fn_mut
}
