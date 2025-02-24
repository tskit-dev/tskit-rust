/// For a type `tsk_foo_t`, this trait abstracts
/// out the functionality of `tsk_foo_free`
///
/// # Note
///
/// This trait should NEVER be part of the public API.
pub trait TskTeardown {
    /// # Safety
    ///
    /// Implementations must abide by the expectations
    /// of `tsk_foo_free` and C's `free`.
    unsafe fn teardown(&mut self) -> i32;
}
