use crate::bindings as ll_bindings;
use crate::{tsk_id_t, tsk_size_t, TskitRustError};

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
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn site(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitRustError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.site);
    }

    /// Return the ``node`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn node(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitRustError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.node);
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent(&'a self, row: tsk_id_t) -> Result<tsk_id_t, TskitRustError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.parent);
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn time(&'a self, row: tsk_id_t) -> Result<f64, TskitRustError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.time);
    }

    /// Get the ``derived_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no derived state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn derived_state(&'a self, row: tsk_id_t) -> Result<Option<Vec<u8>>, TskitRustError> {
        if row < 0 || (row as tsk_size_t) >= self.num_rows() {
            return Err(TskitRustError::IndexError {});
        }
        if self.table_.derived_state_length == 0 {
            return Ok(None);
        }
        let start = unsafe { *self.table_.derived_state_offset.offset(row as isize) };
        let stop = if (row as tsk_size_t) < self.table_.num_rows {
            unsafe { *self.table_.derived_state_offset.offset((row + 1) as isize) }
        } else {
            self.table_.derived_state_length
        };
        if stop - start == 0 {
            return Ok(None);
        }
        let mut buffer: Vec<u8> = vec![];
        for i in start..stop {
            buffer.push(unsafe { *self.table_.derived_state.offset(i as isize) } as u8);
        }
        Ok(Some(buffer))
    }
}
