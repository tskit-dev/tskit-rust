//! Low-level ("unsafe") bindings to the C API.
//!
//! This module is a 1-to-1 mapping of C types
//! and functions for both tskit and kastore.
//! The bindings are generate via [bindgen](https://docs.rs/bindgen).
//!
//! Using things from this module will be ``unsafe``.
//! Further, as many of the types require ``init()`` methods
//! to correctly set up the structs, one has to coerce ``rust``
//! into allowing uninitialized variables:
//!
//! ```
//! # #[cfg(feature="bindings")]
//! # {
//! use std::mem::MaybeUninit;
//! let mut edges: MaybeUninit<tskit::bindings::tsk_edge_table_t> = MaybeUninit::uninit();
//! unsafe {
//!     let _ = tskit::bindings::tsk_edge_table_init(edges.as_mut_ptr(), 0);
//!     let _ = tskit::bindings::tsk_edge_table_add_row(edges.as_mut_ptr(), 0., 10., 0, 1, std::ptr::null(), 0);
//!     assert_eq!((*edges.as_ptr()).num_rows, 1);
//!     tskit::bindings::tsk_edge_table_free(edges.as_mut_ptr());
//! }
//! # }
//! ```
//!
//! The best source for documentation will be the [tskit docs](https://tskit.readthedocs.io).
//! Those docs describe the most important parts of the C API.
//! This module contains the same types/functions with the same names.

#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/auto_bindings.rs"));
