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
            let str_slice: &str = c_str.to_str().expect("failed to obtain &str from c_str");
            let message: String = str_slice.to_owned();
            panic!("{}", message);
        }
    };
}

macro_rules! unsafe_tsk_column_access {
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident) => {{
        if $i < $lo || ($i as $crate::tsk_size_t) >= $hi {
            None
        } else {
            debug_assert!(!($owner).$array.is_null());
            if !$owner.$array.is_null() {
                // SAFETY: array is not null
                // and we did our best effort
                // on bounds checking
                Some(unsafe { *$owner.$array.offset($i as isize) })
            } else {
                None
            }
        }
    }};
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident, $output_id_type: expr) => {{
        if $i < $lo || ($i as $crate::tsk_size_t) >= $hi {
            None
        } else {
            debug_assert!(!($owner).$array.is_null());
            if !$owner.$array.is_null() {
                // SAFETY: array is not null
                // and we did our best effort
                // on bounds checking
                unsafe { Some($output_id_type(*($owner.$array.offset($i as isize)))) }
            } else {
                None
            }
        }
    }};
}

macro_rules! unsafe_tsk_column_access_and_map_into {
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident) => {{
        unsafe_tsk_column_access!($i, $lo, $hi, $owner, $array).map(|v| v.into())
    }};
}

macro_rules! unsafe_tsk_ragged_column_access {
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident, $offset_array: ident, $offset_array_len: ident, $output_id_type: ty) => {{
        let i = $crate::SizeType::try_from($i).ok()?;
        if $i < $lo || i >= $hi {
            None
        } else if $owner.$offset_array_len == 0 {
            None
        } else {
            debug_assert!(!$owner.$array.is_null());
            if $owner.$array.is_null() {
                return None;
            }
            // SAFETY: we have checked bounds and ensured not null
            let start = unsafe { *$owner.$offset_array.offset($i as isize) };
            let stop = if i < $hi {
                unsafe { *$owner.$offset_array.offset(($i + 1) as isize) }
            } else {
                $owner.$offset_array_len as tsk_size_t
            };
            if start == stop {
                None
            } else {
                Some(unsafe {
                    std::slice::from_raw_parts(
                        $owner.$array.offset(start as isize) as *const $output_id_type,
                        stop as usize - start as usize,
                    )
                })
            }
        }
    }};
}

// Allow this to be unused for default features
// to pass clippy checks
#[allow(unused_macros)]
macro_rules! unsafe_tsk_ragged_char_column_access {
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident, $offset_array: ident, $offset_array_len: ident) => {{
        let i = $crate::SizeType::try_from($i).ok()?;
        if $i < $lo || i >= $hi {
            None
        } else if $owner.$offset_array_len == 0 {
            None
        } else {
            assert!(!$owner.$array.is_null());
            assert!(!$owner.$offset_array.is_null());
            let start = unsafe { *$owner.$offset_array.offset($i as isize) };
            let stop = if i < $hi {
                unsafe { *$owner.$offset_array.offset(($i + 1) as isize) }
            } else {
                $owner.$offset_array_len as tsk_size_t
            };
            if start == stop {
                None
            } else {
                let mut buffer = String::new();
                for i in start..stop {
                    buffer.push(unsafe { *$owner.$array.offset(i as isize) as u8 as char });
                }
                Some(buffer)
            }
        }
    }};
}

#[cfg(feature = "provenance")]
macro_rules! unsafe_tsk_ragged_char_column_access_to_slice_u8 {
    ($i: expr, $lo: expr, $hi: expr, $owner: expr, $array: ident, $offset_array: ident, $offset_array_len: ident) => {{
        let i = match $crate::SizeType::try_from($i).ok() {
            Some(j) => j,
            None => $crate::SizeType::from(u64::MAX),
        };
        if $i < $lo || i >= $hi {
            None
        } else if $owner.$offset_array_len == 0 {
            None
        } else {
            assert!(!$owner.$array.is_null());
            assert!(!$owner.$offset_array.is_null());
            let start = unsafe { *$owner.$offset_array.offset($i as isize) };
            let stop = if i < $hi {
                unsafe { *$owner.$offset_array.offset(($i + 1) as isize) }
            } else {
                $owner.$offset_array_len as tsk_size_t
            };
            if start == stop {
                None
            } else {
                let ptr = unsafe { $owner.$array.offset(start as isize) as *const u8 };
                let len = (stop - start) as usize;
                let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
                Some(slice)
            }
        }
    }};
}

