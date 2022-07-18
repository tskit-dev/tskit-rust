use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::Position;
use crate::SiteId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`SiteTable`]
pub struct SiteTableRow {
    pub id: SiteId,
    pub position: Position,
    pub ancestral_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for SiteTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && crate::util::partial_cmp_equal(&self.position, &other.position)
            && self.ancestral_state == other.ancestral_state
            && self.metadata == other.metadata
    }
}

fn make_site_table_row(table: &SiteTable, pos: tsk_id_t) -> Option<SiteTableRow> {
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let table_ref = &unsafe { *table.table_ };
        let rv = SiteTableRow {
            id: pos.into(),
            position: table.position(pos).unwrap(),
            ancestral_state: table.ancestral_state(pos).unwrap(),
            metadata: table_row_decode_metadata!(table_ref, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub(crate) type SiteTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a SiteTable>;
pub(crate) type SiteTableIterator<'a> = crate::table_iterator::TableIterator<SiteTable>;

impl<'a> Iterator for SiteTableRefIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for SiteTableIterator<'a> {
    type Item = SiteTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_site_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::sites`](crate::TableAccess::sites)
/// to get a reference to an existing site table;
pub struct SiteTable {
    table_: *const ll_bindings::tsk_site_table_t,
}

impl SiteTable {
    fn as_ll_ref(&self) -> &ll_bindings::tsk_site_table_t {
        // SAFETY: cannot be constructed with null pointer
        unsafe { &(*self.table_) }
    }

    pub(crate) fn new_from_table(sites: &ll_bindings::tsk_site_table_t) -> Self {
        SiteTable { table_: sites }
    }

    pub(crate) fn new_null() -> Self {
        Self {
            table_: std::ptr::null(),
        }
    }

    pub(crate) fn set_ptr(&mut self, ptr: *const ll_bindings::tsk_site_table_t) {
        assert!(!ptr.is_null());
        self.table_ = ptr;
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ll_ref().num_rows.into()
    }

    /// Return the ``position`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn position<S: Into<SiteId> + Copy>(&self, row: S) -> Result<Position, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.as_ll_ref().position)
        {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(e),
        }
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no ancestral state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn ancestral_state<S: Into<SiteId>>(&self, row: S) -> Result<Option<Vec<u8>>, TskitError> {
        crate::metadata::char_column_to_vector(
            self.as_ll_ref().ancestral_state,
            self.as_ll_ref().ancestral_state_offset,
            row.into().0,
            self.as_ll_ref().num_rows,
            self.as_ll_ref().ancestral_state_length,
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &self,
        row: SiteId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = &unsafe { *self.table_ };
        let buffer = metadata_to_vector!(table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`SiteTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = SiteTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&SiteTable>(self)
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
    pub fn row<S: Into<SiteId> + Copy>(&self, r: S) -> Result<SiteTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(r.into().0, self, make_site_table_row)
    }
}
