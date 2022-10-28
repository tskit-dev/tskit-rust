use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;
use ll_bindings::{tsk_site_table_free, tsk_site_table_init};

/// Row of a [`SiteTable`]
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
    let table_ref = table.table_;
    Some(SiteTableRow {
        id: pos.into(),
        position: table.position(pos).ok()?,
        ancestral_state: table.ancestral_state(pos).ok()?.map(|s| s.to_vec()),
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type SiteTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a SiteTable<'a>>;
pub(crate) type SiteTableIterator<'a> = crate::table_iterator::TableIterator<SiteTable<'a>>;

impl<'a> Iterator for SiteTableRefIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for SiteTableIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::sites`](crate::TableAccess::sites)
/// to get a reference to an existing site table;
pub struct SiteTable<'a> {
    table_: &'a ll_bindings::tsk_site_table_t,
}

impl<'a> SiteTable<'a> {
    pub(crate) fn new_from_table(sites: &'a ll_bindings::tsk_site_table_t) -> Self {
        SiteTable { table_: sites }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
    }

    /// Return the ``position`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn position<S: Into<SiteId> + Copy>(&'a self, row: S) -> Result<Position, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.position) {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(e),
        }
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no ancestral state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn ancestral_state<S: Into<SiteId>>(&'a self, row: S) -> Result<Option<&[u8]>, TskitError> {
        crate::metadata::char_column_to_slice(
            self,
            self.table_.ancestral_state,
            self.table_.ancestral_state_offset,
            row.into().0,
            self.table_.num_rows,
            self.table_.ancestral_state_length,
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: SiteId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`SiteTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = SiteTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&SiteTable<'a>>(self)
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn row<S: Into<SiteId> + Copy>(&self, r: S) -> Result<SiteTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(r.into().0, self, make_site_table_row)
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
    /// if let Some(decoded) = sites.metadata::<SiteMetadata>(rowid).unwrap() {
    ///     assert_eq!(decoded.value, 42);
    /// } else {
    ///     panic!("hmm...we expected some metadata!");
    /// }
    ///
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
