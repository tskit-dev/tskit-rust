use crate::bindings as ll_bindings;
use crate::metadata;
use crate::{tsk_id_t, tsk_size_t, TskitError};

pub struct EdgeTableRow {
    pub left: f64,
    pub right: f64,
    pub parent: tsk_id_t,
    pub child: tsk_id_t,
    pub metadata: Option<Vec<u8>>,
}

pub type EdgeTableIterator<'a> = crate::table_iterator::TableIterator<'a, EdgeTable<'a>>;

impl<'a> Iterator for EdgeTableIterator<'a> {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.table.num_rows() as tsk_id_t {
            let rv = EdgeTableRow {
                left: self.table.left(self.pos).unwrap(),
                right: self.table.right(self.pos).unwrap(),
                parent: self.table.parent(self.pos).unwrap(),
                child: self.table.child(self.pos).unwrap(),
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

/// An immutable view of an edge table.
///
/// These are not created directly.
/// Instead, use [`TableCollection::edges`](crate::TableCollection::edges)
/// to get a reference to an existing edge table;
pub struct EdgeTable<'a> {
    table_: &'a ll_bindings::tsk_edge_table_t,
}

impl<'a> EdgeTable<'a> {
    pub(crate) fn new_from_table(edges: &'a ll_bindings::tsk_edge_table_t) -> Self {
        EdgeTable { table_: edges }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.parent)
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn child(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.child)
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn left(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.left)
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn right(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.right)
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: tsk_id_t,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    /// # Parameters
    ///
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`EdgeTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    pub fn iter(&self, decode_metadata: bool) -> EdgeTableIterator {
        crate::table_iterator::make_table_iterator(self, decode_metadata)
    }
}
