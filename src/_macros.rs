#![macro_use]

#[doc(hidden)]
macro_rules! handle_tsk_return_value {
    ($code: expr) => {{
        if $code < 0 {
            return Err($crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($code)
    }};
    ($code: expr, $return_value: expr) => {{
        if $code < 0 {
            return Err($crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($return_value)
    }};
}

macro_rules! panic_on_tskit_error {
    ($code: expr) => {
        if $code < 0 {
            let c_str =
                unsafe { std::ffi::CStr::from_ptr($crate::sys::bindings::tsk_strerror($code)) };
            let str_slice: &str = c_str.to_str().expect("failed to obtain &str from c_str");
            let message: String = str_slice.to_owned();
            panic!("{}", message);
        }
    };
}

macro_rules! decode_metadata_row {
    ($T: ty, $buffer: expr) => {
        <$T as $crate::metadata::MetadataRoundtrip>::decode($buffer)
    };
}

macro_rules! err_if_not_tracking_samples {
    ($flags: expr, $rv: expr) => {
        match $flags.contains($crate::TreeFlags::SAMPLE_LISTS) {
            false => Err(TskitError::NotTrackingSamples),
            true => Ok($rv),
        }
    };
}

// This macro assumes that table row access helper
// functions have a standard interface.
// Here, we convert the None type to an Error,
// as it implies $row is out of range.
macro_rules! table_row_access {
    ($row: expr, $table: expr, $row_fn: ident) => {
        $row_fn($table, $row)
    };
}

/// Convenience macro to handle implementing
/// [`crate::metadata::MetadataRoundtrip`]
#[macro_export]
macro_rules! handle_metadata_return {
    ($e: expr) => {
        match $e {
            Ok(x) => Ok(x),
            Err(e) => Err($crate::metadata::MetadataError::RoundtripError { value: Box::new(e) }),
        }
    };
}

macro_rules! row_lending_iterator_get {
    () => {
        fn get(&self) -> Option<&Self::Item> {
            if crate::SizeType::try_from(self.id).ok()? < self.table.num_rows() {
                Some(self)
            } else {
                None
            }
        }
    };
}

macro_rules! optional_container_comparison {
    ($lhs: expr, $rhs: expr) => {
        if let Some(value) = &$lhs {
            if let Some(ovalue) = &$rhs {
                if value.len() != ovalue.len() {
                    return false;
                }
                if value.iter().zip(ovalue.iter()).any(|(a, b)| a != b) {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else if $rhs.is_some() {
            false
        } else {
            true
        }
    };
}

macro_rules! build_table_column_slice_getter {
    ($(#[$attr:meta])* => $column: ident, $name: ident, $cast: ty) => {
        $(#[$attr])*
        pub fn $name(&self) -> &[$cast] {
            // SAFETY: all array lengths are the number of rows in the table
            unsafe{$crate::sys::generate_slice(self.as_ref().$column, self.num_rows())}
        }
    };
}

macro_rules! build_table_column_slice_mut_getter {
    ($(#[$attr:meta])* => $column: ident, $name: ident, $cast: ty) => {
        $(#[$attr])*
        pub fn $name(&mut self) -> &mut [$cast] {
            // SAFETY: all array lengths are the number of rows in the table
            unsafe{$crate::sys::generate_slice_mut(self.as_ref().$column, self.num_rows())}
        }
    };
}

#[cfg(test)]
mod test {
    use crate::error::TskitError;
    use crate::TskReturnValue;

    #[test]
    #[should_panic]
    fn test_tskit_panic() {
        panic_on_tskit_error!(-202); // "Node out of bounds"
    }

    fn return_value_mock(rv: i32) -> TskReturnValue {
        handle_tsk_return_value!(rv)
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
