use crate::sys::bindings;
use crate::TskitError;

use bindings::tsk_id_t;
use bindings::tsk_size_t;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct NodeId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct IndividualId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct PopulationId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct SiteId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct MutationId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct MigrationId(tsk_id_t);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct EdgeId(tsk_id_t);

impl_id_traits!(NodeId);
impl_id_traits!(IndividualId);
impl_id_traits!(PopulationId);
impl_id_traits!(SiteId);
impl_id_traits!(MutationId);
impl_id_traits!(MigrationId);
impl_id_traits!(EdgeId);

impl_size_type_comparisons_for_row_ids!(NodeId);
impl_size_type_comparisons_for_row_ids!(EdgeId);
impl_size_type_comparisons_for_row_ids!(SiteId);
impl_size_type_comparisons_for_row_ids!(MutationId);
impl_size_type_comparisons_for_row_ids!(PopulationId);
impl_size_type_comparisons_for_row_ids!(MigrationId);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct SizeType(tsk_size_t);

impl SizeType {
    /// Convenience function to convert to usize.
    ///
    /// Works via [`TryFrom`].
    ///
    /// # Returns
    ///
    /// * `None` if the underlying value is negative.
    /// * `Some(usize)` otherwise.
    pub fn to_usize(&self) -> Option<usize> {
        (*self).try_into().ok()
    }

    /// Convenience function to convert to usize.
    /// Implemented via `as`.
    /// Negative values with therefore wrap.
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for SizeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<tsk_size_t> for SizeType {
    fn from(value: tsk_size_t) -> Self {
        Self(value)
    }
}

impl From<SizeType> for tsk_size_t {
    fn from(value: SizeType) -> Self {
        value.0
    }
}

// SizeType is u64, so converstion
// can fail on systems with smaller pointer widths.
impl TryFrom<SizeType> for usize {
    type Error = TskitError;

    fn try_from(value: SizeType) -> Result<Self, Self::Error> {
        match usize::try_from(value.0) {
            Ok(x) => Ok(x),
            Err(_) => Err(TskitError::RangeError(format!(
                "could not convert {} to usize",
                value
            ))),
        }
    }
}

impl TryFrom<usize> for SizeType {
    type Error = TskitError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match tsk_size_t::try_from(value) {
            Ok(x) => Ok(Self(x)),
            Err(_) => Err(TskitError::RangeError(format!(
                "could not convert usize {} to SizeType",
                value
            ))),
        }
    }
}

impl TryFrom<tsk_id_t> for SizeType {
    type Error = crate::TskitError;

    fn try_from(value: tsk_id_t) -> Result<Self, Self::Error> {
        match tsk_size_t::try_from(value) {
            Ok(v) => Ok(Self(v)),
            Err(_) => Err(crate::TskitError::RangeError(
                stringify!(value.0).to_string(),
            )),
        }
    }
}

impl TryFrom<SizeType> for tsk_id_t {
    type Error = crate::TskitError;

    fn try_from(value: SizeType) -> Result<Self, Self::Error> {
        match tsk_id_t::try_from(value.0) {
            Ok(v) => Ok(v),
            Err(_) => Err(TskitError::RangeError(stringify!(value.0).to_string())),
        }
    }
}

impl PartialEq<SizeType> for tsk_size_t {
    fn eq(&self, other: &SizeType) -> bool {
        *self == other.0
    }
}

impl PartialEq<tsk_size_t> for SizeType {
    fn eq(&self, other: &tsk_size_t) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<tsk_size_t> for SizeType {
    fn partial_cmp(&self, other: &tsk_size_t) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<SizeType> for tsk_size_t {
    fn partial_cmp(&self, other: &SizeType) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Time(f64);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Position(f64);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Location(f64);

impl_f64_newtypes!(Time);
impl_f64_newtypes!(Position);
impl_f64_newtypes!(Location);

// It is natural to be able to * and / times and positions
impl_time_position_arithmetic!(Time, Position);
impl_time_position_arithmetic!(Position, Time);

#[cfg(feature = "provenance")]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct ProvenanceId(tsk_id_t);

#[cfg(feature = "provenance")]
impl_id_traits!(ProvenanceId);

#[test]
fn test_f64_newtype_Display() {
    let x = Position::from(1.0);
    let mut output = String::new();
    std::fmt::write(&mut output, format_args!("{}", x))
        .expect("Error occurred while trying to write in String");
    assert_eq!(output, "1".to_string());
    let x = Time::from(1.0);
    let mut output = String::new();
    std::fmt::write(&mut output, format_args!("{}", x))
        .expect("Error occurred while trying to write in String");
    assert_eq!(output, "1".to_string());
    let x = Location::from(1.0);
    let mut output = String::new();
    std::fmt::write(&mut output, format_args!("{}", x))
        .expect("Error occurred while trying to write in String");
    assert_eq!(output, "1".to_string());
}

#[test]
fn test_usize_to_size_type() {
    let x = usize::MAX;
    let s = SizeType::try_from(x).ok();

    #[cfg(target_pointer_width = "64")]
    assert_eq!(s, Some(bindings::tsk_size_t::MAX.into()));

    #[cfg(target_pointer_width = "32")]
    assert_eq!(s, Some((usize::MAX as bindings::tsk_size_t).into()));

    let x = usize::MIN;
    let s = SizeType::try_from(x).ok();
    assert_eq!(s, Some(0.into()));
}

#[test]
fn test_from_reference() {
    let x = 2;
    let y = NodeId::from(&x);
    assert_eq!(y, 2);
    assert_eq!(2, tsk_id_t::from(&y));
}

#[test]
fn test_try_from_reference() {
    let y = NodeId::from(2);
    assert_eq!(2, usize::try_from(&y).unwrap());
}
