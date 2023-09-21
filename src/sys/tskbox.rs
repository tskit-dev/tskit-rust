use std::ptr::NonNull;

#[derive(Debug)]
pub struct TskBox<T> {
    tsk: NonNull<T>,
    teardown: Option<unsafe extern "C" fn(*mut T) -> i32>,
    owning: bool,
}

// SAFETY: these must be encapsulated in types that work
// via shared/immutable reference AND/OR use data protection methods.
unsafe impl<T> Send for TskBox<T> {}

// SAFETY: these must be encapsulated in types that work
// via shared/immutable reference AND/OR use data protection methods.
unsafe impl<T> Sync for TskBox<T> {}

impl<T> TskBox<T> {
    pub fn new<F: Fn(*mut T) -> i32>(
        init: F,
        teardown: unsafe extern "C" fn(*mut T) -> i32,
    ) -> Self {
        // SAFETY: we will initialize it next
        let mut uninit = unsafe { Self::new_uninit(teardown) };
        let _ = init(uninit.as_mut());
        uninit
    }

    // # Safety
    //
    // The returned value is uninitialized.
    // Using the object prior to initilization is likely to trigger UB.
    pub unsafe fn new_uninit(teardown: unsafe extern "C" fn(*mut T) -> i32) -> Self {
        let x = unsafe { libc::malloc(std::mem::size_of::<T>()) as *mut T };
        let tsk = NonNull::new(x).unwrap();
        Self {
            tsk,
            teardown: Some(teardown),
            owning: true,
        }
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
        Self {
            tsk,
            teardown: None,
            owning: false,
        }
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
        s.teardown = None;
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

impl<T> Drop for TskBox<T> {
    fn drop(&mut self) {
        if let Some(teardown) = self.teardown {
            unsafe {
                (teardown)(self.tsk.as_mut() as *mut T);
            }
        }
        if self.owning {
            unsafe { libc::free(self.tsk.as_ptr() as *mut libc::c_void) }
        }
    }
}

#[cfg(test)]
fn is_send_sync<T: Send + Sync>(_: &T) {}

// NOTE: tests must not make calls into the tskit C API!
// We need to use miri to check for UB, and miri cannot
// work accross FFI.

#[test]
fn test_miri() {
    struct X {
        data: i32,
    }

    unsafe extern "C" fn teardown_x(_: *mut X) -> i32 {
        0
    }

    let options = 0_i32;

    let b = TskBox::new(
        |x: *mut X| unsafe {
            (*x).data = options;
            0
        },
        teardown_x,
    );

    let _ = unsafe { TskBox::new_borrowed(&b) };

    is_send_sync(&b)
}

#[test]
fn test_into_raw_miri() {
    struct X {
        data: i32,
    }

    unsafe extern "C" fn teardown_x(_: *mut X) -> i32 {
        0
    }

    let options = 0_i32;

    let b = TskBox::new(
        |x: *mut X| unsafe {
            (*x).data = options;
            0
        },
        teardown_x,
    );

    let p = unsafe { b.into_raw() };

    unsafe { libc::free(p as *mut libc::c_void) }
}

#[test]
fn test_table_collection_tskbox() {
    let flags: u32 = 0;
    let _ = TskBox::new(
        |t: *mut super::bindings::tsk_table_collection_t| unsafe {
            super::bindings::tsk_table_collection_init(t, flags)
        },
        super::bindings::tsk_table_collection_free,
    );
}

#[test]
fn test_table_collection_tskbox_shared_ptr() {
    let flags: u32 = 0;
    let tables = TskBox::new(
        |t: *mut super::bindings::tsk_table_collection_t| unsafe {
            super::bindings::tsk_table_collection_init(t, flags)
        },
        super::bindings::tsk_table_collection_free,
    );
    let _ = unsafe { TskBox::new_borrowed(&tables) };
}
