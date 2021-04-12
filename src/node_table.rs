use crate::bindings as ll_bindings;
use crate::metadata;
use crate::{tsk_flags_t, tsk_id_t, TskitError};

/// Row of a [`NodeTable`]
pub struct NodeTableRow {
    pub time: f64,
    pub flags: tsk_flags_t,
    pub population: tsk_id_t,
    pub individual: tsk_id_t,
    pub metadata: Option<Vec<u8>>,
}

pub type NodeTableIterator<'a> = crate::table_iterator::TableIterator<'a, NodeTable<'a>>;

impl<'a> Iterator for NodeTableIterator<'a> {
    type Item = NodeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.table.num_rows() as tsk_id_t {
            let rv = NodeTableRow {
                time: self.table.time(self.pos).unwrap(),
                flags: self.table.flags(self.pos).unwrap(),
                population: self.table.population(self.pos).unwrap(),
                individual: self.table.individual(self.pos).unwrap(),
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

/// An immtable view of a node table.
///
/// These are not created directly.
/// Instead, use [`TableCollection::nodes`](crate::TableCollection::nodes)
/// to get a reference to an existing node table;
pub struct NodeTable<'a> {
    table_: &'a ll_bindings::tsk_node_table_t,
}

impl<'a> NodeTable<'a> {
    pub(crate) fn new_from_table(nodes: &'a ll_bindings::tsk_node_table_t) -> Self {
        NodeTable { table_: nodes }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> ll_bindings::tsk_size_t {
        self.table_.num_rows
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time(&'a self, row: tsk_id_t) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.time)
    }

    /// Return the ``flags`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn flags(&'a self, row: tsk_id_t) -> Result<tsk_flags_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.flags)
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn population(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.population)
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn deme(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        self.population(row)
    }

    /// Return the ``individual`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn individual(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.individual)
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: tsk_id_t,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`NodeTableRow`].
    ///
    /// # Parameters
    ///
    /// * `decode_metadata`: if `true`, then a *copy* of row metadata
    ///    will be provided in [`NodeTableRow::metadata`].
    ///    The meta data are *not* decoded.
    ///    Rows with no metadata will contain the value `None`.
    ///
    pub fn iter(&self, decode_metadata: bool) -> NodeTableIterator {
        crate::table_iterator::make_table_iterator(self, decode_metadata)
    }
}
