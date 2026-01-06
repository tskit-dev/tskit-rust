use std::ffi::c_char;
use std::ptr::NonNull;

use super::newtypes::MigrationId;
use super::newtypes::NodeId;
use super::newtypes::PopulationId;
use super::newtypes::Position;
use super::newtypes::Time;

use super::bindings::tsk_id_t;
use super::bindings::tsk_migration_table_add_row;
use super::bindings::tsk_migration_table_clear;
use super::bindings::tsk_migration_table_init;
use super::bindings::tsk_migration_table_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct MigrationTable(TskBox<tsk_migration_table_t>);

impl MigrationTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk = TskBox::new(|e: *mut tsk_migration_table_t| unsafe {
            tsk_migration_table_init(e, options)
        })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_migration_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_migration_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_migration_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_migration_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        span: (f64, f64),
        node: tsk_id_t,
        source: tsk_id_t,
        dest: tsk_id_t,
        time: f64,
    ) -> Result<tsk_id_t, TskitError> {
        self.add_row_with_metadata(span, node, source, dest, time, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        span: (f64, f64),
        node: tsk_id_t,
        source: tsk_id_t,
        dest: tsk_id_t,
        time: f64,
        metadata: &[u8],
    ) -> Result<tsk_id_t, TskitError> {
        unsafe {
            Ok(tsk_migration_table_add_row(
                self.as_mut(),
                span.0,
                span.1,
                node,
                source,
                dest,
                time,
                metadata.as_ptr().cast::<c_char>(),
                metadata.len() as u64,
            ))
        }
    }

    pub fn node(&self, row: MigrationId) -> Option<NodeId> {
        safe_tsk_column_access!(self, row, NodeId, node)
    }

    pub fn source(&self, row: MigrationId) -> Option<PopulationId> {
        safe_tsk_column_access!(self, row, PopulationId, source)
    }

    pub fn dest(&self, row: MigrationId) -> Option<PopulationId> {
        safe_tsk_column_access!(self, row, PopulationId, dest)
    }

    pub fn time(&self, row: MigrationId) -> Option<Time> {
        safe_tsk_column_access!(self, row, Time, time)
    }

    pub fn left(&self, row: MigrationId) -> Option<Position> {
        safe_tsk_column_access!(self, row, Position, left)
    }

    pub fn right(&self, row: MigrationId) -> Option<Position> {
        safe_tsk_column_access!(self, row, Position, right)
    }

    raw_metadata_getter_for_tables!(MigrationId);
}

impl Default for MigrationTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
