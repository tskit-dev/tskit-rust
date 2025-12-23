use std::ffi::c_char;
use std::ptr::NonNull;

use super::newtypes::EdgeId;
use super::newtypes::NodeId;
use super::newtypes::Position;

use super::bindings::tsk_edge_table_add_row;
use super::bindings::tsk_edge_table_clear;
use super::bindings::tsk_edge_table_init;
use super::bindings::tsk_edge_table_t;
use super::bindings::tsk_id_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct EdgeTable(TskBox<tsk_edge_table_t>);

impl EdgeTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk =
            TskBox::new(|e: *mut tsk_edge_table_t| unsafe { tsk_edge_table_init(e, options) })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_edge_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_edge_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_edge_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_edge_table_clear(self.as_mut()) }
    }

    pub fn add_row(
        &mut self,
        left: f64,
        right: f64,
        parent: tsk_id_t,
        child: tsk_id_t,
    ) -> Result<tsk_id_t, TskitError> {
        self.add_row_with_metadata(left, right, parent, child, &[])
    }

    pub fn add_row_with_metadata(
        &mut self,
        left: f64,
        right: f64,
        parent: tsk_id_t,
        child: tsk_id_t,
        metadata: &[u8],
    ) -> Result<tsk_id_t, TskitError> {
        unsafe {
            Ok(tsk_edge_table_add_row(
                self.as_mut(),
                left,
                right,
                parent,
                child,
                metadata.as_ptr().cast::<c_char>(),
                metadata.len() as u64,
            ))
        }
    }

    pub fn parent(&self, row: EdgeId) -> Option<NodeId> {
        safe_tsk_column_access!(self, row, NodeId, parent)
    }

    pub fn child(&self, row: EdgeId) -> Option<NodeId> {
        safe_tsk_column_access!(self, row, NodeId, child)
    }

    pub fn left(&self, row: EdgeId) -> Option<Position> {
        safe_tsk_column_access!(self, row, Position, left)
    }

    pub fn right(&self, row: EdgeId) -> Option<Position> {
        safe_tsk_column_access!(self, row, Position, right)
    }

    raw_metadata_getter_for_tables!(EdgeId);
}

impl Default for EdgeTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
