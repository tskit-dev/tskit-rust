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
pub(crate) struct WrappedTskArray<T> {
    array: T,
    len_: crate::tsk_size_t,
    idx_: crate::tsk_id_t,
}

impl<T> WrappedTskArray<T> {
    pub fn new(array: T, len: crate::tsk_size_t) -> Self {
        Self {
            array,
            len_: len,
            idx_: -1,
        }
    }

    pub fn len(&self) -> crate::tsk_size_t {
        self.len_
    }
}

pub(crate) type TskIdArray = WrappedTskArray<*const crate::tsk_id_t>;
pub(crate) type Tskf64Array = WrappedTskArray<*const f64>;

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

        let mut a = TskIdArray::new(unsafe { (*t.as_ptr()).edges.child }, 1);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0], 17);

        assert_eq!(a.next(), Some(17));
        assert_eq!(a.next(), None);

        let mut v = vec![];
        for i in &mut a {
            v.push(i);
        }
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], 17);

        v = vec![];
        for i in &mut a {
            v.push(i);
        }
        assert_eq!(v.len(), 1);
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
