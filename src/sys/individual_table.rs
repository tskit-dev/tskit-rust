use std::ptr::NonNull;

use super::bindings::tsk_size_t;
use super::flags::IndividualFlags;
use super::newtypes::IndividualId;

use super::bindings::tsk_flags_t;
use super::bindings::tsk_id_t;
use super::bindings::tsk_individual_table_add_row;
use super::bindings::tsk_individual_table_clear;
use super::bindings::tsk_individual_table_init;
use super::bindings::tsk_individual_table_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct IndividualTable(TskBox<tsk_individual_table_t>);

impl IndividualTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk = TskBox::new(|e: *mut tsk_individual_table_t| unsafe {
            tsk_individual_table_init(e, options)
        })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_individual_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_individual_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_individual_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_individual_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
    ) -> Result<tsk_id_t, TskitError> {
        self.add_row_with_metadata(flags, location, parents, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        flags: tsk_flags_t,
        location: &[f64],
        parents: &[tsk_id_t],
        metadata: &[u8],
    ) -> Result<tsk_id_t, TskitError> {
        unsafe {
            Ok(tsk_individual_table_add_row(
                self.as_mut(),
                flags,
                location.as_ptr(),
                location.len() as u64,
                parents.as_ptr(),
                parents.len() as u64,
                metadata.as_ptr().cast::<i8>(),
                metadata.len() as u64,
            ))
        }
    }

    pub fn flags(&self, row: IndividualId) -> Option<IndividualFlags> {
        safe_tsk_column_access!(self, row, IndividualFlags, flags)
    }

    raw_metadata_getter_for_tables!(IndividualId);

    pub fn location_column(&self) -> &[super::newtypes::Location] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().location.cast::<super::newtypes::Location>(),
                self.as_ref().location_length as usize,
            )
        }
    }

    fn location_offset_column_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().location_offset,
                self.as_ref().num_rows as usize,
            )
        }
    }

    pub fn location(&self, row: IndividualId) -> Option<&[super::newtypes::Location]> {
        super::tsk_ragged_column_access(
            row,
            self.location_column(),
            self.location_offset_column_raw(),
        )
    }

    fn parents_column(&self) -> &[IndividualId] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().parents.cast::<IndividualId>(),
                self.as_ref().parents_length as usize,
            )
        }
    }

    fn parents_offset_column_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().parents_offset,
                self.as_ref().num_rows as usize,
            )
        }
    }

    pub fn parents(&self, row: IndividualId) -> Option<&[IndividualId]> {
        assert!(
            (self.as_ref().num_rows == 0 && self.as_ref().parents_length == 0)
                || (!self.as_ref().parents.is_null() && !self.as_ref().location_offset.is_null())
        );
        super::tsk_ragged_column_access(
            row,
            self.parents_column(),
            self.parents_offset_column_raw(),
        )
    }
}

impl Default for IndividualTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
