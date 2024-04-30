use std::ptr::NonNull;

use super::bindings::tsk_node_table_add_row;
use super::bindings::tsk_node_table_clear;
use super::bindings::tsk_node_table_init;
use super::bindings::tsk_node_table_t;
use super::tskbox::TskBox;
use super::TskitError;

#[derive(Debug)]
pub struct NodeTable(TskBox<tsk_node_table_t>);

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
                metadata.as_ptr().cast::<i8>(),
                metadata.len() as u64,
            )
        } {
            id if id >= 0 => Ok(id.into()),
            code => Err(TskitError::ErrorCode { code }),
        }
    }
}

impl Default for NodeTable {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}
