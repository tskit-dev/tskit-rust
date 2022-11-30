#[test]
fn test_node_id_as_usize() {
    let x = tskit::NodeId::from(0);
    assert_eq!(x.to_usize(), Some(0_usize));
    assert_eq!(x.as_usize(), 0_usize);
    let x = tskit::NodeId::from(-1);
    assert_eq!(x.to_usize(), None);
    assert_eq!(x.as_usize(), usize::MAX);
    let x = tskit::NodeId::from(-2);
    assert_eq!(x.as_usize(), -2_i32 as usize);
}
