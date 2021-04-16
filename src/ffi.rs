//! Define traits related to wrapping tskit stuff

/// Provide pointer access to underlying C types
pub trait TskitTypeAccess<T> {
    /// Return const pointer
    fn as_ptr(&self) -> *const T;
    /// Return mutable pointer
    fn as_mut_ptr(&mut self) -> *mut T;
}

/// Indexable, iterable wrapper around C
/// arrays.
#[derive(Copy, Clone)]
pub struct WrappedTskArray<T> {
    array: *const T,
    len_: crate::tsk_size_t,
}

pub struct WrappedTskArrayIter<'a, T: Copy + 'a> {
    inner: &'a WrappedTskArray<T>,
    pos: crate::tsk_size_t,
}

impl<'a, T: Copy> Iterator for WrappedTskArrayIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.len_ {
            None
        } else {
            let rv = Some(unsafe { *self.inner.array.offset(self.pos as isize) as T });
            self.pos += 1;
            rv
        }
    }
}

impl<T: Copy> WrappedTskArray<T> {
    pub(crate) fn new(array: *const T, len: crate::tsk_size_t) -> Self {
        Self { array, len_: len }
    }

    pub fn len(&self) -> crate::tsk_size_t {
        self.len_
    }

    pub fn is_empty(&self) -> bool {
        self.len_ == 0
    }

    /// # Safety
    ///
    /// This function returns the raw C pointer,
    /// and is thus unsafe.
    pub unsafe fn as_ptr(&self) -> *const T {
        self.array
    }

    pub fn iter(&self) -> WrappedTskArrayIter<T> {
        WrappedTskArrayIter {
            inner: self,
            pos: 0,
        }
    }
}

pub(crate) type TskIdArray = WrappedTskArray<crate::tsk_id_t>;
pub(crate) type Tskf64Array = WrappedTskArray<f64>;

wrapped_tsk_array_traits!(TskIdArray, crate::tsk_id_t, crate::tsk_id_t);
wrapped_tsk_array_traits!(Tskf64Array, crate::tsk_id_t, f64);

/// Wrap a tskit type
pub(crate) trait WrapTskitType<T> {
    /// Encapsulate tsk_foo_t and return rust
    /// object.  Best practices seem to
    /// suggest using Box for this.
    fn wrap() -> Self;
}

/// Wrap a tskit type that consumes another
/// tskit type.  The tree sequence is an example.
pub(crate) trait WrapTskitConsumingType<T, C> {
    /// Encapsulate tsk_foo_t and return rust
    /// object.  Best practices seem to
    /// suggest using Box for this.
    fn wrap(consumed: C) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings as ll_bindings;
    use crate::tsk_size_t;
    use ll_bindings::tsk_table_collection_free;

    pub struct TableCollectionMock {
        inner: Box<ll_bindings::tsk_table_collection_t>,
    }

    build_tskit_type!(
        TableCollectionMock,
        ll_bindings::tsk_table_collection_t,
        tsk_table_collection_free
    );

    impl TableCollectionMock {
        fn new(len: f64) -> Self {
            let mut s = Self::wrap();

            let rv = unsafe { ll_bindings::tsk_table_collection_init(s.as_mut_ptr(), 0) };
            assert_eq!(rv, 0);

            s.inner.sequence_length = len;

            s
        }

        fn sequence_length(&self) -> f64 {
            unsafe { (*self.as_ptr()).sequence_length }
        }
    }

    #[test]
    fn test_create_mock_type() {
        let t = TableCollectionMock::new(10.);
        assert_eq!(t.sequence_length() as i64, 10);
    }

    #[test]
    fn test_u32_array_wrapper() {
        let mut t = TableCollectionMock::new(10.);

        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*t.as_mut_ptr()).edges,
                0.,
                10.,
                0,
                17,
                std::ptr::null(),
                0,
            )
        };
        panic_on_tskit_error!(rv);

        let a = TskIdArray::new(unsafe { (*t.as_ptr()).edges.child }, 1);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0], 17);

        let mut v = vec![];
        for i in a.iter() {
            v.push(i);
        }
        assert_eq!(v.len() as tsk_size_t, a.len());
        assert_eq!(v[0], 17);
    }

    #[should_panic]
    #[test]
    fn test_u32_array_wrapper_panic() {
        let mut t = TableCollectionMock::new(10.);

        let rv = unsafe {
            ll_bindings::tsk_edge_table_add_row(
                &mut (*t.as_mut_ptr()).edges,
                0.,
                10.,
                0,
                17,
                std::ptr::null(),
                0,
            )
        };
        panic_on_tskit_error!(rv);

        let a = TskIdArray::new(unsafe { (*t.as_ptr()).edges.child }, 1);
        assert_eq!(a.len(), 1);
        assert_eq!(a[1], 17);
    }
}
