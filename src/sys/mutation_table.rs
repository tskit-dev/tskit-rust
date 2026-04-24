use std::ffi::c_char;
use std::ptr::NonNull;

use super::newtypes::MutationId;
use super::newtypes::NodeId;
use super::newtypes::SiteId;
use super::newtypes::Time;

use super::bindings::tsk_id_t;
use super::bindings::tsk_mutation_table_add_row;
use super::bindings::tsk_mutation_table_clear;
use super::bindings::tsk_mutation_table_init;
use super::bindings::tsk_mutation_table_t;
use super::bindings::tsk_size_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct MutationTable(TskBox<tsk_mutation_table_t>);

pub struct MutationTableIter<'table> {
    table: &'table MutationTable,
    current_row: MutationId,
}

impl<'table> Iterator for MutationTableIter<'table> {
    type Item = super::Mutation<'table, MutationTable>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.current_row;
        self.current_row += 1;
        self.table.row(c)
    }
}

impl MutationTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk = TskBox::new(|e: *mut tsk_mutation_table_t| unsafe {
            tsk_mutation_table_init(e, options)
        })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_mutation_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_mutation_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_mutation_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_mutation_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        site: tsk_id_t,
        node: tsk_id_t,
        parent: tsk_id_t,
        time: f64,
        derived_state: Option<&[u8]>,
    ) -> Result<tsk_id_t, TskitError> {
        self.add_row_with_metadata(site, node, parent, time, derived_state, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        site: tsk_id_t,
        node: tsk_id_t,
        parent: tsk_id_t,
        time: f64,
        derived_state: Option<&[u8]>,
        metadata: &[u8],
    ) -> Result<tsk_id_t, TskitError> {
        unsafe {
            Ok(tsk_mutation_table_add_row(
                self.as_mut(),
                site,
                node,
                parent,
                time,
                match derived_state {
                    Some(d) => d.as_ptr() as *const c_char,
                    None => std::ptr::null(),
                },
                match derived_state {
                    Some(d) => d.len() as u64,
                    None => 0,
                },
                metadata.as_ptr().cast::<c_char>(),
                metadata.len() as u64,
            ))
        }
    }

    pub fn time(&self, row: MutationId) -> Option<Time> {
        safe_tsk_column_access!(self, row, Time, time)
    }

    pub fn site(&self, row: MutationId) -> Option<SiteId> {
        safe_tsk_column_access!(self, row, SiteId, site)
    }

    pub fn node(&self, row: MutationId) -> Option<NodeId> {
        safe_tsk_column_access!(self, row, NodeId, node)
    }

    pub fn parent(&self, row: MutationId) -> Option<MutationId> {
        safe_tsk_column_access!(self, row, MutationId, parent)
    }

    raw_metadata_getter_for_tables!(MutationId);

    fn derived_state_column(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().derived_state.cast::<u8>(),
                self.as_ref().derived_state_length as usize,
            )
        }
    }

    fn derived_state_offset_raw(&self) -> &[tsk_size_t] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().derived_state_offset,
                self.as_ref().num_rows as usize,
            )
        }
    }

    pub fn derived_state(&self, row: MutationId) -> Option<&[u8]> {
        super::tsk_ragged_column_access(
            row,
            self.derived_state_column(),
            self.derived_state_offset_raw(),
        )
    }

    pub fn row<'table>(&self, row: MutationId) -> Option<super::Mutation<'table, Self>> {
        let mut mutation = unsafe {
            std::mem::MaybeUninit::<super::bindings::tsk_mutation_t>::zeroed().assume_init()
        };
        let rv = unsafe {
            super::bindings::tsk_mutation_table_get_row(
                self.as_ref(),
                row.into(),
                &mut mutation as *mut super::bindings::tsk_mutation_t,
            )
        };
        if rv == 0 {
            Some(super::Mutation {
                row: mutation,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = super::Mutation<'_, Self>> {
        MutationTableIter {
            table: self,
            current_row: 0.into(),
        }
    }
}

impl Default for MutationTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
