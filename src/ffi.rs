//! Define traits related to wrapping tskit stuff

/// Define what it means to wrap a tskit struct.
/// In practice, one needs to implement Drop for
/// test types, calling the tsk_foo_free() function
/// corresponding to tsk_foo_t.
pub trait TskitType<T> {
    /// Encapsulate tsk_foo_t and return rust
    /// object.  Best practices seem to
    /// suggest using Box for this.
    fn wrap() -> Self;
    /// Return const pointer
    fn as_ptr(&self) -> *const T;
    /// Return mutable pointer
    fn as_mut_ptr(&mut self) -> *mut T;
}

/// Define what it means to wrap a tskit struct
/// that contains another tskit type C (the
/// "consumed" type).
/// This trait models a type that takes another
/// type as input for initialization and effectively
/// owns it.
/// A key example of such a type is a [`crate::TreeSequence`],
/// which owns the underying [`crate::TableCollection`].
/// In practice, one needs to implement Drop for
/// test types, calling the tsk_foo_free() function
/// corresponding to tsk_foo_t.
pub trait TskitConsumingType<T, C> {
    /// Encapsulate tsk_foo_t and return rust
    /// object.  Best practices seem to
    /// suggest using Box for this.
    fn wrap(consumed: C) -> Self;
    /// Return const pointer
    fn as_ptr(&self) -> *const T;
    /// Return mutable pointer
    fn as_mut_ptr(&mut self) -> *mut T;
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
}
