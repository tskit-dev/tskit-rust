#![macro_use]

#[doc(hidden)]
macro_rules! handle_tsk_return_value {
    ($code: expr) => {{
        if $code < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($code)
    }};
    ($code: expr, $return_value: expr) => {{
        if $code < 0 {
            return Err(crate::error::TskitError::ErrorCode { code: $code });
        }
        Ok($return_value)
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
            return Err(crate::error::TskitError::IndexError {});
        }
        return Ok(unsafe { *$array.offset($i as isize) });
    }};
}

macro_rules! drop_for_tskit_type {
    ($name: ident, $drop: ident) => {
        impl Drop for $name {
            fn drop(&mut self) {
                let rv = unsafe { $drop(&mut *self.inner) };
                panic_on_tskit_error!(rv);
            }
        }
    };
}

macro_rules! tskit_type_access {
    ($name: ident, $ll_name: ty) => {
        impl crate::ffi::TskitTypeAccess<$ll_name> for $name {
            fn as_ptr(&self) -> *const $ll_name {
                &*self.inner
            }

            fn as_mut_ptr(&mut self) -> *mut $ll_name {
                &mut *self.inner
            }
        }
    };
}

macro_rules! build_tskit_type {
    ($name: ident, $ll_name: ty, $drop: ident) => {
        impl crate::ffi::WrapTskitType<$ll_name> for $name {
            fn wrap() -> Self {
                let temp: std::mem::MaybeUninit<$ll_name> = std::mem::MaybeUninit::uninit();
                $name {
                    inner: unsafe { Box::<$ll_name>::new(temp.assume_init()) },
                }
            }
        }
        drop_for_tskit_type!($name, $drop);
        tskit_type_access!($name, $ll_name);
    };
}

macro_rules! build_consuming_tskit_type {
    ($name: ident, $ll_name: ty, $drop: ident, $consumed: ty) => {
        impl crate::ffi::WrapTskitConsumingType<$ll_name, $consumed> for $name {
            fn wrap(consumed: $consumed) -> Self {
                let temp: std::mem::MaybeUninit<$ll_name> = std::mem::MaybeUninit::uninit();
                $name {
                    consumed,
                    inner: unsafe { Box::<$ll_name>::new(temp.assume_init()) },
                }
            }
        }
        tskit_type_access!($name, $ll_name);
        drop_for_tskit_type!($name, $drop);
    };
}

macro_rules! metadata_to_vector {
    ($T: ty, $self: expr, $row: expr) => {
        crate::metadata::char_column_to_vector(
            $self.table_.metadata,
            $self.table_.metadata_offset,
            $row,
            $self.table_.num_rows,
            $self.table_.metadata_length,
        )?
    };
}

macro_rules! decode_metadata_row {
    ($T: ty, $buffer: expr) => {
        match $buffer {
            None => Ok(None),
            Some(v) => Ok(Some(<$T as crate::metadata::MetadataRoundtrip>::decode(
                &v,
            )?)),
        }
    };
}

macro_rules! process_state_input {
    ($state: expr) => {
        match $state {
            Some(x) => (
                x.as_ptr() as *const libc::c_char,
                x.len() as crate::bindings::tsk_size_t,
                $state,
            ),
            None => (std::ptr::null(), 0, $state),
        }
    };
}

macro_rules! index_for_wrapped_tsk_array_type {
    ($name: ty, $index:ty, $output: ty) => {
        impl std::ops::Index<$index> for $name {
            type Output = $output;

            fn index(&self, index: $index) -> &Self::Output {
                if index >= self.len() as $index {
                    panic!("fatal: index out of range");
                }
                let rv = unsafe { self.array.offset(index as isize) };
                unsafe { &*rv }
            }
        }
    };
}

macro_rules! iterator_for_wrapped_tsk_array_type {
    ($name: ty, $item: ty) => {
        impl Iterator for $name {
            type Item = $item;

            fn next(&mut self) -> Option<$item> {
                if self.idx_ >= self.len_ as crate::tsk_id_t {
                    self.idx_ = -1;
                }
                self.idx_ += 1;

                if self.idx_ < self.len_ as crate::tsk_id_t {
                    Some(unsafe { *self.array.offset(self.idx_ as isize) })
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! wrapped_tsk_array_traits {
    ($name: ty, $index:ty, $output: ty) => {
        index_for_wrapped_tsk_array_type!($name, $index, $output);
        iterator_for_wrapped_tsk_array_type!($name, $output);
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
