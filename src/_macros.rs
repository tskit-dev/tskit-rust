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
            let c_str = unsafe { std::ffi::CStr::from_ptr($crate::bindings::tsk_strerror($code)) };
            let str_slice: &str = c_str.to_str().unwrap();
            let message: String = str_slice.to_owned();
            panic!("{}", message);
        }
    };
}

macro_rules! unsafe_tsk_column_access {
    ($i: expr, $lo: expr, $hi: expr, $array: expr) => {{
        if $i < $lo || ($i as $crate::tsk_size_t) >= $hi {
            Err($crate::error::TskitError::IndexError {})
        } else {
            Ok(unsafe { *$array.offset($i as isize) })
        }
    }};
    ($i: expr, $lo: expr, $hi: expr, $array: expr, $output_id_type: expr) => {{
        if $i < $lo || ($i as $crate::tsk_size_t) >= $hi {
            Err($crate::error::TskitError::IndexError {})
        } else {
            Ok($output_id_type(unsafe { *$array.offset($i as isize) }))
        }
    }};
}

macro_rules! unsafe_tsk_ragged_column_access {
    ($i: expr, $lo: expr, $hi: expr, $array: expr, $offset_array: expr, $offset_array_len: expr) => {{
        if $i < $lo || $i >= ($hi as tsk_id_t) {
            Err(TskitError::IndexError {})
        } else if $offset_array_len == 0 {
            Ok(None)
        } else {
            let start = unsafe { *$offset_array.offset($i as isize) };
            let stop = if $i < ($hi as tsk_id_t) {
                unsafe { *$offset_array.offset(($i + 1) as isize) }
            } else {
                $offset_array_len as tsk_size_t
            };
            if start == stop {
                Ok(None)
            } else {
                let mut buffer = vec![];
                for i in start..stop {
                    buffer.push(unsafe { *$array.offset(i as isize) });
                }
                Ok(Some(buffer))
            }
        }
    }};

    ($i: expr, $lo: expr, $hi: expr, $array: expr, $offset_array: expr, $offset_array_len: expr, $output_id_type: expr) => {{
        if $i < $lo || $i >= ($hi as tsk_id_t) {
            Err(TskitError::IndexError {})
        } else if $offset_array_len == 0 {
            Ok(None)
        } else {
            let start = unsafe { *$offset_array.offset($i as isize) };
            let stop = if $i < ($hi as tsk_id_t) {
                unsafe { *$offset_array.offset(($i + 1) as isize) }
            } else {
                $offset_array_len as tsk_size_t
            };
            if start == stop {
                Ok(None)
            } else {
                let mut buffer = vec![];
                for i in start..stop {
                    buffer.push($output_id_type(unsafe { *$array.offset(i as isize) }));
                }
                Ok(Some(buffer))
            }
        }
    }};
}

// Allow this to be unused for default features
// to pass clippy checks
#[allow(unused_macros)]
macro_rules! unsafe_tsk_ragged_char_column_access {
    ($i: expr, $lo: expr, $hi: expr, $array: expr, $offset_array: expr, $offset_array_len: expr) => {{
        if $i < $lo || $i >= ($hi as tsk_id_t) {
            Err(TskitError::IndexError {})
        } else if $offset_array_len == 0 {
            Ok(None)
        } else {
            let start = unsafe { *$offset_array.offset($i as isize) };
            let stop = if $i < ($hi as tsk_id_t) {
                unsafe { *$offset_array.offset(($i + 1) as isize) }
            } else {
                $offset_array_len as tsk_size_t
            };
            if start == stop {
                Ok(None)
            } else {
                let mut buffer = String::new();
                for i in start..stop {
                    buffer.push(unsafe { *$array.offset(i as isize) as u8 as char });
                }
                Ok(Some(buffer))
            }
        }
    }};
}

macro_rules! drop_for_tskit_type {
    ($name: ident, $drop: ident) => {
        impl Drop for $name {
            fn drop(&mut self) {
                let rv = unsafe { $drop(&mut *self.inner) };
                panic_on_tskit_error!(rv);
                unsafe {
                    libc::free(self.inner as *mut libc::c_void);
                }
                self.inner = std::ptr::null_mut();
            }
        }
    };
}

