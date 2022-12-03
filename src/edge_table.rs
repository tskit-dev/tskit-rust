use std::ptr::NonNull;

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
    let table_ref = table.as_ref();
    Some(EdgeTableRow {
        id: pos.into(),
        left: table.left(pos)?,
        right: table.right(pos)?,
        parent: table.parent(pos)?,
        child: table.child(pos)?,
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type EdgeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a EdgeTable>;
pub(crate) type EdgeTableIterator = crate::table_iterator::TableIterator<EdgeTable>;

impl<'a> Iterator for EdgeTableRefIterator<'a> {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for EdgeTableIterator {
    type Item = EdgeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_edge_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// Row of an [`EdgeTable`]
#[derive(Debug)]
pub struct EdgeTableRowView<'a> {
    table: &'a EdgeTable,
    pub id: EdgeId,
    pub left: Position,
    pub right: Position,
    pub parent: NodeId,
    pub child: NodeId,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> EdgeTableRowView<'a> {
    fn new(table: &'a EdgeTable) -> Self {
        Self {
            table,
            id: (-1).into(),
            left: f64::NAN.into(),
            right: f64::NAN.into(),
            parent: NodeId::NULL,
            child: NodeId::NULL,
            metadata: None,
        }
    }
}

impl<'a> PartialEq for EdgeTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && self.metadata == other.metadata
    }
}

impl<'a> Eq for EdgeTableRowView<'a> {}

impl<'a> PartialEq<EdgeTableRow> for EdgeTableRowView<'a> {
    fn eq(&self, other: &EdgeTableRow) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<EdgeTableRowView<'_>> for EdgeTableRow {
    fn eq(&self, other: &EdgeTableRowView) -> bool {
        self.id == other.id
            && self.parent == other.parent
            && self.child == other.child
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for EdgeTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.left = self.table.left(self.id).unwrap_or_else(|| f64::NAN.into());
        self.right = self.table.right(self.id).unwrap_or_else(|| f64::NAN.into());
        self.parent = self.table.parent(self.id).unwrap_or(NodeId::NULL);
        self.child = self.table.child(self.id).unwrap_or(NodeId::NULL);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// An immutable view of an edge table.
///
/// These are not created directly but are accessed
/// by types implementing [`std::ops::Deref`] to
/// [`crate::table_views::TableViews`]
#[repr(transparent)]
#[derive(Debug)]
pub struct EdgeTable {
    pub(crate) table_: NonNull<ll_bindings::tsk_edge_table_t>,
}

impl EdgeTable {
    pub(crate) fn new_from_table(
        edges: *mut ll_bindings::tsk_edge_table_t,
    ) -> Result<Self, TskitError> {
        let n = NonNull::new(edges).ok_or_else(|| {
            TskitError::LibraryError("null pointer to tsk_edge_table_t".to_string())
        })?;
        Ok(EdgeTable { table_: n })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_edge_table_t {
        // SAFETY: NonNull
        unsafe { self.table_.as_ref() }
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ref().num_rows.into()
    }

    raw_metadata_getter_for_tables!(EdgeId);

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(parent)` if `u` is valid.
    /// * `None` otherwise.
    pub fn parent<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        unsafe_tsk_column_access!(
            row.into(),
            0,
            self.num_rows(),
            self.as_ref(),
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
    pub fn child<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        unsafe_tsk_column_access!(row.into(), 0, self.num_rows(), self.as_ref(), child, NodeId)
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn left<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        unsafe_tsk_column_access!(
            row.into(),
            0,
            self.num_rows(),
            self.as_ref(),
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
    pub fn right<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        unsafe_tsk_column_access_and_map_into!(row.into(), 0, self.num_rows(), self.as_ref(), right)
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is not valid and decoding failed.
    /// * `None` if `row` is not valid.
    ///
    /// # Errors
    ///
    /// * [`TskitError::MetadataError`] if decoding fails.
    ///
    /// # Examples.
    ///
    /// The big-picture semantics are the same for all table types.
    /// See [`crate::IndividualTable::metadata`] for examples.
    pub fn metadata<T: metadata::EdgeMetadata>(
        &self,
        row: EdgeId,
    ) -> Option<Result<T, TskitError>> {
        let table_ref = self.as_ref();
        let buffer = metadata_to_vector!(self, table_ref, row.into())?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`EdgeTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = EdgeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&EdgeTable>(self)
    }

    pub fn lending_iter(&self) -> EdgeTableRowView {
        EdgeTableRowView::new(self)
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
        table_row_access!(r.into().into(), self, make_edge_table_row)
    }

    /// Return a view of row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row_view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<E: Into<EdgeId> + Copy>(&self, r: E) -> Option<EdgeTableRowView> {
        let view = EdgeTableRowView {
            table: self,
            id: r.into(),
            left: self.left(r)?,
            right: self.right(r)?,
            parent: self.parent(r)?,
            child: self.child(r)?,
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
    }

    build_table_column_slice_getter!(
        /// Get the left column as a slice
        => left, left_slice, Position);
    build_table_column_slice_getter!(
        /// Get the left column as a slice of [`f64`] 
        => left, left_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the right column as a slice
        => right, right_slice, Position);
    build_table_column_slice_getter!(
        /// Get the left column as a slice of [`f64`] 
        => right, right_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice
        => parent, parent_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice of [`crate::bindings::tsk_id_t`]
        => parent, parent_slice_raw, ll_bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the child column as a slice
        => child, child_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the child column as a slice of [`crate::bindings::tsk_id_t`]
        => child, child_slice_raw, ll_bindings::tsk_id_t);
}

build_owned_table_type!(
    /// A standalone edge table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwningEdgeTable;
    ///
    /// let mut edges = OwningEdgeTable::default();
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
    /// use tskit::OwningEdgeTable;
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
    /// let mut edges = OwningEdgeTable::default();
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
    => OwningEdgeTable,
    EdgeTable,
    tsk_edge_table_t,
    tsk_edge_table_init,
    tsk_edge_table_free,
    crate::bindings::tsk_edge_table_clear
);

impl OwningEdgeTable {
    edge_table_add_row!(=> add_row, self, *self.table);
    edge_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
