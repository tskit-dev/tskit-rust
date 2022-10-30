use crate::bindings as ll_bindings;
use crate::metadata;
use crate::Position;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{MigrationId, NodeId, PopulationId};
use ll_bindings::{tsk_migration_table_free, tsk_migration_table_init};

/// Row of a [`MigrationTable`]
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
    let table_ref = table.table_;
    Some(MigrationTableRow {
        id: pos.into(),
        left: table.left(pos)?,
        right: table.right(pos)?,
        node: table.node(pos)?,
        source: table.source(pos)?,
        dest: table.dest(pos)?,
        time: table.time(pos)?,
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type MigrationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MigrationTable<'a>>;
pub(crate) type MigrationTableIterator<'a> =
    crate::table_iterator::TableIterator<MigrationTable<'a>>;

impl<'a> Iterator for MigrationTableRefIterator<'a> {
    type Item = MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for MigrationTableIterator<'a> {
    type Item = crate::migration_table::MigrationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_migration_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of a migration table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::migrations`](crate::TableAccess::migrations)
/// to get a reference to an existing node table;
pub struct MigrationTable<'a> {
    table_: &'a ll_bindings::tsk_migration_table_t,
}

impl<'a> MigrationTable<'a> {
    pub(crate) fn new_from_table(migrations: &'a ll_bindings::tsk_migration_table_t) -> Self {
        MigrationTable { table_: migrations }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
    }

    /// Return the left coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(position)` if `row` is valid.
    /// * `None` otherwise.
    pub fn left<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<Position> {
        unsafe_tsk_column_access_and_map_into!(row.into().0, 0, self.num_rows(), self.table_.left)
    }

    /// Return the right coordinate for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(positions)` if `row` is valid.
    /// * `None` otherwise.
    pub fn right<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<Position> {
        unsafe_tsk_column_access_and_map_into!(row.into().0, 0, self.num_rows(), self.table_.right)
    }

    /// Return the node for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(node)` if `row` is valid.
    /// * `None` otherwise.
    pub fn node<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<NodeId> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.node, NodeId)
    }

    /// Return the source population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn source<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<PopulationId> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.source,
            PopulationId
        )
    }

    /// Return the destination population for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn dest<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<PopulationId> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.dest,
            PopulationId
        )
    }

    /// Return the time of the migration event for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(time)` if `row` is valid.
    /// * `None` otherwise.
    pub fn time<M: Into<MigrationId> + Copy>(&'a self, row: M) -> Option<Time> {
        unsafe_tsk_column_access_and_map_into!(row.into().0, 0, self.num_rows(), self.table_.time)
    }

    /// Return the metadata for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(metadata))` if `row` is valid and decoding succeeded
    /// * `Some(Err(_))` if `row` is valid and decoding failed.
    /// * `None` if `row` is not valid.
    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: MigrationId,
    ) -> Option<Result<T, TskitError>> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MigrationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = MigrationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&MigrationTable<'a>>(self)
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
    pub fn row<M: Into<MigrationId> + Copy>(&self, r: M) -> Result<MigrationTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(r.into().0, self, make_migration_table_row)
    }
}

build_owned_table_type!(
    /// A standalone migration table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwnedMigrationTable;
    ///
    /// let mut migrations = OwnedMigrationTable::default();
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
    /// use tskit::OwnedMigrationTable;
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
    /// let mut migrations = OwnedMigrationTable::default();
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
    => OwnedMigrationTable,
    MigrationTable,
    tsk_migration_table_t,
    tsk_migration_table_init,
    tsk_migration_table_free,
    ll_bindings::tsk_migration_table_clear
);

impl OwnedMigrationTable {
    migration_table_add_row!(=> add_row, self, *self.table);
    migration_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
