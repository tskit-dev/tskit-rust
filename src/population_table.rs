use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::PopulationId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`PopulationTable`]
#[derive(Eq)]
pub struct PopulationTableRow {
    pub id: PopulationId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for PopulationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.metadata == other.metadata
    }
}

fn make_population_table_row(table: &PopulationTable, pos: tsk_id_t) -> Option<PopulationTableRow> {
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let table_ref = unsafe { *table.table_ };
        let rv = PopulationTableRow {
            id: pos.into(),
            metadata: table_row_decode_metadata!(table_ref, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub(crate) type PopulationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a PopulationTable>;
pub(crate) type PopulationTableIterator<'a> = crate::table_iterator::TableIterator<PopulationTable>;

impl<'a> Iterator for PopulationTableRefIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for PopulationTableIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::populations`](crate::TableAccess::populations)
/// to get a reference to an existing population table;
pub struct PopulationTable {
    table_: *const ll_bindings::tsk_population_table_t,
}

impl PopulationTable {
    fn ll_table_ref(&self) -> &ll_bindings::tsk_population_table_t {
        unsafe { &(*self.table_) }
    }

    pub(crate) fn new_from_table(populations: *const ll_bindings::tsk_population_table_t) -> Self {
        assert!(!populations.is_null());
        PopulationTable {
            table_: populations,
        }
    }

    /// Return the number of rows.
    pub fn num_rows(&self) -> SizeType {
        self.ll_table_ref().num_rows.into()
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &self,
        row: PopulationId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = unsafe { *self.table_ };
        let buffer = metadata_to_vector!(table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`PopulationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = PopulationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&PopulationTable>(self)
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
    pub fn row<P: Into<PopulationId> + Copy>(
        &self,
        r: P,
    ) -> Result<PopulationTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_population_table_row)
    }
}

pub struct OwningPopulationTable(PopulationTable);

impl OwningPopulationTable {
    fn new() -> Self {
        let table_ = unsafe {
            libc::malloc(std::mem::size_of::<ll_bindings::tsk_population_table_t>())
                as *mut ll_bindings::tsk_population_table_t
        };
        //let nonnull = match std::ptr::NonNull::<ll_bindings::tsk_population_table_t>::new(temp) {
        //    Some(x) => x,
        //    None => panic!("out of memory"),
        //};
        //let mbox = unsafe { MBox::from_non_null_raw(nonnull) };
        Self {
            0: PopulationTable { table_ },
        }
    }
}

impl OwningPopulationTable {
    fn add_row(&mut self) -> i32 {
        let rv = unsafe {
            let ptr = self.0.table_ as *mut ll_bindings::tsk_population_table_t;
            ll_bindings::tsk_population_table_add_row(ptr, std::ptr::null(), 0)
        };
        rv
    }
}

impl std::ops::Deref for OwningPopulationTable {
    type Target = PopulationTable;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for OwningPopulationTable {
    fn drop(&mut self) {
        unsafe {
            ll_bindings::tsk_population_table_free(
                self.table_ as *mut ll_bindings::tsk_population_table_t,
            )
        };
        assert!(!self.0.table_.is_null());
        unsafe { libc::free(self.0.table_ as *mut libc::c_void) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_population() {
        let mut p = OwningPopulationTable::new();
        let x = p.add_row();
        assert_eq!(x, 0);
        assert_eq!(p.num_rows(), 1);
    }
}
