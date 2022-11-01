use crate::bindings as ll_bindings;
use crate::metadata;
use crate::Position;
use crate::{tsk_id_t, TskitError};
use crate::{EdgeId, NodeId};
use ll_bindings::{tsk_edge_table_free, tsk_edge_table_init};

/// Row of an [`EdgeTable`]
#[derive(Debug)]
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
    let table_ref = table.table_;
    Some(EdgeTableRow {
        id: pos.into(),
        left: table.left(pos)?,
        right: table.right(pos)?,
        parent: table.parent(pos)?,
        child: table.child(pos)?,
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type EdgeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a EdgeTable<'a>>;
pub(crate) type EdgeTableIterator<'a> = crate::table_iterator::TableIterator<EdgeTable<'a>>;

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
#[repr(transparent)]
pub struct EdgeTable<'a> {
    table_: &'a ll_bindings::tsk_edge_table_t,
}

impl<'a> EdgeTable<'a> {
    pub(crate) fn new_from_table(edges: &'a ll_bindings::tsk_edge_table_t) -> Self {
        EdgeTable { table_: edges }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> crate::SizeType {
        self.table_.num_rows.into()
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(parent)` if `u` is valid.
    /// * `None` otherwise.
    pub fn parent<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Option<NodeId> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_,
            parent,
            NodeId
        )
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(child)` if `u` is valid.
    /// * `None` otherwise.
    pub fn child<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Option<NodeId> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_, child, NodeId)
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn left<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Option<Position> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_,
            left,
            Position
        )
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn right<E: Into<EdgeId> + Copy>(&'a self, row: E) -> Option<Position> {
        unsafe_tsk_column_access_and_map_into!(row.into().0, 0, self.num_rows(), self.table_, right)
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: EdgeId,
    ) -> Option<Result<T, TskitError>> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = EdgeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&EdgeTable<'a>>(self)
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<E: Into<EdgeId> + Copy>(&self, r: E) -> Option<EdgeTableRow> {
        table_row_access!(r.into().0, self, make_edge_table_row)
    }
}

build_owned_table_type!(
    /// A standalone edge table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwnedEdgeTable;
    ///
    /// let mut edges = OwnedEdgeTable::default();
    /// let rowid = edges.add_row(1., 2., 0, 1).unwrap();
    /// assert_eq!(rowid, 0);
    /// assert_eq!(edges.num_rows(), 1);
    ///
    /// edges.clear().unwrap();
    /// assert_eq!(edges.num_rows(), 0);
    /// ```
    ///
    /// An example with metadata.
    /// This requires the cargo feature `"derive"` for `tskit`.
    ///
    /// ```
    /// # #[cfg(any(feature="doc", feature="derive"))] {
    /// use tskit::OwnedEdgeTable;
    ///
    /// #[derive(serde::Serialize,
    ///          serde::Deserialize,
    ///          tskit::metadata::EdgeMetadata)]
    /// #[serializer("serde_json")]
    /// struct EdgeMetadata {
    ///     value: i32,
    /// }
    ///
    /// let metadata = EdgeMetadata{value: 42};
    ///
    /// let mut edges = OwnedEdgeTable::default();
    ///
    /// let rowid = edges.add_row_with_metadata(0., 1., 5, 10, &metadata).unwrap();
    /// assert_eq!(rowid, 0);
    ///
    /// match edges.metadata::<EdgeMetadata>(rowid) {
    ///     // rowid is in range, decoding succeeded
    ///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
    ///     // rowid is in range, decoding failed
    ///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
    ///     None => panic!("row id out of range")
    /// }
    /// # }
    /// ```
    => OwnedEdgeTable,
    EdgeTable,
    tsk_edge_table_t,
    tsk_edge_table_init,
    tsk_edge_table_free,
    crate::bindings::tsk_edge_table_clear
);

impl OwnedEdgeTable {
    edge_table_add_row!(=> add_row, self, *self.table);
    edge_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
