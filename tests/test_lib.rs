use tskit::*;

mod test_variables {
    // These tests basically make sure that we
    // can actually bind these things
    use super::*;

    #[test]
    fn test_node_is_sample() {
        let x = bindings::TSK_NODE_IS_SAMPLE;
        assert!(x > 0);
    }

    #[test]
    fn test_tsk_null() {
        assert_eq!(TSK_NULL, -1);
    }
}
