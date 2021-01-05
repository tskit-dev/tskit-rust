use tskit::*;

mod test_macros {
    use super::*;
    #[test]
    #[should_panic]
    fn test_tskit_panic() {
        panic_on_tskit_error!(-202); // "Node out of bounds"
    }

    fn return_value_mock(rv: i32) -> TskReturnValue {
        handle_tsk_return_value!(rv);
    }

    fn must_not_error(x: TskReturnValue) -> bool {
        x.map_or_else(|_: TskitError| false, |_| true)
    }

    fn must_error(x: TskReturnValue) -> bool {
        x.map_or_else(|_: TskitError| true, |_| false)
    }

    #[test]
    fn test_handle_good_return_value() {
        assert!(must_not_error(return_value_mock(0)));
        assert!(must_not_error(return_value_mock(1)));
    }

    #[test]
    fn test_handle_return_value_test_panic() {
        assert!(must_error(return_value_mock(-207)));
    }
}
