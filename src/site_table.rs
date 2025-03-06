use crate::metadata;
use crate::metadata::SiteMetadata;
use crate::sys;
use crate::sys::bindings as ll_bindings;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;
use ll_bindings::tsk_id_t;

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
        metadata: table.table_.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type SiteTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a SiteTable>;
pub(crate) type SiteTableIterator = crate::table_iterator::TableIterator<SiteTable>;

impl Iterator for SiteTableRefIterator<'_> {
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

impl PartialEq for SiteTableRowView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && self.ancestral_state == other.ancestral_state
            && self.metadata == other.metadata
    }
}

impl Eq for SiteTableRowView<'_> {}

impl PartialEq<SiteTableRow> for SiteTableRowView<'_> {
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

impl streaming_iterator::StreamingIterator for SiteTableRowView<'_> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.position = self
            .table
            .position(self.id)
            .unwrap_or_else(|| f64::NAN.into());
        self.ancestral_state = self.table.ancestral_state(self.id);
        self.metadata = self.table.table_.raw_metadata(self.id);
    }
}

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
    pub(crate) fn new_from_table(
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
        row: impl Into<SiteId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
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
            metadata: self.table_.raw_metadata(r.into()),
        };
        Some(view)
    }

    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice, Position);
    build_table_column_slice_getter!(
        /// Get the position column as a slice
        => position, position_slice_raw, f64);

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