macro_rules! metadata_to_vector {
    ($outer: ident, $table: expr, $row: expr) => {
        $crate::metadata::char_column_to_slice(
            $outer,
            $table.metadata,
            $table.metadata_offset,
            $row,
            $table.num_rows,
            $table.metadata_length,
        )
    };
}

macro_rules! decode_metadata_row {
    ($T: ty, $buffer: expr) => {
        <$T as $crate::metadata::MetadataRoundtrip>::decode($buffer)
    };
}

macro_rules! table_row_decode_metadata {
    ($owner: ident, $table: ident, $pos: ident) => {
        metadata_to_vector!($owner, $table, $pos)
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
// as it implies $row is out of range.
macro_rules! table_row_access {
    ($row: expr, $table: expr, $row_fn: ident) => {
        $row_fn($table, $row)
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
                    false => write!(f, "{}", self.0),
                    true => write!(f, "NULL"),
                }
            }
        }

        impl From<$crate::tsk_id_t> for $idtype {
            fn from(value: $crate::tsk_id_t) -> Self {
                Self(value)
            }
        }

        impl TryFrom<$idtype> for usize {
            type Error = crate::TskitError;
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

        impl From<$idtype> for $crate::tsk_id_t {
            fn from(value: $idtype) -> Self {
                value.0
            }
        }

        impl TryFrom<$idtype> for $crate::SizeType {
            type Error = $crate::TskitError;

            fn try_from(value: $idtype) -> Result<Self, Self::Error> {
                $crate::SizeType::try_from(value.0)
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

macro_rules! impl_size_type_comparisons_for_row_ids {
    ($idtype: ty) => {
        impl PartialEq<$idtype> for $crate::SizeType {
            fn eq(&self, other: &$idtype) -> bool {
                self.0 == other.0 as $crate::bindings::tsk_size_t
            }
        }

        impl PartialEq<$crate::SizeType> for $idtype {
            fn eq(&self, other: &$crate::SizeType) -> bool {
                (self.0 as $crate::bindings::tsk_size_t) == other.0
            }
        }

        impl PartialOrd<$idtype> for $crate::SizeType {
            fn partial_cmp(&self, other: &$idtype) -> Option<std::cmp::Ordering> {
                self.0
                    .partial_cmp(&(other.0 as $crate::bindings::tsk_size_t))
            }
        }

        impl PartialOrd<$crate::SizeType> for $idtype {
            fn partial_cmp(&self, other: &$crate::SizeType) -> Option<std::cmp::Ordering> {
                (self.0 as $crate::bindings::tsk_size_t).partial_cmp(&other.0)
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

macro_rules! impl_from_for_flag_types {
    ($flagstype: ty) => {
        impl From<$crate::RawFlags> for $flagstype {
            fn from(value: $crate::RawFlags) -> Self {
                <$flagstype>::from_bits_truncate(value)
            }
        }
    };
}

macro_rules! impl_flags {
    ($flagstype: ty) => {
        impl $flagstype {
            /// We do not enforce valid flags in the library.
            /// This function will return `true` if any bits
            /// are set that do not correspond to allowed flags.
            pub fn is_valid(&self) -> bool {
                Self::from_bits(self.bits()).is_some()
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

macro_rules! build_owned_tables {
    ($name: ty, $deref: ident, $llname: ty, $init: ident, $free: ident, $clear: expr) => {
        impl $name {
            fn new() -> Self {
                let temp = unsafe { libc::malloc(std::mem::size_of::<$llname>()) as *mut $llname };
                let nonnull = match std::ptr::NonNull::<$llname>::new(temp) {
                    Some(x) => x,
                    None => panic!("out of memory"),
                };
                let mut table = unsafe { mbox::MBox::from_non_null_raw(nonnull) };
                let rv = unsafe { $init(&mut (*table), 0) };
                assert_eq!(rv, 0);
                Self { table }
            }

            /// Clear the table.
            pub fn clear(&mut self) -> $crate::TskReturnValue {
                let rv = unsafe { $clear(self.as_mut_ptr()) };
                handle_tsk_return_value!(rv)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = $deref;

            fn deref(&self) -> &Self::Target {
                // SAFETY: that T* and &T have same layout,
                // and Target is repr(transparent).
                unsafe { std::mem::transmute(&self.table) }
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                // SAFETY: that T* and &T have same layout,
                // and Target is repr(transparent).
                unsafe { std::mem::transmute(&mut self.table) }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                let rv = unsafe { $free(&mut (*self.table)) };
                if rv != 0 {
                    panic!("error when calling {}: {}", stringify!(free), rv);
                }
            }
        }

        impl $name {
            pub fn as_ptr(&self) -> *const $llname {
                &*self.table
            }

            pub fn as_mut_ptr(&mut self) -> *mut $llname {
                &mut *self.table as *mut $llname
            }
        }
    };
}

macro_rules! node_table_add_row_details {
    ($flags: ident,
     $time: ident,
     $population: ident,
     $individual: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let rv = unsafe {
            $crate::bindings::tsk_node_table_add_row(
                &mut $table,
                $flags.into().bits(),
                $time.into().0,
                $population.into().0,
                $individual.into().0,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! node_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<F,T,P,I>(
            &mut $self,
            flags: F ,
            time: T ,
            population: P ,
            individual: I ,
        ) -> Result<$crate::NodeId, $crate::TskitError>
        where
            F: Into<$crate::NodeFlags>,
            T:Into<$crate::Time>,
            P:Into<$crate::PopulationId>,
            I:Into<$crate::IndividualId>
        {
            node_table_add_row_details!(flags,
                                        time,
                                        population,
                                        individual,
                                        std::ptr::null(),
                                        0,
                                        $table)
        }
    };
}

macro_rules! node_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<F,T,P,I,M>(
            &mut $self,
            flags: F,
            time: T,
            population: P,
            individual: I,
            metadata: &M,
        ) -> Result<$crate::NodeId, $crate::TskitError>
        where
            F: Into<$crate::NodeFlags>,
            T:Into<$crate::Time>,
            P:Into<$crate::PopulationId>,
            I:Into<$crate::IndividualId>,
            M: $crate::metadata::NodeMetadata,
        {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            node_table_add_row_details!(flags,
                                        time,
                                        population,
                                        individual,
                                        md.as_ptr(),
                                        md.len().into(),
                                        $table)
        }
    };
}

macro_rules! edge_table_add_row_details {
    ($left: ident,
     $right: ident,
     $parent: ident,
     $child: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let rv = unsafe {
            $crate::bindings::tsk_edge_table_add_row(
                &mut $table,
                $left.into().0,
                $right.into().0,
                $parent.into().0,
                $child.into().0,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! edge_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<L,R,P,C>(
            &mut $self,
            left: L,
            right: R,
            parent: P,
            child: C,
        ) -> Result<$crate::EdgeId, $crate::TskitError>
        where
            L: Into<$crate::Position>,
            R: Into<$crate::Position>,
            P: Into<$crate::NodeId>,
            C: Into<$crate::NodeId>,
        {
            edge_table_add_row_details!(left,
                                        right,
                                        parent,
                                        child,
                                        std::ptr::null(),
                                        0,
                                        $table)
        }
    };
}

macro_rules! edge_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<L,R,P,C,M>(
            &mut $self,
            left: L,
            right: R,
            parent: P,
            child: C,
            metadata: &M,
        ) -> Result<$crate::EdgeId, $crate::TskitError>
        where
            L: Into<$crate::Position>,
            R: Into<$crate::Position>,
            P: Into<$crate::NodeId>,
            C: Into<$crate::NodeId>,
            M: $crate::metadata::EdgeMetadata
        {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            edge_table_add_row_details!(left,
                                        right,
                                        parent,
                                        child,
                                        md.as_ptr(),
                                        md.len().into(),
                                        $table)
        }
    };
}

macro_rules! population_table_add_row_details {
    ($metadata: expr, $metadata_len: expr, $table: expr) => {{
        let rv = unsafe {
            $crate::bindings::tsk_population_table_add_row(&mut $table, $metadata, $metadata_len)
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! population_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name(&mut $self) -> Result<$crate::PopulationId, $crate::TskitError> {
            population_table_add_row_details!(std::ptr::null(), 0, $table)
        }
    };
}

macro_rules! population_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<M>(&mut $self, metadata: &M) -> Result<$crate::PopulationId, $crate::TskitError>
        where M: $crate::metadata::PopulationMetadata {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            population_table_add_row_details!(md.as_ptr(), md.len().into(), $table)
        }
    };
}

macro_rules! individual_table_add_row_details {
    ($flags: ident,
     $location: ident,
     $parents: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let rv = unsafe {
            $crate::bindings::tsk_individual_table_add_row(
                &mut $table,
                $flags.into().bits(),
                $location.get_slice().as_ptr().cast::<f64>(),
                $location.get_slice().len() as $crate::bindings::tsk_size_t,
                $parents
                    .get_slice()
                    .as_ptr()
                    .cast::<$crate::bindings::tsk_id_t>(),
                $parents.get_slice().len() as $crate::bindings::tsk_size_t,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! individual_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<F,L,P>(&mut $self,
        flags: F,
        location: L,
        parents: P,
        ) -> Result<$crate::IndividualId, $crate::TskitError>
        where
            F: Into<$crate::IndividualFlags>,
            L: $crate::IndividualLocation,
            P: $crate::IndividualParents,
        {
            individual_table_add_row_details!(flags,
                                              location,
                                              parents,
                                              std::ptr::null(),
                                              0,
                                              $table)
        }
    };
}

macro_rules! individual_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<F,L,P,M>(&mut $self,
                        flags: F,
                        location: L,
                        parents: P,
                        metadata: &M,
        ) -> Result<$crate::IndividualId, $crate::TskitError>
            where
                F: Into<$crate::IndividualFlags>,
                L: $crate::IndividualLocation,
                P: $crate::IndividualParents,
                M: $crate::metadata::IndividualMetadata
            {
                let md = $crate::metadata::EncodedMetadata::new(metadata)?;
                individual_table_add_row_details!(flags,
                                                  location,
                                                  parents,
                                                  md.as_ptr(),
                                                  md.len().into(),
                                                  $table)
            }
    };
}

macro_rules! mutation_table_add_row_details {
    ($site: ident, $node: ident, $parent: ident,
     $time: ident, $derived_state: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let dstate = process_state_input!($derived_state);
        let rv = unsafe {
            $crate::bindings::tsk_mutation_table_add_row(
                &mut $table,
                $site.into().0,
                $node.into().0,
                $parent.into().0,
                $time.into().0,
                dstate.0,
                dstate.1,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! mutation_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<S,N,P,T>(&mut $self,
                     site: S,
                     node: N,
                     parent: P,
                     time: T,
                     derived_state: Option<&[u8]>) -> Result<$crate::MutationId, $crate::TskitError>
        where
             S: Into<$crate::SiteId>,
             N: Into<$crate::NodeId>,
             P: Into<$crate::MutationId>,
             T: Into<$crate::Time>,
        {
            mutation_table_add_row_details!(site,
                                            node,
                                            parent,
                                            time,
                                            derived_state,
                                            std::ptr::null(),
                                            0,
                                            $table)
        }
    };
}

macro_rules! mutation_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<S,N,P,T,M>(&mut $self,
                                site: S,
                                node: N,
                                parent: P,
                                time: T,
                                derived_state: Option<&[u8]>,
                                metadata: &M) -> Result<$crate::MutationId, $crate::TskitError>
            where
                S: Into<$crate::SiteId>,
                N: Into<$crate::NodeId>,
                P: Into<$crate::MutationId>,
                T: Into<$crate::Time>,
                M: $crate::metadata::MutationMetadata
        {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            mutation_table_add_row_details!(site,
                                            node,
                                            parent,
                                            time,
                                            derived_state,
                                            md.as_ptr(),
                                            md.len().into(),
                                            $table)
        }
    };
}

macro_rules! site_table_add_row_details {
    ($position: ident,
     $ancestral_state: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let astate = process_state_input!($ancestral_state);
        let rv = unsafe {
            $crate::bindings::tsk_site_table_add_row(
                &mut $table,
                $position.into().0,
                astate.0,
                astate.1,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! site_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<P>(&mut $self,
                     position: P,
                     ancestral_state: Option<&[u8]>) -> Result<$crate::SiteId, $crate::TskitError>
        where
            P: Into<$crate::Position>,
        {
            site_table_add_row_details!(position, ancestral_state,
                                        std::ptr::null(), 0, $table)
        }
    };
}

macro_rules! site_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<P, M>(&mut $self,
                        position: P,
                        ancestral_state: Option<&[u8]>,
                        metadata: &M) -> Result<$crate::SiteId, $crate::TskitError>
        where
            P: Into<$crate::Position>,
            M: $crate::metadata::SiteMetadata
        {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            site_table_add_row_details!(position, ancestral_state,
                                        md.as_ptr(),
                                        md.len().into(),
                                        $table)
        }
    };
}

macro_rules! migration_table_add_row_details {
    ($span: ident,
     $node: ident,
     $source_dest: ident,
     $time: ident,
     $metadata: expr,
     $metadata_len: expr,
     $table: expr) => {{
        let rv = unsafe {
            $crate::bindings::tsk_migration_table_add_row(
                &mut $table,
                $span.0.into().0,
                $span.1.into().0,
                $node.into().0,
                $source_dest.0.into().0,
                $source_dest.1.into().0,
                $time.into().0,
                $metadata,
                $metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }};
}

macro_rules! migration_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<LEFT,RIGHT,N,SOURCE,DEST,T>(&mut $self,
                     span: (LEFT, RIGHT),
                     node: N,
                     source_dest: (SOURCE, DEST),
                     time: T)
        -> Result<$crate::MigrationId, $crate::TskitError>
        where
            LEFT: Into<$crate::Position>,
            RIGHT: Into<$crate::Position>,
            N: Into<$crate::NodeId>,
            SOURCE: Into<$crate::PopulationId>,
            DEST: Into<$crate::PopulationId>,
            T: Into<$crate::Time>,
        {
            migration_table_add_row_details!(span, node, source_dest, time, std::ptr::null(), 0, $table)
        }
    };
}

macro_rules! migration_table_add_row_with_metadata {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name<LEFT, RIGHT,N,SOURCE,DEST,T,M>(&mut $self,
                        span: (LEFT, RIGHT),
                        node: N,
                        source_dest: (SOURCE, DEST),
                        time: T,
                        metadata: &M)
        -> Result<$crate::MigrationId, $crate::TskitError>
        where
            LEFT: Into<$crate::Position>,
            RIGHT: Into<$crate::Position>,
            N: Into<$crate::NodeId>,
            SOURCE: Into<$crate::PopulationId>,
            DEST: Into<$crate::PopulationId>,
            T: Into<$crate::Time>,
            M: $crate::metadata::MigrationMetadata
        {
            let md = $crate::metadata::EncodedMetadata::new(metadata)?;
            migration_table_add_row_details!(span, node, source_dest, time,
                                             md.as_ptr(), md.len().into(), $table)
        }
    };
}

#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
macro_rules! provenance_table_add_row {
    ($(#[$attr:meta])* => $name: ident, $self: ident, $table: expr) => {
        $(#[$attr])*
        pub fn $name(&mut $self, record: &str) -> Result<$crate::ProvenanceId, $crate::TskitError> {
            if record.is_empty() {
                return Err($crate::TskitError::ValueError{got: "empty string".to_string(), expected: "provenance record".to_string()})
            }
            let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
            let rv = unsafe {
                $crate::bindings::tsk_provenance_table_add_row(
                    &mut $table,
                    timestamp.as_ptr() as *mut i8,
                    timestamp.len() as tsk_size_t,
                    record.as_ptr() as *mut i8,
                    record.len() as tsk_size_t,
                )
            };
            handle_tsk_return_value!(rv, rv.into())
        }
    };
}

macro_rules! build_owned_table_type {
    ($(#[$attr:meta])* => $name: ident,
    $deref_type: ident,
    $tskname: ident,
    $tskinit: ident,
    $tskfree: ident,
    $tskclear: expr) => {
        $(#[$attr])*
        pub struct $name {
            table: mbox::MBox<$crate::bindings::$tskname>,
        }

        build_owned_tables!(
            $name,
            $deref_type,
            $crate::bindings::$tskname,
            $tskinit,
            $tskfree,
            $tskclear
        );
    };
}

macro_rules! raw_metadata_getter_for_tables {
    ($idtype: ident) => {
        fn raw_metadata(&self, row: $idtype) -> Option<&[u8]> {
            $crate::metadata::char_column_to_slice(
                self,
                self.as_ref().metadata,
                self.as_ref().metadata_offset,
                row.0,
                self.num_rows().into(),
                self.as_ref().metadata_length,
            )
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
            // Caveat: num_rows is u64 but we need usize
            // The conversion is fallible but unlikely.
            let num_rows =
                usize::try_from(self.num_rows()).expect("conversion of num_rows to usize failed");
            let ptr = self.as_ref().$column as *const $cast;
            // SAFETY: tables are initialzed, num rows comes
            // from the C back end.
            unsafe { std::slice::from_raw_parts(ptr, num_rows) }
        }
    };
}

macro_rules! build_table_column_slice_mut_getter {
    ($(#[$attr:meta])* => $column: ident, $name: ident, $cast: ty) => {
        $(#[$attr])*
        pub fn $name(&mut self) -> &mut [$cast] {
            // Caveat: num_rows is u64 but we need usize
            // The conversion is fallible but unlikely.
            let num_rows =
                usize::try_from(self.num_rows()).expect("conversion of num_rows to usize failed");
            let ptr = self.as_ref().$column as *mut $cast;
            // SAFETY: tables are initialzed, num rows comes
            // from the C back end.
            unsafe { std::slice::from_raw_parts_mut(ptr, num_rows) }
        }
    };
}

macro_rules! delegate_table_view_api {
    () => {
        delegate::delegate! {
            to self.views {
                /// Get reference to the [``EdgeTable``](crate::EdgeTable).
                pub fn edges(&self) -> &crate::EdgeTable;
                /// Get reference to the [``IndividualTable``](crate::IndividualTable).
                pub fn individuals(&self) -> &crate::IndividualTable;
                /// Get reference to the [``MigrationTable``](crate::MigrationTable).
                pub fn migrations(&self) -> &crate::MigrationTable;
                /// Get reference to the [``MutationTable``](crate::MutationTable).
                pub fn mutations(&self) -> &crate::MutationTable;
                /// Get reference to the [``NodeTable``](crate::NodeTable).
                pub fn nodes(&self) -> &crate::NodeTable;
                /// Get reference to the [``PopulationTable``](crate::PopulationTable).
                pub fn populations(&self) -> &crate::PopulationTable;
                /// Get reference to the [``SiteTable``](crate::SiteTable).
                pub fn sites(&self) -> &crate::SiteTable;

                #[cfg(feature = "provenance")]
                #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
                /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
                pub fn provenances(&self) -> &crate::provenance::ProvenanceTable ;

                /// Return an iterator over the individuals.
                pub fn individuals_iter(&self) -> impl Iterator<Item = crate::IndividualTableRow> + '_;
                /// Return an iterator over the nodes.
                pub fn nodes_iter(&self) -> impl Iterator<Item = crate::NodeTableRow> + '_;
                /// Return an iterator over the edges.
                pub fn edges_iter(&self) -> impl Iterator<Item = crate::EdgeTableRow> + '_;
                /// Return an iterator over the migrations.
                pub fn migrations_iter(&self) -> impl Iterator<Item = crate::MigrationTableRow> + '_;
                /// Return an iterator over the mutations.
                pub fn mutations_iter(&self) -> impl Iterator<Item = crate::MutationTableRow> + '_;
                /// Return an iterator over the populations.
                pub fn populations_iter(&self) -> impl Iterator<Item = crate::PopulationTableRow> + '_;
                /// Return an iterator over the sites.
                pub fn sites_iter(&self) -> impl Iterator<Item = crate::SiteTableRow> + '_;

                #[cfg(feature = "provenance")]
                #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
                /// Return an iterator over provenances
                pub fn provenances_iter(&self,) -> impl Iterator<Item = crate::provenance::ProvenanceTableRow> + '_;

                /// Obtain a vector containing the indexes ("ids")
                /// of all nodes for which [`crate::TSK_NODE_IS_SAMPLE`]
                /// is `true`.
                ///
                /// The provided implementation dispatches to
                /// [`crate::NodeTable::samples_as_vector`].
                pub fn samples_as_vector(&self) -> Vec<crate::NodeId>;

                /// Obtain a vector containing the indexes ("ids") of all nodes
                /// satisfying a certain criterion.
                ///
                /// The provided implementation dispatches to
                /// [`crate::NodeTable::create_node_id_vector`].
                ///
                /// # Parameters
                ///
                /// * `f`: a function.  The function is passed the current table
                ///    collection and each [`crate::node_table::NodeTableRow`].
                ///    If `f` returns `true`, the index of that row is included
                ///    in the return value.
                ///
                /// # Examples
                ///
                /// Get all nodes with time > 0.0:
                ///
                /// ```
                /// use tskit::bindings::tsk_id_t;
                ///
                /// let mut tables = tskit::TableCollection::new(100.).unwrap();
                /// tables
                ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 0.0, tskit::PopulationId::NULL,
                ///     tskit::IndividualId::NULL)
                ///     .unwrap();
                /// tables
                ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 1.0, tskit::PopulationId::NULL,
                ///     tskit::IndividualId::NULL)
                ///     .unwrap();
                /// let samples = tables.create_node_id_vector(
                ///     |row: &tskit::NodeTableRow| row.time > 0.,
                /// );
                /// assert_eq!(samples[0], 1);
                ///
                /// // Get all nodes that have a mutation:
                ///
                /// // fn node_has_mutation(
                /// //     // dyn trait here means this
                /// //     // will work with TreeSequence, too.
                /// //     tables_type: &dyn std::ops::Deref<Target=tskit::table_views::TableViews>,
                /// //     row: &tskit::NodeTableRow,
                /// // ) -> bool {
                /// //     for mrow in tables_type.mutations_iter() {
                /// //         if mrow.node == row.id {
                /// //             return true;
                /// //         }
                /// //     }
                /// //     false
                /// // }
                ///
                /// // // Get all nodes that have a mutation:
                ///
                /// // tables.add_mutation(0, 0, tskit::MutationId::NULL, 0.0, None).unwrap();
                /// // let samples_with_mut = tables.create_node_id_vector(
                /// //     |row: &tskit::NodeTableRow| node_has_mutation(&tables, row));
                /// // assert_eq!(samples_with_mut[0], 0);
                /// ```
                pub fn create_node_id_vector(&self, f: impl FnMut(&crate::NodeTableRow) -> bool) -> Vec<crate::NodeId>;
            }
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
