use std::ptr::NonNull;

use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;
use ll_bindings::{tsk_site_table_free, tsk_site_table_init};

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
    let table_ref = table.as_ref();
    let ancestral_state = table.ancestral_state(pos).map(|s| s.to_vec());
    Some(SiteTableRow {
        id: pos.into(),
        position: table.position(pos)?,
        ancestral_state,
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
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

/// An immutable view of site table.
///
/// These are not created directly but are accessed
/// by types implementing [`std::ops::Deref`] to
/// [`crate::table_views::TableViews`]
#[derive(Debug)]
pub struct SiteTable {
    table_: NonNull<ll_bindings::tsk_site_table_t>,
}

impl SiteTable {
    pub(crate) fn new_from_table(
        sites: *mut ll_bindings::tsk_site_table_t,
    ) -> Result<Self, TskitError> {
        let n = NonNull::new(sites).ok_or_else(|| {
            TskitError::LibraryError("null pointer to tsk_site_table_t".to_string())
        })?;
        Ok(SiteTable { table_: n })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_site_table_t {
        // SAFETY: NonNull
        unsafe { self.table_.as_ref() }
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
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ref(),
            position,
            Position
        )
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(ancestral state)` if `row` is valid.
    /// * `None` otherwise.
    pub fn ancestral_state<S: Into<SiteId>>(&self, row: S) -> Option<&[u8]> {
        crate::metadata::char_column_to_slice(
            self,
            self.as_ref().ancestral_state,
            self.as_ref().ancestral_state_offset,
            row.into().0,
            self.as_ref().num_rows,
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
        let table_ref = self.as_ref();
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
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
        let ri = r.into().0;
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
}

build_owned_table_type!(
    /// A standalone site table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwnedSiteTable;
    ///
    /// let mut sites = OwnedSiteTable::default();
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
    /// use tskit::OwnedSiteTable;
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
    /// let mut sites = OwnedSiteTable::default();
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
    => OwnedSiteTable,
    SiteTable,
    tsk_site_table_t,
    tsk_site_table_init,
    tsk_site_table_free,
    ll_bindings::tsk_site_table_clear
);

impl OwnedSiteTable {
    site_table_add_row!(=> add_row, self, *self.table);
    site_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
