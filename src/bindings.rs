// re-export the auto-generate bindings
pub use crate::auto_bindings::*;

// tskit defines this via a type cast
// in a macro. bindgen thus misses it.
// See bindgen issue 316.
pub const TSK_NULL: tsk_id_t = -1;

