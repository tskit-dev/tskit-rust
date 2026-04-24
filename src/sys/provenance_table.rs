use std::ptr::NonNull;

use super::newtypes::ProvenanceId;

use super::bindings::tsk_id_t;
use super::bindings::tsk_provenance_table_add_row;
use super::bindings::tsk_provenance_table_clear;
use super::bindings::tsk_provenance_table_init;
use super::bindings::tsk_provenance_table_t;
use super::bindings::tsk_size_t;
use super::tskbox::TskBox;
use super::TskitError;
use std::ffi::c_char;

#[derive(Debug)]
pub struct ProvenanceTable(TskBox<tsk_provenance_table_t>);

pub struct ProvenanceTableIter<'table> {
    table: &'table ProvenanceTable,
    current_row: ProvenanceId,
}

impl<'table> Iterator for ProvenanceTableIter<'table> {
    type Item = super::Provenance<'table, ProvenanceTable>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.current_row;
        self.current_row += 1;
        self.table.row(c)
    }
}

impl ProvenanceTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk = TskBox::new(|e: *mut tsk_provenance_table_t| unsafe {
            tsk_provenance_table_init(e, options)
        })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_provenance_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_provenance_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_provenance_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_provenance_table_clear(self.as_mut()) }
    }

    pub fn add_row(&mut self, record: &str) -> Result<tsk_id_t, TskitError> {
        if record.is_empty() {
            return Err(TskitError::LibraryError(
                "empty provenance record".to_owned(),
            ));
        }
        let timestamp = chrono::Utc::now().to_string();
        let rv = unsafe {
            tsk_provenance_table_add_row(
                self.as_mut(),
                timestamp.as_ptr() as *const c_char,
                timestamp.len() as tsk_size_t,
                record.as_ptr() as *const c_char,
                record.len() as tsk_size_t,
            )
        };
        Ok(rv)
    }

    fn timestamp_column(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().timestamp.cast::<u8>(),
                self.as_ref().timestamp_length as usize,
            )
        }
    }

    fn timestamp_offset_column_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().timestamp_offset,
                self.as_ref().num_rows as usize,
            )
        }
    }

    pub fn timestamp(&self, row: ProvenanceId) -> Option<&str> {
        let timestamp_slice = super::tsk_ragged_column_access(
            row,
            self.timestamp_column(),
            self.timestamp_offset_column_raw(),
        );
        match timestamp_slice {
            Some(tstamp) => std::str::from_utf8(tstamp).ok(),
            None => None,
        }
    }

    fn record_column(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().record.cast::<u8>(),
                self.as_ref().record_length as usize,
            )
        }
    }

    fn record_offset_column_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(self.as_ref().record_offset, self.as_ref().num_rows as usize)
        }
    }

    pub fn record(&self, row: ProvenanceId) -> Option<&str> {
        let record_slice = super::tsk_ragged_column_access(
            row,
            self.record_column(),
            self.record_offset_column_raw(),
        );
        match record_slice {
            Some(rec) => std::str::from_utf8(rec).ok(),
            None => None,
        }
    }

    pub fn row<'table>(&self, row: ProvenanceId) -> Option<super::Provenance<'table, Self>> {
        let mut provenance = unsafe {
            std::mem::MaybeUninit::<super::bindings::tsk_provenance_t>::zeroed().assume_init()
        };
        let rv = unsafe {
            super::bindings::tsk_provenance_table_get_row(
                self.as_ref(),
                row.into(),
                &mut provenance as *mut super::bindings::tsk_provenance_t,
            )
        };
        if rv == 0 {
            Some(super::Provenance {
                row: provenance,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = super::Provenance<'_, Self>> {
        ProvenanceTableIter {
            table: self,
            current_row: 0.into(),
        }
    }
}

impl Default for ProvenanceTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
