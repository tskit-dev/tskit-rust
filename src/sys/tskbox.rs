use std::ptr::NonNull;

use super::Error;
use super::TskTeardown;

#[derive(Debug)]
pub struct TskBox<T: TskTeardown> {
    tsk: NonNull<T>,
    owning: bool,
}

// SAFETY: these must be encapsulated in types that work
// via shared/immutable reference AND/OR use data protection methods.
unsafe impl<T> Send for TskBox<T> where T: TskTeardown {}

// SAFETY: these must be encapsulated in types that work
// via shared/immutable reference AND/OR use data protection methods.
unsafe impl<T> Sync for TskBox<T> where T: TskTeardown {}

impl<T> TskBox<T>
where
    T: TskTeardown,
{
    pub fn new<F: Fn(*mut T) -> i32>(init: F) -> Result<Self, Error> {
        // SAFETY: we will initialize it next
        let mut uninit = unsafe { Self::new_uninit() };
        let rv = init(uninit.as_mut());
        if rv < 0 {
            Err(Error::Code(rv))
        } else {
            Ok(uninit)
        }
    }

    // # Safety
    //
    // The returned value is uninitialized.
    // Using the object prior to initilization is likely to trigger UB.
    //
    // UB will occur if the object is dropped before the pointer is initialized.
    // One way to avoid that is to call [`libc::memset`].
    pub unsafe fn new_uninit() -> Self {
        let x = unsafe { libc::malloc(std::mem::size_of::<T>()) as *mut T };
        let tsk = NonNull::new(x).unwrap();
        Self { tsk, owning: true }
    }

    /// # Safety
    ///
    /// This function clones the NonNull of `owner` and will
    /// not perform any teardown when dropped.
    ///
    /// Cloning the pointer elides the tied lifetimes of owner
    /// and the new instance.
    ///
    /// Therefore, the only sound use of this type involves
    /// encapsulation in such a way that its lifetime is bound
    /// to the owner.
    ///
    /// For example, instances should only be publicly exposed
    /// via reference types.
    #[allow(dead_code)]
    pub unsafe fn new_borrowed(owner: &Self) -> Self {
        let tsk = owner.tsk;
        Self { tsk, owning: false }
    }

    /// # Safety
    ///
    /// The returned pointer is no longer managed, meaing
    /// that it is the caller's responsibility to properly
    /// tear down and free the return value.
    ///
    /// Failing to do so will may result in a
    /// memory leak.
    #[allow(dead_code)]
    pub unsafe fn into_raw(self) -> *mut T {
        let mut s = self;
        let rv = s.as_mut_ptr();
        s.owning = false;
        rv
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.tsk.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.tsk.as_mut() }
    }

    pub fn as_ptr(&self) -> *const T {
        self.as_ref()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.as_mut()
    }
}

impl<T> Drop for TskBox<T>
where
    T: TskTeardown,
{
    fn drop(&mut self) {
        if self.owning {
            unsafe {
                // SAFETY: The internal storage type is NonNull.
                // Whe new_uninit is used, the crate is sure to
                // initialize objects.
                self.as_mut().teardown();
                libc::free(self.tsk.as_ptr() as *mut libc::c_void)
            }
        }
    }
}

#[cfg(test)]
fn is_send_sync<T: Send + Sync>(_: &T) {}

#[cfg(test)]
struct X {
    data: i32,
}

#[cfg(test)]
unsafe extern "C" fn teardown_x(_: *mut X) -> i32 {
    0
}

#[cfg(test)]
impl super::TskTeardown for X {
    unsafe fn teardown(&mut self) -> i32 {
        teardown_x(self as _)
    }
}

// NOTE: tests must not make calls into the tskit C API!
// We need to use miri to check for UB, and miri cannot
// work accross FFI.

#[test]
fn test_miri() {
    let options = 0_i32;

    let b = TskBox::new(|x: *mut X| unsafe {
        (*x).data = options;
        0
    })
    .unwrap();

    let _ = unsafe { TskBox::new_borrowed(&b) };

    is_send_sync(&b)
}

#[test]
fn test_miri_uninit() {
    let _ = unsafe { TskBox::<X>::new_uninit() };
}

#[test]
fn test_into_raw_miri() {
    let options = 0_i32;

    let b = TskBox::new(|x: *mut X| unsafe {
        (*x).data = options;
        0
    })
    .unwrap();

    let p = unsafe { b.into_raw() };

    unsafe { libc::free(p as *mut libc::c_void) }
}

#[test]
fn test_table_collection_tskbox_uninit() {
    let mut tables = unsafe { TskBox::<super::bindings::tsk_table_collection_t>::new_uninit() };
    unsafe {
        libc::memset(
            tables.as_mut_ptr() as _,
            0,
            std::mem::size_of::<super::bindings::tsk_table_collection_t>(),
        )
    };
}

#[test]
fn test_table_collection_tskbox() {
    let flags: u32 = 0;
    let _ = TskBox::new(|t: *mut super::bindings::tsk_table_collection_t| unsafe {
        super::bindings::tsk_table_collection_init(t, flags)
    });
}

#[test]
fn test_table_collection_tskbox_shared_ptr() {
    let flags: u32 = 0;
    let tables = TskBox::new(|t: *mut super::bindings::tsk_table_collection_t| unsafe {
        super::bindings::tsk_table_collection_init(t, flags)
    })
    .unwrap();
    let _ = unsafe { TskBox::new_borrowed(&tables) };
}
