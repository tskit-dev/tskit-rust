use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`SiteTable`]
pub struct SiteTableRow {
    pub id: SiteId,
    pub position: f64,
    pub ancestral_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for SiteTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && crate::util::f64_partial_cmp_equal(&self.position, &other.position)
            && self.ancestral_state == other.ancestral_state
            && self.metadata == other.metadata
    }
}

fn make_site_table_row(table: &SiteTable, pos: tsk_id_t) -> Option<SiteTableRow> {
    use std::convert::TryFrom;
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let rv = SiteTableRow {
            id: pos.into(),
            position: table.position(pos).unwrap(),
            ancestral_state: table.ancestral_state(pos).unwrap(),
            metadata: table_row_decode_metadata!(table, pos),
        };
        Some(rv)
    } else {
        None
    }
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
/// Instead, use [`TableCollection::sites`](crate::TableCollection::sites)
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
    pub fn position<S: Into<SiteId> + Copy>(&'a self, row: S) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.position)
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
    pub fn ancestral_state<S: Into<SiteId>>(
        &'a self,
        row: S,
    ) -> Result<Option<Vec<u8>>, TskitError> {
        crate::metadata::char_column_to_vector(
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
        let buffer = metadata_to_vector!(self, row.0)?;
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
