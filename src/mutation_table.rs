use crate::bindings as ll_bindings;
use crate::metadata;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{MutationId, NodeId, SiteId};
use ll_bindings::{tsk_mutation_table_free, tsk_mutation_table_init};

/// Row of a [`MutationTable`]
pub struct MutationTableRow {
    pub id: MutationId,
    pub site: SiteId,
    pub node: NodeId,
    pub parent: MutationId,
    pub time: Time,
    pub derived_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for MutationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.derived_state == other.derived_state
            && self.metadata == other.metadata
    }
}

fn make_mutation_table_row(table: &MutationTable, pos: tsk_id_t) -> Option<MutationTableRow> {
    let table_ref = table.table_;
    Some(MutationTableRow {
        id: pos.into(),
        site: table.site(pos).ok()?,
        node: table.node(pos).ok()?,
        parent: table.parent(pos).ok()?,
        time: table.time(pos).ok()?,
        derived_state: table.derived_state(pos).ok()?.map(|s| s.to_vec()),
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}
pub(crate) type MutationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MutationTable<'a>>;
pub(crate) type MutationTableIterator<'a> = crate::table_iterator::TableIterator<MutationTable<'a>>;

impl<'a> Iterator for MutationTableRefIterator<'a> {
    type Item = MutationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_mutation_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for MutationTableIterator<'a> {
    type Item = MutationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_mutation_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::mutations`](crate::TableAccess::mutations)
/// to get a reference to an existing mutation table;
pub struct MutationTable<'a> {
    table_: &'a ll_bindings::tsk_mutation_table_t,
}

impl<'a> MutationTable<'a> {
    pub(crate) fn new_from_table(mutations: &'a ll_bindings::tsk_mutation_table_t) -> Self {
        MutationTable { table_: mutations }
    }

    /// Return the number of rows.
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
    }

    /// Return the ``site`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn site<M: Into<MutationId> + Copy>(&'a self, row: M) -> Result<SiteId, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.site, SiteId)
    }

    /// Return the ``node`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn node<M: Into<MutationId> + Copy>(&'a self, row: M) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.node, NodeId)
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent<M: Into<MutationId> + Copy>(&'a self, row: M) -> Result<MutationId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.parent,
            MutationId
        )
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time<M: Into<MutationId> + Copy>(&'a self, row: M) -> Result<Time, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.time) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(e),
        }
    }

    /// Get the ``derived_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no derived state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn derived_state<M: Into<MutationId>>(
        &'a self,
        row: M,
    ) -> Result<Option<&[u8]>, TskitError> {
        metadata::char_column_to_slice(
            self,
            self.table_.derived_state,
            self.table_.derived_state_offset,
            row.into().0,
            self.table_.num_rows,
            self.table_.derived_state_length,
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: MutationId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MutationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = MutationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&MutationTable<'a>>(self)
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
    pub fn row<M: Into<MutationId> + Copy>(&self, r: M) -> Result<MutationTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_mutation_table_row)
    }
}

build_owned_table_type!(
/// A standalone mutation table that owns its data.
///
/// # Examples
///
/// ```
/// use tskit::OwnedMutationTable;
///
/// let mut mutations = OwnedMutationTable::default();
/// let rowid = mutations.add_row(1, 2, 0, 1.0, None).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(mutations.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::OwnedMutationTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::MutationMetadata)]
/// #[serializer("serde_json")]
/// struct MutationMetadata {
///     value: i32,
/// }
///
/// let metadata = MutationMetadata{value: 42};
///
/// let mut mutations = OwnedMutationTable::default();
///
/// let rowid = mutations.add_row_with_metadata(0, 1, 5, 10.0, None, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// if let Some(decoded) = mutations.metadata::<MutationMetadata>(rowid).unwrap() {
///     assert_eq!(decoded.value, 42);
/// } else {
///     panic!("hmm...we expected some metadata!");
/// }
///
/// # }
/// ```
    => OwnedMutationTable,
    MutationTable,
    tsk_mutation_table_t,
    tsk_mutation_table_init,
    tsk_mutation_table_free,
    ll_bindings::tsk_mutation_table_clear
);

impl OwnedMutationTable {
    mutation_table_add_row!(=> add_row, self, *self.table);
    mutation_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
