use crate::bindings as ll_bindings;
use crate::metadata;
use crate::Position;
use crate::{tsk_id_t, TskitError};
use crate::{EdgeId, NodeId};

/// Row of an [`EdgeTable`]
pub struct EdgeTableRow {
    pub id: EdgeId,
    pub left: Position,
    pub right: Position,
    pub parent: NodeId,
    pub child: NodeId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for EdgeTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && self.metadata == other.metadata
    }
}

fn make_edge_table_row(table: &EdgeTable, pos: tsk_id_t) -> Option<EdgeTableRow> {
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let table_ref = &unsafe { *table.table_ };
        let rv = EdgeTableRow {
            id: pos.into(),
            left: table.left(pos).unwrap(),
            right: table.right(pos).unwrap(),
            parent: table.parent(pos).unwrap(),
            child: table.child(pos).unwrap(),
            metadata: table_row_decode_metadata!(table_ref, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub(crate) type EdgeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a EdgeTable>;
pub(crate) type EdgeTableIterator<'a> = crate::table_iterator::TableIterator<EdgeTable>;

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
/// Instead, use [`TableAccess::edges`](crate::TableAccess::edges)
/// to get a reference to an existing edge table;
pub struct EdgeTable {
    table_: *const ll_bindings::tsk_edge_table_t,
}

impl EdgeTable {
    pub(crate) fn as_ll_ref(&self) -> &ll_bindings::tsk_edge_table_t {
        // SAFETY: cannot be constructed with null pointer
        unsafe { &(*self.table_) }
    }

    pub(crate) fn new_from_table(edges: &ll_bindings::tsk_edge_table_t) -> Self {
        EdgeTable { table_: edges }
    }

    pub(crate) fn new_null() -> Self {
        Self {
            table_: std::ptr::null(),
        }
    }

    pub(crate) fn set_ptr(&mut self, ptr: *const ll_bindings::tsk_edge_table_t) {
        assert!(!ptr.is_null());
        self.table_ = ptr;
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ll_ref().num_rows.into()
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent<E: Into<EdgeId> + Copy>(&self, row: E) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().parent,
            NodeId
        )
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn child<E: Into<EdgeId> + Copy>(&self, row: E) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().child,
            NodeId
        )
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn left<E: Into<EdgeId> + Copy>(&self, row: E) -> Result<Position, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.as_ll_ref().left) {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(e),
        }
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn right<E: Into<EdgeId> + Copy>(&self, row: E) -> Result<Position, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.as_ll_ref().right) {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(e),
        }
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &self,
        row: EdgeId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = &unsafe { *self.table_ };
        let buffer = metadata_to_vector!(table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = EdgeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&EdgeTable>(self)
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
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_edge_table_row)
    }
}
