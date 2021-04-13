use crate::bindings as ll_bindings;
use crate::metadata;
use crate::TskitError;
use crate::{tsk_id_t, tsk_size_t};

/// Row of a [`PopulationTable`]
pub struct PopulationTableRow {
    pub metadata: Option<Vec<u8>>,
}

pub type PopulationTableIterator<'a> =
    crate::table_iterator::TableIterator<'a, PopulationTable<'a>>;

impl<'a> Iterator for PopulationTableIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.table.num_rows() as tsk_id_t {
            let rv = PopulationTableRow {
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
/// Instead, use [`TableCollection::populations`](crate::TableCollection::populations)
/// to get a reference to an existing population table;
pub struct PopulationTable<'a> {
    table_: &'a ll_bindings::tsk_population_table_t,
}

impl<'a> PopulationTable<'a> {
    pub(crate) fn new_from_table(mutations: &'a ll_bindings::tsk_population_table_t) -> Self {
        PopulationTable { table_: mutations }
    }

    /// Return the number of rows.
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: tsk_id_t,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`PopulationTableRow`].
    ///
    /// # Parameters
    ///
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`PopulationTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    pub fn iter(&self, decode_metadata: bool) -> PopulationTableIterator {
        crate::table_iterator::make_table_iterator(self, decode_metadata)
    }
}
