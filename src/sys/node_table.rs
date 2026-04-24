use std::ffi::c_char;
use std::ptr::NonNull;

use super::flags::NodeFlags;
use super::newtypes::IndividualId;
use super::newtypes::PopulationId;
use super::newtypes::Time;

use super::bindings::tsk_node_table_add_row;
use super::bindings::tsk_node_table_clear;
use super::bindings::tsk_node_table_init;
use super::bindings::tsk_node_table_t;
use super::newtypes::NodeId;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct NodeTable(TskBox<tsk_node_table_t>);

pub struct NodeTableIter<'table> {
    table: &'table NodeTable,
    current_row: NodeId,
}

impl<'table> Iterator for NodeTableIter<'table> {
    type Item = super::Node<'table, NodeTable>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.current_row;
        self.current_row += 1;
        self.table.row(c)
    }
}

impl NodeTable {
    pub fn new(options: u32) -> Result<Self, TskitError> {
        let tsk =
            TskBox::new(|e: *mut tsk_node_table_t| unsafe { tsk_node_table_init(e, options) })?;
        Ok(Self(tsk))
    }

    pub unsafe fn new_borrowed(ptr: NonNull<tsk_node_table_t>) -> Self {
        let tsk = TskBox::new_init_from_ptr(ptr);
        Self(tsk)
    }

    pub fn as_ref(&self) -> &tsk_node_table_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut tsk_node_table_t {
        self.0.as_mut()
    }

    pub fn clear(&mut self) -> i32 {
        unsafe { tsk_node_table_clear(self.as_mut()) }
    }

    pub fn add_row<F, T, P, I>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
    ) -> Result<super::newtypes::NodeId, TskitError>
    where
        F: Into<super::flags::NodeFlags>,
        T: Into<super::newtypes::Time>,
        P: Into<super::newtypes::PopulationId>,
        I: Into<super::newtypes::IndividualId>,
    {
        self.add_row_with_metadata(flags, time, population, individual, &[])
    }

    pub fn add_row_with_metadata<F, T, P, I>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
        metadata: &[u8],
    ) -> Result<super::newtypes::NodeId, TskitError>
    where
        F: Into<super::flags::NodeFlags>,
        T: Into<super::newtypes::Time>,
        P: Into<super::newtypes::PopulationId>,
        I: Into<super::newtypes::IndividualId>,
    {
        // SAFETY: pointer is not null
        // If it points to an unititalized object,
        // the error is in an earlier "unsafe" call.
        match unsafe {
            tsk_node_table_add_row(
                self.as_mut(),
                flags.into().bits(),
                time.into().into(),
                population.into().into(),
                individual.into().into(),
                metadata.as_ptr().cast::<c_char>(),
                metadata.len() as u64,
            )
        } {
            id if id >= 0 => Ok(id.into()),
            code => Err(TskitError::ErrorCode { code }),
        }
    }

    pub fn individual(&self, row: NodeId) -> Option<IndividualId> {
        safe_tsk_column_access!(self, row, IndividualId, individual)
    }

    pub fn population(&self, row: NodeId) -> Option<PopulationId> {
        safe_tsk_column_access!(self, row, PopulationId, population)
    }

    pub fn time(&self, row: NodeId) -> Option<Time> {
        safe_tsk_column_access!(self, row, Time, time)
    }

    pub fn flags(&self, row: NodeId) -> Option<NodeFlags> {
        safe_tsk_column_access!(self, row, NodeFlags, flags)
    }

    raw_metadata_getter_for_tables!(NodeId);

    pub fn row<'table>(&self, row: NodeId) -> Option<super::Node<'table, Self>> {
        let mut node =
            unsafe { std::mem::MaybeUninit::<super::bindings::tsk_node_t>::zeroed().assume_init() };
        let rv = unsafe {
            super::bindings::tsk_node_table_get_row(
                self.as_ref(),
                row.into(),
                &mut node as *mut super::bindings::tsk_node_t,
            )
        };
        if rv == 0 {
            Some(super::Node {
                row: node,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = super::Node<'_, Self>> {
        NodeTableIter {
            table: self,
            current_row: 0.into(),
        }
    }
}

impl Default for NodeTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
