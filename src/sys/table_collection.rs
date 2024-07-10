use super::bindings::tsk_edge_table_t;
use super::bindings::tsk_individual_table_t;
use super::bindings::tsk_migration_table_t;
use super::bindings::tsk_mutation_table_t;
use super::bindings::tsk_node_table_t;
use super::bindings::tsk_population_table_t;
#[cfg(feature = "provenance")]
use super::bindings::tsk_provenance_table_t;
use super::bindings::tsk_site_table_t;
use super::bindings::tsk_table_collection_init;
use super::bindings::tsk_table_collection_t;
use super::tskbox::TskBox;
use super::TskitError;

pub struct TableCollection(TskBox<tsk_table_collection_t>);

impl TableCollection {
    pub fn new(sequence_length: f64) -> Result<Self, TskitError> {
        let mut tsk = TskBox::new(|tc: *mut tsk_table_collection_t| unsafe {
            tsk_table_collection_init(tc, 0)
        })?;
        tsk.as_mut().sequence_length = sequence_length;
        Ok(Self(tsk))
    }

    // # Safety
    //
    // `tables` must be an initialized `tsk_table_collection_t`
    pub unsafe fn new_owning_from_nonnull(
        tables: std::ptr::NonNull<tsk_table_collection_t>,
    ) -> Self {
        Self(TskBox::new_init_owning_from_ptr(tables))
    }

    pub unsafe fn new_borrowed(tables: std::ptr::NonNull<tsk_table_collection_t>) -> Self {
        Self(TskBox::new_init_from_ptr(tables))
    }

    // # Safety
    //
    // The returned value is uninitialized.
    // Using the object prior to initilization is likely to trigger UB.
    pub unsafe fn new_uninit() -> Self {
        let tsk = unsafe { TskBox::new_uninit() };
        Self(tsk)
    }

    pub fn copy(&self) -> (i32, TableCollection) {
        // SAFETY: the C API requires that the destiniation of a copy be uninitalized.
        // Copying into it will initialize the object.
        let mut dest = unsafe { Self::new_uninit() };
        // SAFETY: self.as_ptr() is not null and dest matches the input
        // expectations of the C API.
        let rv = unsafe {
            super::bindings::tsk_table_collection_copy(self.as_ptr(), dest.as_mut_ptr(), 0)
        };
        (rv, dest)
    }

    pub fn sequence_length(&self) -> f64 {
        self.0.as_ref().sequence_length
    }

    pub fn as_ptr(&self) -> *const tsk_table_collection_t {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut tsk_table_collection_t {
        self.0.as_mut_ptr()
    }

    pub fn individuals_mut(&mut self) -> &mut tsk_individual_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).individuals }
    }

    pub fn nodes_mut(&mut self) -> &mut tsk_node_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).nodes }
    }

    pub fn edges_mut(&mut self) -> &mut tsk_edge_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).edges }
    }

    pub fn migrations_mut(&mut self) -> &mut tsk_migration_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).migrations }
    }

    pub fn mutations_mut(&mut self) -> &mut tsk_mutation_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).mutations }
    }

    pub fn populations_mut(&mut self) -> &mut tsk_population_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).populations }
    }

    #[cfg(feature = "provenance")]
    pub fn provenances_mut(&mut self) -> &mut tsk_provenance_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).provenances }
    }

    pub fn sites_mut(&mut self) -> &mut tsk_site_table_t {
        // SAFETY: self pointer is not null
        unsafe { &mut (*self.as_mut_ptr()).sites }
    }

    pub fn into_raw(self) -> *mut tsk_table_collection_t {
        self.0.into_raw()
    }
}
