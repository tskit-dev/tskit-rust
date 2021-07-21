use crate::bindings as ll_bindings;
use crate::metadata;
use crate::{tsk_id_t, tsk_size_t, TskitError};
use crate::{EdgeId, NodeId};

/// Row of an [`EdgeTable`]
pub struct EdgeTableRow {
    pub id: EdgeId,
    pub left: f64,
    pub right: f64,
    pub parent: NodeId,
    pub child: NodeId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for EdgeTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::f64_partial_cmp_equal(&self.left, &other.left)
            && crate::util::f64_partial_cmp_equal(&self.right, &other.right)
            && self.metadata == other.metadata
    }
}

fn make_edge_table_row(table: &EdgeTable, pos: tsk_id_t) -> Option<EdgeTableRow> {
    if pos < table.num_rows() as tsk_id_t {
        let rv = EdgeTableRow {
            id: pos.into(),
            left: table.left(pos).unwrap(),
            right: table.right(pos).unwrap(),
            parent: table.parent(pos).unwrap(),
            child: table.child(pos).unwrap(),
            metadata: table_row_decode_metadata!(table, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub type EdgeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a EdgeTable<'a>>;
pub type EdgeTableIterator<'a> = crate::table_iterator::TableIterator<EdgeTable<'a>>;

impl<'a> Iterator for EdgeTableRefIterator<'a> {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for EdgeTableIterator<'a> {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
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
    pub fn parent<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.parent, NodeId)
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn child<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.child, NodeId)
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn left<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.left)
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn right<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.right)
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: EdgeId,
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    pub fn iter(&self) -> EdgeTableRefIterator {
        crate::table_iterator::make_table_iterator::<&EdgeTable<'a>>(&self)
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
    pub fn row<E: Into<EdgeId> + Copy>(&self, r: E) -> Result<EdgeTableRow, TskitError> {
        table_row_access!(r.into().0, self, make_edge_table_row)
    }
}
