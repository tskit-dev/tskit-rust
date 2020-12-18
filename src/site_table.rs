use crate::bindings as ll_bindings;
use crate::TskitRustError;
use crate::{tsk_id_t, tsk_size_t};

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableCollection::sites`](crate::TableCollection::sites)
/// to get a reference to an existing site table;
pub struct SiteTable<'a> {
    table_: &'a ll_bindings::tsk_site_table_t,
}

impl<'a> SiteTable<'a> {
    pub(crate) fn new_from_table(sites: &'a ll_bindings::tsk_site_table_t) -> Self {
        SiteTable { table_: sites }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
    }

    /// Return the ``position`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn position(&'a self, row: tsk_id_t) -> Result<f64, TskitRustError> {
        unsafe_tsk_column_access!(row, 0, self.num_rows(), self.table_.position);
    }

    /// Get the ``ancestral_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no ancestral state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitRustError::IndexError)
    /// if ``row`` is out of range.
    pub fn ancestral_state(&'a self, row: tsk_id_t) -> Result<Option<Vec<u8>>, TskitRustError> {
        if row < 0 || (row as tsk_size_t) >= self.num_rows() {
            return Err(TskitRustError::IndexError {});
        }
        if self.table_.ancestral_state_length == 0 {
            return Ok(None);
        }
        let start = unsafe { *self.table_.ancestral_state_offset.offset(row as isize) };
        let stop = if (row as tsk_size_t) < self.table_.num_rows {
            unsafe {
                *self
                    .table_
                    .ancestral_state_offset
                    .offset((row + 1) as isize)
            }
        } else {
            self.table_.ancestral_state_length
        };
        if stop - start == 0 {
            return Ok(None);
        }
        let mut buffer: Vec<u8> = vec![];
        for i in start..stop {
            buffer.push(unsafe { *self.table_.ancestral_state.offset(i as isize) } as u8);
        }
        Ok(Some(buffer))
    }
}
