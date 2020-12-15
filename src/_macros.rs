#![macro_use]

#[doc(hidden)]
macro_rules! handle_tsk_return_value {
    ($code: expr) => {{
        if $code < 0 {
            return Err(crate::error::TskitRustError::ErrorCode { code: $code });
        }
        return Ok($code);
    }};
}

macro_rules! panic_on_tskit_error {
    ($code: expr) => {
        if $code < 0 {
            let c_str = unsafe { std::ffi::CStr::from_ptr(crate::bindings::tsk_strerror($code)) };
            let str_slice: &str = c_str.to_str().unwrap();
            let message: String = str_slice.to_owned();
            panic!("{}", message);
        }
    };
}

macro_rules! unsafe_tsk_column_access {
    ($i: expr, $lo: expr, $hi: expr, $array: expr) => {{
        if $i < $lo || ($i as crate::tsk_size_t) >= $hi {
            return Err(crate::error::TskitRustError::IndexError {});
        }
        return Ok(unsafe { *$array.offset($i as isize) });
    }};
}

#[cfg(test)]
mod test {
    use crate::error::TskitRustError;
    use crate::TskReturnValue;

    #[test]
    #[should_panic]
    fn test_tskit_panic() {
        panic_on_tskit_error!(-202); // "Node out of bounds"
    }

    fn return_value_mock(rv: i32) -> TskReturnValue {
        handle_tsk_return_value!(rv);
    }

    fn must_not_error(x: TskReturnValue) -> bool {
        return x.map_or_else(|_: TskitRustError| false, |_| true);
    }

    fn must_error(x: TskReturnValue) -> bool {
        return x.map_or_else(|_: TskitRustError| true, |_| false);
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
