use crate::metadata;
use crate::metadata::SiteMetadata;
use crate::sys;
use crate::sys::bindings as ll_bindings;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;

/// A site table.
///
/// # Examples
///
/// ```
/// use tskit::SiteTable;
///
/// let mut sites = SiteTable::default();
/// let rowid = sites.add_row(1., None).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(sites.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::SiteTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::SiteMetadata)]
/// #[serializer("serde_json")]
/// struct SiteMetadata {
///     value: i32,
/// }
///
/// let metadata = SiteMetadata{value: 42};
///
/// let mut sites = SiteTable::default();
///
/// let rowid = sites.add_row_with_metadata(0., None, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match sites.metadata::<SiteMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
/// # }
/// ```
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct SiteTable {
    table_: sys::SiteTable,
}

impl SiteTable {
    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        sites: *mut ll_bindings::tsk_site_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(sites).unwrap();
        let table_ = unsafe { sys::SiteTable::new_borrowed(ptr) };
        Ok(SiteTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_site_table_t {
        self.table_.as_ref()
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    /// Return the ``position`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `row` is valid.
    /// * `None` otherwise.
    pub fn position<S: Into<SiteId> + Copy>(&self, row: S) -> Option<Position> {
        self.table_.position(row.into())
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(ancestral state)` if `row` is valid.
    /// * `None` otherwise.
    pub fn ancestral_state<S: Into<SiteId>>(&self, row: S) -> Option<&[u8]> {
        self.table_.ancestral_state(row.into())
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
    pub fn metadata<T: metadata::SiteMetadata>(
        &self,
        row: impl Into<SiteId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(TskitError::from))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`crate::Site`].
    pub fn iter(&self) -> impl Iterator<Item = crate::Site<'_, crate::sys::SiteTable>> {
        self.table_.iter()
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<S: Into<SiteId> + Copy>(
        &self,
        r: S,
    ) -> Option<super::Site<'_, crate::sys::SiteTable>> {
        self.table_.row(r.into())
    }

    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice, Position);
    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice_raw, f64);

    pub fn position_column(&self) -> impl crate::TableColumn<SiteId, Position> + '_ {
        crate::table_column::OpaqueTableColumn(self.position_slice())
    }

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }

    pub fn add_row<P: Into<Position>>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
    ) -> Result<SiteId, TskitError> {
        let rv = self
            .table_
            .add_row(position.into().into(), ancestral_state)?;
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row_with_metadata<P: Into<Position>, M: SiteMetadata>(
        &mut self,
        position: P,
        ancestral_state: Option<&[u8]>,
        metadata: &M,
    ) -> Result<SiteId, TskitError> {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let rv = self.table_.add_row_with_metadata(
            position.into().into(),
            ancestral_state,
            md.as_slice(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }
}
