use crate::metadata;
use crate::sys;
use crate::PopulationId;
use crate::SizeType;
use crate::TskitError;
use sys::bindings as ll_bindings;

/// A population table
///
/// # Examples
///
/// ```
/// use tskit::PopulationTable;
///
/// let mut populations = PopulationTable::default();
/// let rowid = populations.add_row().unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(populations.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::PopulationTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::PopulationMetadata)]
/// #[serializer("serde_json")]
/// struct PopulationMetadata {
///     name: String,
/// }
///
/// let metadata = PopulationMetadata{name: "YRB".to_string()};
///
/// let mut populations = PopulationTable::default();
///
/// let rowid = populations.add_row_with_metadata(&metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match populations.metadata::<PopulationMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(&decoded.name, "YRB"),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
/// # }
/// ```
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct PopulationTable {
    table_: sys::PopulationTable,
}

impl PopulationTable {
    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        populations: *mut ll_bindings::tsk_population_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(populations).unwrap();
        let table_ = unsafe { sys::PopulationTable::new_borrowed(ptr) };
        Ok(PopulationTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_population_table_t {
        self.table_.as_ref()
    }

    /// Return the number of rows.
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is valid and decoding failed.
    /// * `None` if `row` is not valid or the row has no metadata.
    ///
    /// # Errors
    ///
    /// * [`TskitError::MetadataError`] if decoding fails.
    ///
    /// # Examples.
    ///
    /// The big-picture semantics are the same for all table types.
    /// See [`crate::IndividualTable::metadata`] for examples.
    pub fn metadata<T: metadata::PopulationMetadata>(
        &self,
        row: impl Into<PopulationId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(TskitError::from))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`crate::Population`].
    pub fn iter(&self) -> impl Iterator<Item = crate::Population<'_, crate::sys::PopulationTable>> {
        self.table_.iter()
    }

    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<P: Into<PopulationId> + Copy>(
        &self,
        r: P,
    ) -> Option<crate::Population<'_, crate::sys::PopulationTable>> {
        self.table_.row(r.into())
    }

    pub fn add_row(&mut self) -> Result<PopulationId, TskitError> {
        Ok(self.table_.add_row()?.into())
    }

    pub fn add_row_with_metadata<M: crate::metadata::PopulationMetadata>(
        &mut self,
        metadata: &M,
    ) -> Result<PopulationId, TskitError> {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        Ok(self.table_.add_row_with_metadata(md.as_slice())?.into())
    }

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }
}
