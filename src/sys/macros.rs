#![macro_use]

macro_rules! impl_tskteardown {
    ($tsk: ty, $teardown: expr) => {
        impl super::TskTeardown for $tsk {
            unsafe fn teardown(&mut self) -> i32 {
                $teardown(self as _)
            }
        }
    };
}

macro_rules! impl_id_traits {
    ($idtype: ty) => {
        impl $idtype {
            /// NULL value for this type.
            pub const NULL: $idtype = Self(super::TSK_NULL);

            /// Return `true` is `self == Self::NULL`
            pub fn is_null(&self) -> bool {
                *self == Self::NULL
            }

            /// Convenience function to convert to usize.
            ///
            /// Works via [`TryFrom`].
            ///
            /// # Returns
            ///
            /// * `None` if the underlying value is negative.
            /// * `Some(usize)` otherwise.
            pub fn to_usize(&self) -> Option<usize> {
                usize::try_from(*self).ok()
            }

            /// Convenience function to convert to usize.
            /// Implemented via `as`.
            /// Negative values with therefore wrap.
            pub fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }

        impl std::fmt::Display for $idtype {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self == Self::NULL {
                    false => write!(f, "{}", self.0),
                    true => write!(f, "NULL"),
                }
            }
        }

        impl From<super::bindings::tsk_id_t> for $idtype {
            fn from(value: super::bindings::tsk_id_t) -> Self {
                Self(value)
            }
        }

        impl TryFrom<$idtype> for usize {
            type Error = $crate::TskitError;
            fn try_from(value: $idtype) -> Result<Self, Self::Error> {
                match value.0.try_into() {
                    Ok(value) => Ok(value),
                    Err(_) => Err(crate::TskitError::RangeError(format!(
                        "could not convert {:?} to usize",
                        value
                    ))),
                }
            }
        }

        impl From<$idtype> for super::bindings::tsk_id_t {
            fn from(value: $idtype) -> Self {
                value.0
            }
        }

        impl TryFrom<$idtype> for SizeType {
            type Error = $crate::TskitError;

            fn try_from(value: $idtype) -> Result<Self, Self::Error> {
                SizeType::try_from(value.0)
            }
        }

        impl PartialEq<super::bindings::tsk_id_t> for $idtype {
            fn eq(&self, other: &super::bindings::tsk_id_t) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$idtype> for super::bindings::tsk_id_t {
            fn eq(&self, other: &$idtype) -> bool {
                *self == other.0
            }
        }

        impl PartialOrd<super::bindings::tsk_id_t> for $idtype {
            fn partial_cmp(&self, other: &super::bindings::tsk_id_t) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$idtype> for super::bindings::tsk_id_t {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl Default for $idtype {
            fn default() -> Self {
                Self::NULL
            }
        }
    };
}

macro_rules! impl_size_type_comparisons_for_row_ids {
    ($idtype: ty) => {
        impl PartialEq<$idtype> for SizeType {
            fn eq(&self, other: &$idtype) -> bool {
                self.0 == other.0 as super::bindings::tsk_size_t
            }
        }

        impl PartialEq<SizeType> for $idtype {
            fn eq(&self, other: &SizeType) -> bool {
                (self.0 as super::bindings::tsk_size_t) == other.0
            }
        }

        impl PartialOrd<$idtype> for SizeType {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.0
                    .partial_cmp(&(other.0 as super::bindings::tsk_size_t))
            }
        }

        impl PartialOrd<SizeType> for $idtype {
            fn partial_cmp(&self, other: &SizeType) -> Option<std::cmp::Ordering> {
                (self.0 as super::bindings::tsk_size_t).partial_cmp(&other.0)
            }
        }
    };
}

macro_rules! impl_f64_newtypes {
    ($type: ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl PartialEq<f64> for $type {
            fn eq(&self, other: &f64) -> bool {
                self.0.eq(other)
            }
        }

        impl PartialEq<$type> for f64 {
            fn eq(&self, other: &$type) -> bool {
                self.eq(&other.0)
            }
        }

        impl PartialOrd<f64> for $type {
            fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$type> for f64 {
            fn partial_cmp(&self, other: &$type) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl From<f64> for $type {
            fn from(value: f64) -> Self {
                Self(value)
            }
        }

        impl From<$type> for f64 {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl std::ops::Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0
            }
        }

        impl std::ops::Add for $type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0
            }
        }

        impl std::ops::Mul for $type {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl std::ops::MulAssign for $type {
            fn mul_assign(&mut self, rhs: Self) {
                self.0.mul_assign(&rhs.0)
            }
        }

        impl std::ops::Div for $type {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl std::ops::DivAssign for $type {
            fn div_assign(&mut self, rhs: Self) {
                self.0.div_assign(&rhs.0)
            }
        }
    };
}

macro_rules! impl_time_position_arithmetic {
    ($lhs: ty, $rhs:ty) => {
        impl std::ops::Mul<$rhs> for $lhs {
            type Output = $lhs;

            fn mul(self, other: $rhs) -> Self {
                Self(self.0.mul(&other.0))
            }
        }

        impl std::ops::MulAssign<$rhs> for $lhs {
            fn mul_assign(&mut self, other: $rhs) {
                self.0.mul_assign(&other.0)
            }
        }

        impl std::ops::Div<$rhs> for $lhs {
            type Output = $lhs;

            fn div(self, other: $rhs) -> Self {
                Self(self.0.div(&other.0))
            }
        }

        impl std::ops::DivAssign<$rhs> for $lhs {
            fn div_assign(&mut self, other: $rhs) {
                self.0.div_assign(&other.0)
            }
        }
    };
}

macro_rules! safe_tsk_column_access {
    ($self: ident, $u: ident, $output_type: ty, $column: ident) => {{
        assert!($self.as_ref().num_rows == 0 || !$self.as_ref().$column.is_null());
        // SAFETY: row is empty or is that the column is not a null pointer
        // and the length is from the C API
        unsafe {
            super::tsk_column_access::<$output_type, _, _, _>(
                $u,
                $self.as_ref().$column,
                $self.as_ref().num_rows,
            )
        }
    }};
}

macro_rules! raw_metadata_getter_for_tables {
    ($idtype: ty) => {
        fn metadata_column(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(
                    self.as_ref().metadata.cast::<u8>(),
                    self.as_ref().metadata_length as usize,
                )
            }
        }

        fn metadata_offset_raw(&self) -> &[super::bindings::tsk_size_t] {
            unsafe {
                std::slice::from_raw_parts(
                    self.as_ref().metadata_offset,
                    self.as_ref().num_rows as usize,
                )
            }
        }

        pub fn raw_metadata<I: Into<$idtype>>(&self, row: I) -> Option<&[u8]> {
            $crate::sys::tsk_ragged_column_access(
                row.into(),
                self.metadata_column(),
                self.metadata_offset_raw(),
            )
        }
    };
}
