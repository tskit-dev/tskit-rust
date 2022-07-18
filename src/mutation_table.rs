use crate::bindings as ll_bindings;
use crate::metadata;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{MutationId, NodeId, SiteId};

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
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let table_ref = &unsafe { *table.table_ };
        let rv = MutationTableRow {
            id: pos.into(),
            site: table.site(pos).unwrap(),
            node: table.node(pos).unwrap(),
            parent: table.parent(pos).unwrap(),
            time: table.time(pos).unwrap(),
            derived_state: table.derived_state(pos).unwrap(),
            metadata: table_row_decode_metadata!(table_ref, pos),
        };
        Some(rv)
    } else {
        None
    }
}
pub(crate) type MutationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MutationTable>;
pub(crate) type MutationTableIterator<'a> = crate::table_iterator::TableIterator<MutationTable>;

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
pub struct MutationTable {
    table_: *const ll_bindings::tsk_mutation_table_t,
}

impl MutationTable {
    fn as_ll_ref(&self) -> &ll_bindings::tsk_mutation_table_t {
        // SAFETY: cannot be constructed with null pointer
        unsafe { &(*self.table_) }
    }

    pub(crate) fn new_from_table(mutations: &ll_bindings::tsk_mutation_table_t) -> Self {
        MutationTable { table_: mutations }
    }

    pub(crate) fn new_null() -> Self {
        Self {
            table_: std::ptr::null(),
        }
    }

    pub(crate) fn set_ptr(&mut self, ptr: *const ll_bindings::tsk_mutation_table_t) {
        assert!(!ptr.is_null());
        self.table_ = ptr;
    }

    /// Return the number of rows.
    pub fn num_rows(&self) -> SizeType {
        self.as_ll_ref().num_rows.into()
    }

    /// Return the ``site`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn site<M: Into<MutationId> + Copy>(&self, row: M) -> Result<SiteId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().site,
            SiteId
        )
    }

    /// Return the ``node`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn node<M: Into<MutationId> + Copy>(&self, row: M) -> Result<NodeId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().node,
            NodeId
        )
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent<M: Into<MutationId> + Copy>(&self, row: M) -> Result<MutationId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().parent,
            MutationId
        )
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time<M: Into<MutationId> + Copy>(&self, row: M) -> Result<Time, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.as_ll_ref().time) {
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
        &self,
        row: M,
    ) -> Result<Option<Vec<u8>>, TskitError> {
        metadata::char_column_to_vector(
            self.as_ll_ref().derived_state,
            self.as_ll_ref().derived_state_offset,
            row.into().0,
            self.as_ll_ref().num_rows,
            self.as_ll_ref().derived_state_length,
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &self,
        row: MutationId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = &unsafe { *self.table_ };
        let buffer = metadata_to_vector!(table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MutationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = MutationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&MutationTable>(self)
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
