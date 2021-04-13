use crate::bindings as ll_bindings;
use crate::metadata;
use crate::TskitError;
use crate::{tsk_id_t, tsk_size_t};

/// Row of a [`SiteTable`]
pub struct SiteTableRow {
    pub position: f64,
    pub ancestral_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

pub type SiteTableIterator<'a> = crate::table_iterator::TableIterator<'a, SiteTable<'a>>;

impl<'a> Iterator for SiteTableIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.table.num_rows() as tsk_id_t {
            let rv = SiteTableRow {
                position: self.table.position(self.pos).unwrap(),
                ancestral_state: self.table.ancestral_state(self.pos).unwrap(),
                metadata: match self.decode_metadata {
                    true => match metadata_to_vector!(self.table, self.pos).unwrap() {
                        Some(x) => Some(x),
                        None => None,
                    },
                    false => None,
                },
            };
            self.pos += 1;
            Some(rv)
        } else {
            None
        }
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
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
    }

    /// Return the ``position`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn position(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.position)
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
    pub fn ancestral_state(&'a self, row: tsk_id_t) -> Result<Option<Vec<u8>>, TskitError> {
        crate::metadata::char_column_to_vector(
            self.table_.ancestral_state,
            self.table_.ancestral_state_offset,
            row,
            self.table_.num_rows,
            self.table_.ancestral_state_length,
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: tsk_id_t,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`SiteTableRow`].
    ///
    /// # Parameters
    ///
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`SiteTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    pub fn iter(&self, decode_metadata: bool) -> SiteTableIterator {
        crate::table_iterator::make_table_iterator(self, decode_metadata)
    }
}
