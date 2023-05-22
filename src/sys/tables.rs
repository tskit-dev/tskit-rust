use std::ptr::NonNull;

use mbox::MBox;

use super::Error;

macro_rules! basic_lltableref_impl {
    ($lltable: ident, $tsktable: ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $lltable(NonNull<super::bindings::$tsktable>);

        impl $lltable {
            pub fn new_from_table(table: *mut super::bindings::$tsktable) -> Result<Self, Error> {
                let internal = NonNull::new(table).ok_or_else(|| {
                    let msg = format!("null pointer to {}", stringify!($tsktable));
                    Error::Message(msg)
                })?;
                Ok(Self(internal))
            }

            pub fn as_ref(&self) -> &super::bindings::$tsktable {
                // SAFETY: we cannot get this far w/o
                // going through new_from_table and that
                // fn protects us from null ptrs
                unsafe { self.0.as_ref() }
            }
        }
    };
}

basic_lltableref_impl!(LLEdgeTableRef, tsk_edge_table_t);
basic_lltableref_impl!(LLNodeTableRef, tsk_node_table_t);
basic_lltableref_impl!(LLMutationTableRef, tsk_mutation_table_t);
basic_lltableref_impl!(LLSiteTableRef, tsk_site_table_t);
basic_lltableref_impl!(LLMigrationTableRef, tsk_migration_table_t);
basic_lltableref_impl!(LLPopulationTableRef, tsk_population_table_t);
basic_lltableref_impl!(LLIndividualTableRef, tsk_individual_table_t);

#[cfg(feature = "provenance")]
basic_lltableref_impl!(LLProvenanceTableRef, tsk_provenance_table_t);

macro_rules! basic_llowningtable_impl {
    ($llowningtable: ident, $tsktable: ident, $init: ident, $free: ident, $clear: ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $llowningtable(MBox<super::bindings::$tsktable>);

        impl $llowningtable {
            pub fn new() -> Self {
                let temp = unsafe {
                    libc::malloc(std::mem::size_of::<super::bindings::$tsktable>())
                        as *mut super::bindings::$tsktable
                };
                let nonnull = match std::ptr::NonNull::<super::bindings::$tsktable>::new(temp) {
                    Some(x) => x,
                    None => panic!("out of memory"),
                };
                let mut table = unsafe { mbox::MBox::from_non_null_raw(nonnull) };
                let rv = unsafe { super::bindings::$init(&mut (*table), 0) };
                assert_eq!(rv, 0);
                Self(table)
            }

            pub fn as_ptr(&self) -> *const super::bindings::$tsktable {
                MBox::<super::bindings::$tsktable>::as_ptr(&self.0)
            }

            pub fn as_mut_ptr(&mut self) -> *mut super::bindings::$tsktable {
                MBox::<super::bindings::$tsktable>::as_mut_ptr(&mut self.0)
            }

            fn free(&mut self) -> Result<(), Error> {
                match unsafe { super::bindings::$free(self.as_mut_ptr()) } {
                    code if code < 0 => Err(Error::Code(code)),
                    _ => Ok(()),
                }
            }

            pub fn clear(&mut self) -> Result<i32, Error> {
                match unsafe { super::bindings::$clear(self.as_mut_ptr()) } {
                    code if code < 0 => Err(Error::Code(code)),
                    code => Ok(code),
                }
            }
        }

        impl Drop for $llowningtable {
            fn drop(&mut self) {
                match self.free() {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
        }
    };
}

basic_llowningtable_impl!(
    LLOwningEdgeTable,
    tsk_edge_table_t,
    tsk_edge_table_init,
    tsk_edge_table_free,
    tsk_edge_table_clear
);
basic_llowningtable_impl!(
    LLOwningNodeTable,
    tsk_node_table_t,
    tsk_node_table_init,
    tsk_node_table_free,
    tsk_node_table_clear
);
basic_llowningtable_impl!(
    LLOwningSiteTable,
    tsk_site_table_t,
    tsk_site_table_init,
    tsk_site_table_free,
    tsk_site_table_clear
);
basic_llowningtable_impl!(
    LLOwningMutationTable,
    tsk_mutation_table_t,
    tsk_mutation_table_init,
    tsk_mutation_table_free,
    tsk_mutation_table_clear
);
basic_llowningtable_impl!(
    LLOwningIndividualTable,
    tsk_individual_table_t,
    tsk_individual_table_init,
    tsk_individual_table_free,
    tsk_individual_table_clear
);
basic_llowningtable_impl!(
    LLOwningMigrationTable,
    tsk_migration_table_t,
    tsk_migration_table_init,
    tsk_migration_table_free,
    tsk_migration_table_clear
);
basic_llowningtable_impl!(
    LLOwningPopulationTable,
    tsk_population_table_t,
    tsk_population_table_init,
    tsk_population_table_free,
    tsk_population_table_clear
);

#[cfg(feature = "provenance")]
basic_llowningtable_impl!(
    LLOwningProvenanceTable,
    tsk_provenance_table_t,
    tsk_provenance_table_init,
    tsk_provenance_table_free,
    tsk_provenance_table_clear
);
