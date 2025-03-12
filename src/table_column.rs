macro_rules! make_table_column {
    ($name: ident, $index: ident) => {
        /// Immutable view of a column
        #[derive(Clone, Debug)]
        #[repr(transparent)]
        pub struct $name<'table, T>(&'table [T]);

        impl<'table, T> $name<'table, T> {
            pub(crate) fn new(column: &'table [T]) -> $name<'table, T> {
                Self(column)
            }

            /// View the underlying slice
            pub fn as_slice(&self) -> &[T] {
                self.0
            }
        }

        impl<T> std::ops::Index<usize> for $name<'_, T> {
            type Output = T;
            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<T> std::ops::Index<crate::$index> for $name<'_, T> {
            type Output = T;
            fn index(&self, index: crate::$index) -> &Self::Output {
                &self.0[usize::try_from(index).unwrap()]
            }
        }

        impl<T> std::ops::Index<crate::SizeType> for $name<'_, T> {
            type Output = T;
            fn index(&self, index: crate::SizeType) -> &Self::Output {
                &self.0[usize::try_from(index).unwrap()]
            }
        }

        impl<T> std::convert::AsRef<[T]> for $name<'_, T> {
            fn as_ref(&self) -> &[T] {
                self.0
            }
        }
    };
}

make_table_column!(NodeTableColumn, NodeId);
