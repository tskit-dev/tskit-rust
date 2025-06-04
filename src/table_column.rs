#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct OpaqueTableColumn<'table, T>(pub(crate) &'table [T]);

impl<T> std::ops::Index<usize> for OpaqueTableColumn<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> std::ops::Index<crate::SizeType> for OpaqueTableColumn<'_, T> {
    type Output = T;

    fn index(&self, index: crate::SizeType) -> &Self::Output {
        &self.0[usize::try_from(index).unwrap()]
    }
}
