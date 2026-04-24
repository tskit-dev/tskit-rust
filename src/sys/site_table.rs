use std::ffi::c_char;
use std::ptr::NonNull;

use super::bindings::tsk_size_t;
use super::newtypes::Position;
use super::newtypes::SiteId;

use super::bindings::tsk_id_t;
use super::bindings::tsk_site_table_add_row;
use super::bindings::tsk_site_table_clear;
use super::bindings::tsk_site_table_init;
use super::bindings::tsk_site_table_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct SiteTable(TskBox<tsk_site_table_t>);

pub struct SiteTableIter<'table> {
    table: &'table SiteTable,
    current_row: SiteId,
}

impl<'table> Iterator for SiteTableIter<'table> {
    type Item = super::Site<'table, SiteTable>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.current_row;
        self.current_row += 1;
        self.table.row(c)
    }
}

impl SiteTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk =
            TskBox::new(|e: *mut tsk_site_table_t| unsafe { tsk_site_table_init(e, options) })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_site_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_site_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_site_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_site_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        position: f64,
        ancestral_state: Option<&[u8]>,
    ) -> Result<tsk_id_t, TskitError> {
        self.add_row_with_metadata(position, ancestral_state, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        position: f64,
        ancestral_state: Option<&[u8]>,
        metadata: &[u8],
    ) -> Result<tsk_id_t, TskitError> {
        unsafe {
            Ok(tsk_site_table_add_row(
                self.as_mut(),
                position,
                match ancestral_state {
                    Some(d) => d.as_ptr() as *const c_char,
                    None => std::ptr::null(),
                },
                match ancestral_state {
                    Some(d) => d.len() as u64,
                    None => 0,
                },
                metadata.as_ptr().cast::<c_char>(),
                metadata.len() as u64,
            ))
        }
    }

    pub fn position(&self, row: SiteId) -> Option<Position> {
        safe_tsk_column_access!(self, row, Position, position)
    }

    raw_metadata_getter_for_tables!(SiteId);

    fn ancestral_state_column(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().ancestral_state.cast::<u8>(),
                self.as_ref().ancestral_state_length as usize,
            )
        }
    }

    fn ancestral_state_offset_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().ancestral_state_offset,
                self.as_ref().num_rows as usize,
            )
        }
    }

    pub fn ancestral_state(&self, row: SiteId) -> Option<&[u8]> {
        super::tsk_ragged_column_access(
            row,
            self.ancestral_state_column(),
            self.ancestral_state_offset_raw(),
        )
    }

    pub fn row<'table>(&self, row: SiteId) -> Option<super::Site<'table, Self>> {
        let mut site =
            unsafe { std::mem::MaybeUninit::<super::bindings::tsk_site_t>::zeroed().assume_init() };
        let rv = unsafe {
            super::bindings::tsk_site_table_get_row(
                self.as_ref(),
                row.into(),
                &mut site as *mut super::bindings::tsk_site_t,
            )
        };
        if rv == 0 {
            Some(super::Site {
                row: site,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = super::Site<'_, Self>> {
        SiteTableIter {
            table: self,
            current_row: 0.into(),
        }
    }
}

impl Default for SiteTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
