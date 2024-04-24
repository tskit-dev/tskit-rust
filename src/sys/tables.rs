use std::ptr::NonNull;

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

basic_lltableref_impl!(LLPopulationTableRef, tsk_population_table_t);

#[cfg(feature = "provenance")]
basic_lltableref_impl!(LLProvenanceTableRef, tsk_provenance_table_t);

macro_rules! basic_llowningtable_impl {
    ($llowningtable: ident, $tsktable: ident, $init: ident, $clear: ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $llowningtable(super::tskbox::TskBox<super::bindings::$tsktable>);

        impl $llowningtable {
            pub fn new() -> Self {
                let table =
                    super::tskbox::TskBox::new(|x: *mut super::bindings::$tsktable| unsafe {
                        super::bindings::$init(x, 0)
                    })
                    .unwrap();
                Self(table)
            }

            pub fn as_ptr(&self) -> *const super::bindings::$tsktable {
                self.0.as_ptr()
            }

            pub fn as_mut_ptr(&mut self) -> *mut super::bindings::$tsktable {
                self.0.as_mut_ptr()
            }

            pub fn clear(&mut self) -> Result<i32, Error> {
                match unsafe { super::bindings::$clear(self.as_mut_ptr()) } {
                    code if code < 0 => Err(Error::Code(code)),
                    code => Ok(code),
                }
            }
        }
    };
}

basic_llowningtable_impl!(
    LLOwningPopulationTable,
    tsk_population_table_t,
    tsk_population_table_init,
    tsk_population_table_clear
);

#[cfg(feature = "provenance")]
basic_llowningtable_impl!(
    LLOwningProvenanceTable,
    tsk_provenance_table_t,
    tsk_provenance_table_init,
    tsk_provenance_table_clear
);
