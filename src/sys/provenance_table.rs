#![cfg(feature = "provenance")]

use std::ptr::NonNull;

use super::bindings::tsk_id_t;
use super::bindings::tsk_provenance_table_add_row;
use super::bindings::tsk_provenance_table_clear;
use super::bindings::tsk_provenance_table_init;
use super::bindings::tsk_provenance_table_t;
use super::bindings::tsk_size_t;
use super::tskbox::TskBox;
use super::Error;

#[derive(Debug)]
pub struct ProvenanceTable(TskBox<tsk_provenance_table_t>);

impl ProvenanceTable {
    pub fn new(options: u32) -> Result<Self, Error> {
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

    pub fn add_row(&mut self, record: &str) -> Result<tsk_id_t, Error> {
        if record.is_empty() {
            return Err(Error::Message("empty provenance record".to_owned()));
        }
        let timestamp = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        let rv = unsafe {
            tsk_provenance_table_add_row(
                self.as_mut(),
                timestamp.as_ptr() as *mut i8,
                timestamp.len() as tsk_size_t,
                record.as_ptr() as *mut i8,
                record.len() as tsk_size_t,
            )
        };
        Ok(rv)
    }
}

impl Default for ProvenanceTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
