use crate::metadata;
use crate::metadata::MigrationMetadata;
use crate::sys;
use crate::Position;
use crate::SizeType;
use crate::Time;
use crate::TskitError;
use crate::{MigrationId, NodeId, PopulationId};
use sys::bindings as ll_bindings;

/// A migration table.
///
/// # Examples
///
/// ```
/// use tskit::MigrationTable;
///
/// let mut migrations = MigrationTable::default();
/// let rowid = migrations.add_row((0., 1.), 1, (0, 1), 10.3).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(migrations.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::MigrationTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::MigrationMetadata)]
/// #[serializer("serde_json")]
/// struct MigrationMetadata {
///     value: i32,
/// }
///
/// let metadata = MigrationMetadata{value: 42};
///
/// let mut migrations = MigrationTable::default();
///
/// let rowid = migrations.add_row_with_metadata((0., 1.), 1, (0, 1), 10.3, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match migrations.metadata::<MigrationMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
///
/// # }
/// ```
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct MigrationTable {
    table_: sys::MigrationTable,
}

impl MigrationTable {
    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        migrations: *mut ll_bindings::tsk_migration_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(migrations).unwrap();
        let table_ = unsafe { sys::MigrationTable::new_borrowed(ptr) };
        Ok(MigrationTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_migration_table_t {
        self.table_.as_ref()
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    /// Return the left coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `row` is valid.
    /// * `None` otherwise.
    pub fn left<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Position> {
        self.table_.left(row.into())
    }

    /// Return the right coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(positions)` if `row` is valid.
    /// * `None` otherwise.
    pub fn right<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Position> {
        self.table_.right(row.into())
    }

    /// Return the node for a given row.
    ///
    /// # Returns
    //
    /// * `Some(node)` if `row` is valid.
    /// * `None` otherwise.
    pub fn node<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<NodeId> {
        self.table_.node(row.into())
    }

    /// Return the source population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn source<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<PopulationId> {
        self.table_.source(row.into())
    }

    /// Return the destination population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn dest<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<PopulationId> {
        self.table_.dest(row.into())
    }

    /// Return the time of the migration event for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(time)` if `row` is valid.
    /// * `None` otherwise.
    pub fn time<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Time> {
        self.table_.time(row.into())
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
    pub fn metadata<T: metadata::MigrationMetadata>(
        &self,
        row: impl Into<MigrationId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`crate::Migration`].
    pub fn iter(&self) -> impl Iterator<Item = crate::Migration<'_, crate::sys::MigrationTable>> {
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
    pub fn row<M: Into<MigrationId> + Copy>(
        &self,
        r: M,
    ) -> Option<crate::Migration<'_, crate::sys::MigrationTable>> {
        self.table_.row(r.into())
    }

    build_table_column_slice_getter!(
        /// Get the left column as a slice
        => left, left_slice, Position);
    build_table_column_slice_getter!(
        /// Get the left column as a slice
        => left, left_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the right column as a slice
        => right, right_slice, Position);
    build_table_column_slice_getter!(
        /// Get the right column as a slice
        => right, right_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice, Time);
    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the node column as a slice
        => node, node_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the node column as a slice
        => node, node_slice_raw, ll_bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the source column as a slice
        => source, source_slice, PopulationId);
    build_table_column_slice_getter!(
        /// Get the source column as a slice
        => source, source_slice_raw, ll_bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the dest column as a slice
        => dest, dest_slice, PopulationId);
    build_table_column_slice_getter!(
        /// Get the dest column as a slice
        => dest, dest_slice_raw, ll_bindings::tsk_id_t);

    pub fn left_column(&self) -> impl crate::TableColumn<MigrationId, Position> + '_ {
        crate::table_column::OpaqueTableColumn(self.left_slice())
    }

    pub fn right_column(&self) -> impl crate::TableColumn<MigrationId, Position> + '_ {
        crate::table_column::OpaqueTableColumn(self.right_slice())
    }

    pub fn node_column(&self) -> impl crate::TableColumn<MigrationId, NodeId> + '_ {
        crate::table_column::OpaqueTableColumn(self.node_slice())
    }

    pub fn time_column(&self) -> impl crate::TableColumn<MigrationId, Time> + '_ {
        crate::table_column::OpaqueTableColumn(self.time_slice())
    }

    pub fn source_column(&self) -> impl crate::TableColumn<MigrationId, PopulationId> + '_ {
        crate::table_column::OpaqueTableColumn(self.source_slice())
    }

    pub fn dest_column(&self) -> impl crate::TableColumn<MigrationId, PopulationId> + '_ {
        crate::table_column::OpaqueTableColumn(self.dest_slice())
    }

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }

    pub fn add_row<LEFT, RIGHT, N, SOURCE, DEST, T>(
        &mut self,
        span: (LEFT, RIGHT),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
    ) -> Result<MigrationId, TskitError>
    where
        LEFT: Into<Position>,
        RIGHT: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        T: Into<Time>,
    {
        let rv = self.table_.add_row(
            (span.0.into().into(), span.1.into().into()),
            node.into().into(),
            source_dest.0.into().into(),
            source_dest.1.into().into(),
            time.into().into(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row_with_metadata<LEFT, RIGHT, N, SOURCE, DEST, T, M>(
        &mut self,
        span: (LEFT, RIGHT),
        node: N,
        source_dest: (SOURCE, DEST),
        time: T,
        metadata: &M,
    ) -> Result<MigrationId, TskitError>
    where
        LEFT: Into<Position>,
        RIGHT: Into<Position>,
        N: Into<NodeId>,
        SOURCE: Into<PopulationId>,
        DEST: Into<PopulationId>,
        T: Into<Time>,
        M: MigrationMetadata,
    {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let rv = self.table_.add_row_with_metadata(
            (span.0.into().into(), span.1.into().into()),
            node.into().into(),
            source_dest.0.into().into(),
            source_dest.1.into().into(),
            time.into().into(),
            md.as_slice(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }
}
