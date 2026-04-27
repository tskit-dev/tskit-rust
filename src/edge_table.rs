use crate::metadata;
use crate::metadata::EdgeMetadata;
use crate::sys;
use crate::Position;
use crate::TskitError;
use crate::{EdgeId, NodeId};
use sys::bindings as ll_bindings;

/// An edge table.
///
/// # Examples
///
/// ```
/// use tskit::EdgeTable;
///
/// let mut edges = EdgeTable::default();
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
/// use tskit::EdgeTable;
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
/// let mut edges = EdgeTable::default();
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
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct EdgeTable {
    table_: sys::EdgeTable,
}

impl EdgeTable {
    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::EdgeTable::new(0)?;
        Ok(Self { table_ })
    }

    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        edges: *mut ll_bindings::tsk_edge_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(edges).unwrap();
        let table_ = unsafe { sys::EdgeTable::new_borrowed(ptr) };
        Ok(EdgeTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_edge_table_t {
        self.table_.as_ref()
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ref().num_rows.into()
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(parent)` if `u` is valid.
    /// * `None` otherwise.
    pub fn parent<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        self.table_.parent(row.into())
    }

    /// Return the ``child`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(child)` if `u` is valid.
    /// * `None` otherwise.
    pub fn child<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<NodeId> {
        self.table_.child(row.into())
    }

    /// Return the ``left`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn left<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        self.table_.left(row.into())
    }

    /// Return the ``right`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `u` is valid.
    /// * `None` otherwise.
    pub fn right<E: Into<EdgeId> + Copy>(&self, row: E) -> Option<Position> {
        self.table_.right(row.into())
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is valid and decoding failed.
    /// * `None` if `row` is not valid or the row has no metadata.
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
        row: impl Into<EdgeId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`crate::Edge`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = crate::Edge<'_, crate::sys::EdgeTable>> {
        self.table_.iter()
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
    pub fn row<E: Into<EdgeId> + Copy>(
        &self,
        r: E,
    ) -> Option<crate::Edge<'_, crate::sys::EdgeTable>> {
        self.table_.row(r.into())
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
        /// Get the parent column as a slice of the underlying integer type
        => parent, parent_slice_raw, ll_bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the child column as a slice
        => child, child_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the child column as a slice of the underlying integer type
        => child, child_slice_raw, ll_bindings::tsk_id_t);

    pub fn parent_column(&self) -> impl crate::TableColumn<EdgeId, NodeId> + '_ {
        crate::table_column::OpaqueTableColumn(self.parent_slice())
    }

    pub fn child_column(&self) -> impl crate::TableColumn<EdgeId, NodeId> + '_ {
        crate::table_column::OpaqueTableColumn(self.child_slice())
    }

    pub fn left_column(&self) -> impl crate::TableColumn<EdgeId, Position> + '_ {
        crate::table_column::OpaqueTableColumn(self.left_slice())
    }

    pub fn right_column(&self) -> impl crate::TableColumn<EdgeId, Position> + '_ {
        crate::table_column::OpaqueTableColumn(self.right_slice())
    }

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }

    /// Add a row without metadata.
    ///
    /// See [crate::TableCollection::add_edge] for examples
    pub fn add_row<L: Into<Position>, R: Into<Position>, P: Into<NodeId>, C: Into<NodeId>>(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
    ) -> Result<EdgeId, TskitError> {
        let rv = self.table_.add_row(
            left.into().into(),
            right.into().into(),
            parent.into().into(),
            child.into().into(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }

    /// Add a row with metadata.
    ///
    /// See [crate::TableCollection::add_edge_with_metadata] for examples
    pub fn add_row_with_metadata<
        L: Into<Position>,
        R: Into<Position>,
        P: Into<NodeId>,
        C: Into<NodeId>,
        M: EdgeMetadata,
    >(
        &mut self,
        left: L,
        right: R,
        parent: P,
        child: C,
        metadata: &M,
    ) -> Result<EdgeId, TskitError> {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let rv = self.table_.add_row_with_metadata(
            left.into().into(),
            right.into().into(),
            parent.into().into(),
            child.into().into(),
            md.as_slice(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }
}
