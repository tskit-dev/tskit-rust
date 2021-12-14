//! Define traits related to wrapping tskit stuff

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
    use crate::TskitTypeAccess;
    use ll_bindings::tsk_table_collection_free;
    use mbox::MBox;

    pub struct TableCollectionMock {
        inner: MBox<ll_bindings::tsk_table_collection_t>,
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

            (*s.inner).sequence_length = len;

            s
        }

        fn sequence_length(&self) -> f64 {
            (*self.inner).sequence_length
        }
    }

    #[test]
    fn test_create_mock_type() {
        let t = TableCollectionMock::new(10.);
        assert_eq!(t.sequence_length() as i64, 10);
    }
}
