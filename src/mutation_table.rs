use crate::bindings as ll_bindings;
use crate::metadata;
use crate::{tsk_id_t, tsk_size_t, TskitError};
use crate::{MutationId, NodeId, SiteId};

/// Row of a [`MutationTable`]
pub struct MutationTableRow {
    pub id: MutationId,
    pub site: SiteId,
    pub node: NodeId,
    pub parent: MutationId,
    pub time: f64,
    pub derived_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for MutationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::f64_partial_cmp_equal(&self.time, &other.time)
            && self.derived_state == other.derived_state
            && self.metadata == other.metadata
    }
}

fn make_mutation_table_row(table: &MutationTable, pos: tsk_id_t) -> Option<MutationTableRow> {
    if pos < table.num_rows() as tsk_id_t {
        let rv = MutationTableRow {
            id: pos.into(),
            site: table.site(pos).unwrap(),
            node: table.node(pos).unwrap(),
            parent: table.parent(pos).unwrap(),
            time: table.time(pos).unwrap(),
            derived_state: table.derived_state(pos).unwrap(),
            metadata: table_row_decode_metadata!(table, pos),
        };
        Some(rv)
    } else {
        None
    }
}
pub type MutationTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a MutationTable<'a>>;
pub type MutationTableIterator<'a> = crate::table_iterator::TableIterator<MutationTable<'a>>;

impl<'a> Iterator for MutationTableRefIterator<'a> {
    type Item = MutationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_mutation_table_row(&self.table, self.pos);
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
/// Instead, use [`TableCollection::mutations`](crate::TableCollection::mutations)
/// to get a reference to an existing mutation table;
pub struct MutationTable<'a> {
    table_: &'a ll_bindings::tsk_mutation_table_t,
}

impl<'a> MutationTable<'a> {
    pub(crate) fn new_from_table(mutations: &'a ll_bindings::tsk_mutation_table_t) -> Self {
        MutationTable { table_: mutations }
    }

    /// Return the number of rows.
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
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
    pub fn time<M: Into<MutationId> + Copy>(&'a self, row: M) -> Result<f64, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.time)
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
    ) -> Result<Option<Vec<u8>>, TskitError> {
        metadata::char_column_to_vector(
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
        let buffer = metadata_to_vector!(self, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MutationTableRow`].
    pub fn iter(&self) -> MutationTableRefIterator {
        crate::table_iterator::make_table_iterator::<&MutationTable<'a>>(&self)
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
        table_row_access!(r.into().0, self, make_mutation_table_row)
    }
}
