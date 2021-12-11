use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::PopulationId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`PopulationTable`]
#[derive(Eq)]
pub struct PopulationTableRow {
    pub id: PopulationId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for PopulationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.metadata == other.metadata
    }
}

fn make_population_table_row(table: &PopulationTable, pos: tsk_id_t) -> Option<PopulationTableRow> {
    use std::convert::TryFrom;
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let rv = PopulationTableRow {
            id: pos.into(),
            metadata: table_row_decode_metadata!(table, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub(crate) type PopulationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a PopulationTable<'a>>;
pub(crate) type PopulationTableIterator<'a> =
    crate::table_iterator::TableIterator<PopulationTable<'a>>;

impl<'a> Iterator for PopulationTableRefIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for PopulationTableIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
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
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: PopulationId,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`PopulationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = PopulationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&PopulationTable<'a>>(self)
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
    pub fn row<P: Into<PopulationId> + Copy>(
        &self,
        r: P,
    ) -> Result<PopulationTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_population_table_row)
    }
}