macro_rules! tskit_type_access {
    ($name: ident, $ll_name: ty) => {
        impl $crate::TskitTypeAccess<$ll_name> for $name {
            fn as_ptr(&self) -> *const $ll_name {
                self.inner
            }

            fn as_mut_ptr(&mut self) -> *mut $ll_name {
                self.inner
            }
        }
    };
}

macro_rules! build_tskit_type {
    ($name: ident, $ll_name: ty, $drop: ident) => {
        impl $crate::ffi::WrapTskitType<$ll_name> for $name {
            fn wrap() -> Self {
                let temp =
                    unsafe { libc::malloc(std::mem::size_of::<$ll_name>()) as *mut $ll_name };
                if temp.is_null() {
                    panic!("out of memory");
                }
                $name { inner: temp }
            }
        }
        drop_for_tskit_type!($name, $drop);
        tskit_type_access!($name, $ll_name);
    };
}

macro_rules! metadata_to_vector {
    ($self: expr, $row: expr) => {
        $crate::metadata::char_column_to_vector(
            $self.table_.metadata,
            $self.table_.metadata_offset,
            $row,
            $self.table_.num_rows,
            $self.table_.metadata_length,
        )
    };
}

macro_rules! decode_metadata_row {
    ($T: ty, $buffer: expr) => {
        match $buffer {
            None => Ok(None),
            Some(v) => Ok(Some(<$T as $crate::metadata::MetadataRoundtrip>::decode(
                &v,
            )?)),
        }
    };
}

macro_rules! table_row_decode_metadata {
    ($table: ident, $pos: ident) => {
        metadata_to_vector!($table, $pos).unwrap().map(|x| x)
    };
}

macro_rules! process_state_input {
    ($state: expr) => {
        match $state {
            Some(x) => (
                x.as_ptr() as *const libc::c_char,
                x.len() as $crate::bindings::tsk_size_t,
                $state,
            ),
            None => (std::ptr::null(), 0, $state),
        }
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
// as it applies $row is out of range.
macro_rules! table_row_access {
    ($row: expr, $table: expr, $row_fn: ident) => {
        if $row < 0 {
            Err(TskitError::IndexError)
        } else {
            match $row_fn($table, $row) {
                Some(x) => Ok(x),
                None => Err(TskitError::IndexError),
            }
        }
    };
}

macro_rules! iterator_for_nodeiterator {
    ($ty: ty) => {
        impl Iterator for $ty {
            type Item = $crate::NodeId;
            fn next(&mut self) -> Option<Self::Item> {
                self.next_node();
                self.current_node()
            }
        }
    };
}

macro_rules! tree_array_slice {
    ($self: ident, $array: ident, $len: expr) => {
        unsafe {
            std::slice::from_raw_parts(
                (*$self.as_ptr()).$array as *const $crate::NodeId,
                $len as usize,
            )
        }
    };
}

macro_rules! impl_id_traits {
    ($idtype: ty) => {
        impl $idtype {
            /// NULL value for this type.
            pub const NULL: $idtype = Self($crate::TSK_NULL);

            /// Return `true` is `self == Self::NULL`
            pub fn is_null(&self) -> bool {
                *self == Self::NULL
            }
        }

        impl std::fmt::Display for $idtype {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self == Self::NULL {
                    false => write!(f, "{}({})", stringify!($idtype), self.0),
                    true => write!(f, "{}(NULL)", stringify!($idtype)),
                }
            }
        }

        impl From<$crate::tsk_id_t> for $idtype {
            fn from(value: $crate::tsk_id_t) -> Self {
                Self(value)
            }
        }

        impl From<$idtype> for usize {
            fn from(value: $idtype) -> Self {
                value.0 as usize
            }
        }

        impl From<$idtype> for $crate::tsk_id_t {
            fn from(value: $idtype) -> Self {
                value.0
            }
        }

        impl PartialEq<$crate::tsk_id_t> for $idtype {
            fn eq(&self, other: &$crate::tsk_id_t) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$idtype> for $crate::tsk_id_t {
            fn eq(&self, other: &$idtype) -> bool {
                *self == other.0
            }
        }

        impl PartialOrd<$crate::tsk_id_t> for $idtype {
            fn partial_cmp(&self, other: &$crate::tsk_id_t) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$idtype> for $crate::tsk_id_t {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }
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
