use crate::EdgeTable;

pub struct TableViews<'a> {
    pub(crate) edges: EdgeTable<'a>,
}

impl<'a> TableViews<'a> {
    fn edges(&self) -> &EdgeTable<'a> {
        &self.edges
    }
}
