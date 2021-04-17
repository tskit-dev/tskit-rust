use crate::bindings as ll_bindings;
use crate::metadata;
use crate::{tsk_id_t, TskitError};

/// Row of a [`MigrationTable`]
pub struct MigrationTableRow {
    pub id: tsk_id_t,
    pub left: f64,
    pub right: f64,
    pub node: tsk_id_t,
    pub source: tsk_id_t,
    pub dest: tsk_id_t,
    pub time: f64,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for MigrationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.node == other.node
            && self.source == other.source
            && self.dest == other.dest
            && crate::util::f64_partial_cmp_equal(&self.left, &other.left)
            && crate::util::f64_partial_cmp_equal(&self.right, &other.right)
            && crate::util::f64_partial_cmp_equal(&self.time, &other.time)
            && self.metadata == other.metadata
    }
}

fn make_migration_table_row(
    table: &MigrationTable,
    pos: tsk_id_t,
    decode_metadata: bool,
) -> Option<MigrationTableRow> {
    if pos < table.num_rows() as tsk_id_t {
        Some(MigrationTableRow {
            id: pos,
            left: table.left(pos).unwrap(),
            right: table.right(pos).unwrap(),
            node: table.node(pos).unwrap(),
            source: table.source(pos).unwrap(),
            dest: table.dest(pos).unwrap(),
            time: table.time(pos).unwrap(),
            metadata: match decode_metadata {
                true => match metadata_to_vector!(table, pos).unwrap() {
                    Some(x) => Some(x),
                    None => None,
                },
                false => None,
            },
        })
    } else {
        None
    }
}

pub type MigrationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MigrationTable<'a>>;
pub type MigrationTableIterator<'a> = crate::table_iterator::TableIterator<MigrationTable<'a>>;

impl<'a> Iterator for MigrationTableRefIterator<'a> {
    type Item = MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(self.table, self.pos, self.decode_metadata);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for MigrationTableIterator<'a> {
    type Item = crate::migration_table::MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(&self.table, self.pos, self.decode_metadata);
        self.pos += 1;
        rv
    }
}

/// An immutable view of a migration table.
///
/// These are not created directly.
/// Instead, use [`TableCollection::migrations`](crate::TableCollection::migrations)
/// to get a reference to an existing node table;
pub struct MigrationTable<'a> {
    table_: &'a ll_bindings::tsk_migration_table_t,
}

impl<'a> MigrationTable<'a> {
    pub(crate) fn new_from_table(migrations: &'a ll_bindings::tsk_migration_table_t) -> Self {
        MigrationTable { table_: migrations }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> ll_bindings::tsk_size_t {
        self.table_.num_rows
    }

    /// Return the left coordinate for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn left(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.left)
    }

    /// Return the right coordinate for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn right(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.right)
    }

    /// Return the node for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn node(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.source)
    }

    /// Return the source population for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn source(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.node)
    }

    /// Return the destination population for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn dest(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.dest)
    }

    /// Return the time of the migration event for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn time(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.time)
    }

    /// Return the metadata for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: tsk_id_t,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MigrationTableRow`].
    ///
    /// # Parameters
    ///
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`MigrationTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    pub fn iter(&self, decode_metadata: bool) -> MigrationTableRefIterator {
        crate::table_iterator::make_table_iterator::<&MigrationTable<'a>>(&self, decode_metadata)
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`MigrationTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn row(&self, r: tsk_id_t, decode_metadata: bool) -> Result<MigrationTableRow, TskitError> {
        table_row_access!(r, decode_metadata, self, make_migration_table_row)
    }
}
