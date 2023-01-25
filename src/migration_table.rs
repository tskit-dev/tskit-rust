use crate::bindings as ll_bindings;
use crate::metadata;
use crate::sys;
use crate::Position;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{MigrationId, NodeId, PopulationId};

/// Row of a [`MigrationTable`]
#[derive(Debug)]
pub struct MigrationTableRow {
    pub id: MigrationId,
    pub left: Position,
    pub right: Position,
    pub node: NodeId,
    pub source: PopulationId,
    pub dest: PopulationId,
    pub time: Time,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for MigrationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.node == other.node
            && self.source == other.source
            && self.dest == other.dest
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.metadata == other.metadata
    }
}

fn make_migration_table_row(table: &MigrationTable, pos: tsk_id_t) -> Option<MigrationTableRow> {
    Some(MigrationTableRow {
        id: pos.into(),
        left: table.left(pos)?,
        right: table.right(pos)?,
        node: table.node(pos)?,
        source: table.source(pos)?,
        dest: table.dest(pos)?,
        time: table.time(pos)?,
        metadata: table.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type MigrationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MigrationTable>;
pub(crate) type MigrationTableIterator = crate::table_iterator::TableIterator<MigrationTable>;

impl<'a> Iterator for MigrationTableRefIterator<'a> {
    type Item = MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for MigrationTableIterator {
    type Item = crate::migration_table::MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct MigrationTableRowView<'a> {
    table: &'a MigrationTable,
    pub id: MigrationId,
    pub left: Position,
    pub right: Position,
    pub node: NodeId,
    pub source: PopulationId,
    pub dest: PopulationId,
    pub time: Time,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> MigrationTableRowView<'a> {
    fn new(table: &'a MigrationTable) -> Self {
        Self {
            table,
            id: MigrationId::NULL,
            left: Position::from(f64::NAN),
            right: Position::from(f64::NAN),
            node: NodeId::NULL,
            source: PopulationId::NULL,
            dest: PopulationId::NULL,
            time: Time::from(f64::NAN),
            metadata: None,
        }
    }
}

impl<'a> PartialEq for MigrationTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.node == other.node
            && self.source == other.source
            && self.dest == other.dest
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.metadata == other.metadata
    }
}

impl<'a> Eq for MigrationTableRowView<'a> {}

impl<'a> PartialEq<MigrationTableRow> for MigrationTableRowView<'a> {
    fn eq(&self, other: &MigrationTableRow) -> bool {
        self.id == other.id
            && self.node == other.node
            && self.source == other.source
            && self.dest == other.dest
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<MigrationTableRowView<'_>> for MigrationTableRow {
    fn eq(&self, other: &MigrationTableRowView) -> bool {
        self.id == other.id
            && self.node == other.node
            && self.source == other.source
            && self.dest == other.dest
            && crate::util::partial_cmp_equal(&self.left, &other.left)
            && crate::util::partial_cmp_equal(&self.right, &other.right)
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for MigrationTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.left = self.table.left(self.id).unwrap_or_else(|| f64::NAN.into());
        self.right = self.table.right(self.id).unwrap_or_else(|| f64::NAN.into());
        self.node = self.table.node(self.id).unwrap_or(NodeId::NULL);
        self.source = self.table.source(self.id).unwrap_or(PopulationId::NULL);
        self.dest = self.table.dest(self.id).unwrap_or(PopulationId::NULL);
        self.time = self.table.time(self.id).unwrap_or_else(|| f64::NAN.into());
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// An migration table.
///
/// # Examples
///
/// ## Standalone table
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
#[derive(Debug)]
#[repr(transparent)]
pub struct MigrationTable {
    table_: sys::LLMigrationTable,
}

impl MigrationTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_migration_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_migration_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        migrations: *mut ll_bindings::tsk_migration_table_t,
    ) -> Result<Self, TskitError> {
        let table_ = sys::LLMigrationTable::new_non_owning(migrations)?;
        Ok(MigrationTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_migration_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::LLMigrationTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    raw_metadata_getter_for_tables!(MigrationId);

    /// Return the left coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `row` is valid.
    /// * `None` otherwise.
    pub fn left<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Position> {
        sys::tsk_column_access::<Position, _, _, _>(row.into(), self.as_ref().left, self.num_rows())
    }

    /// Return the right coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(positions)` if `row` is valid.
    /// * `None` otherwise.
    pub fn right<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Position> {
        sys::tsk_column_access::<Position, _, _, _>(
            row.into(),
            self.as_ref().right,
            self.num_rows(),
        )
    }

    /// Return the node for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(node)` if `row` is valid.
    /// * `None` otherwise.
    pub fn node<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<NodeId> {
        sys::tsk_column_access::<NodeId, _, _, _>(row.into(), self.as_ref().node, self.num_rows())
    }

    /// Return the source population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn source<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<PopulationId> {
        sys::tsk_column_access::<PopulationId, _, _, _>(
            row.into(),
            self.as_ref().source,
            self.num_rows(),
        )
    }

    /// Return the destination population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn dest<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<PopulationId> {
        sys::tsk_column_access::<PopulationId, _, _, _>(
            row.into(),
            self.as_ref().dest,
            self.num_rows(),
        )
    }

    /// Return the time of the migration event for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(time)` if `row` is valid.
    /// * `None` otherwise.
    pub fn time<M: Into<MigrationId> + Copy>(&self, row: M) -> Option<Time> {
        sys::tsk_column_access::<Time, _, _, _>(row.into(), self.as_ref().time, self.num_rows())
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
    pub fn metadata<T: metadata::MigrationMetadata>(
        &self,
        row: MigrationId,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MigrationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = MigrationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&MigrationTable>(self)
    }

    pub fn lending_iter(&self) -> MigrationTableRowView {
        MigrationTableRowView::new(self)
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
    pub fn row<M: Into<MigrationId> + Copy>(&self, r: M) -> Option<MigrationTableRow> {
        let ri = r.into().into();
        table_row_access!(ri, self, make_migration_table_row)
    }

    /// Return a view of `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<M: Into<MigrationId> + Copy>(&self, r: M) -> Option<MigrationTableRowView> {
        let view = MigrationTableRowView {
            table: self,
            id: r.into(),
            left: self.left(r)?,
            right: self.right(r)?,
            node: self.node(r)?,
            source: self.source(r)?,
            dest: self.dest(r)?,
            time: self.time(r)?,
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
    }

    pub fn clear(&mut self) -> Result<(), TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    migration_table_add_row!(=> add_row, self, self.as_mut_ptr());
    migration_table_add_row_with_metadata!(=> add_row_with_metadata, self, self.as_mut_ptr());

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
}

impl Default for MigrationTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

pub type OwningMigrationTable = MigrationTable;

impl OwningMigrationTable {}
