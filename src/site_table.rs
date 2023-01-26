use crate::bindings as ll_bindings;
use crate::metadata;
use crate::sys;
use crate::tsk_id_t;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`SiteTable`]
#[derive(Debug)]
pub struct SiteTableRow {
    pub id: SiteId,
    pub position: Position,
    pub ancestral_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for SiteTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && self.ancestral_state == other.ancestral_state
            && self.metadata == other.metadata
    }
}

fn make_site_table_row(table: &SiteTable, pos: tsk_id_t) -> Option<SiteTableRow> {
    let ancestral_state = table.ancestral_state(pos).map(|s| s.to_vec());
    Some(SiteTableRow {
        id: pos.into(),
        position: table.position(pos)?,
        ancestral_state,
        metadata: table.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type SiteTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a SiteTable>;
pub(crate) type SiteTableIterator = crate::table_iterator::TableIterator<SiteTable>;

impl<'a> Iterator for SiteTableRefIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for SiteTableIterator {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct SiteTableRowView<'a> {
    table: &'a SiteTable,
    pub id: SiteId,
    pub position: Position,
    pub ancestral_state: Option<&'a [u8]>,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> SiteTableRowView<'a> {
    fn new(table: &'a SiteTable) -> Self {
        Self {
            table,
            id: SiteId::NULL,
            position: f64::NAN.into(),
            ancestral_state: None,
            metadata: None,
        }
    }
}

impl<'a> PartialEq for SiteTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && self.ancestral_state == other.ancestral_state
            && self.metadata == other.metadata
    }
}

impl<'a> Eq for SiteTableRowView<'a> {}

impl<'a> PartialEq<SiteTableRow> for SiteTableRowView<'a> {
    fn eq(&self, other: &SiteTableRow) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && optional_container_comparison!(self.ancestral_state, other.ancestral_state)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<SiteTableRowView<'_>> for SiteTableRow {
    fn eq(&self, other: &SiteTableRowView) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && optional_container_comparison!(self.ancestral_state, other.ancestral_state)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for SiteTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.position = self
            .table
            .position(self.id)
            .unwrap_or_else(|| f64::NAN.into());
        self.ancestral_state = self.table.ancestral_state(self.id);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// A site table.
///
/// # Examples
///
/// # Standalone tables
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
#[derive(Debug)]
#[repr(transparent)]
pub struct SiteTable {
    table_: sys::LLSiteTable,
}

impl SiteTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_site_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_site_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        sites: *mut ll_bindings::tsk_site_table_t,
    ) -> Result<Self, TskitError> {
        let table_ = sys::LLSiteTable::new_non_owning(sites)?;
        Ok(SiteTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_site_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::LLSiteTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    raw_metadata_getter_for_tables!(SiteId);

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
        sys::tsk_column_access::<Position, _, _, _>(
            row.into(),
            self.as_ref().position,
            self.num_rows(),
        )
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(ancestral state)` if `row` is valid.
    /// * `None` otherwise.
    pub fn ancestral_state<S: Into<SiteId>>(&self, row: S) -> Option<&[u8]> {
        sys::tsk_ragged_column_access(
            row.into(),
            self.as_ref().ancestral_state,
            self.num_rows(),
            self.as_ref().ancestral_state_offset,
            self.as_ref().ancestral_state_length,
        )
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is not valid and decoding failed.
    /// * `None` if `row` is not valid.
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
        row: SiteId,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(TskitError::from))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`SiteTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = SiteTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&SiteTable>(self)
    }

    pub fn lending_iter(&self) -> SiteTableRowView {
        SiteTableRowView::new(self)
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
    pub fn row<S: Into<SiteId> + Copy>(&self, r: S) -> Option<SiteTableRow> {
        let ri = r.into().into();
        table_row_access!(ri, self, make_site_table_row)
    }

    /// Return a view of row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<S: Into<SiteId> + Copy>(&self, r: S) -> Option<SiteTableRowView> {
        let view = SiteTableRowView {
            table: self,
            id: r.into(),
            position: self.position(r)?,
            ancestral_state: self.ancestral_state(r),
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
    }

    pub fn clear(&mut self) -> Result<(), TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    site_table_add_row!(=> add_row, self, self.as_mut_ptr());
    site_table_add_row_with_metadata!(=> add_row_with_metadata, self, self.as_mut_ptr());

    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice, Position);
    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice_raw, f64);
}

impl Default for SiteTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

pub type OwningSiteTable = SiteTable;
