use crate::edge_table::EdgeTable2;

pub struct TableViews {
    pub(crate) edges: EdgeTable2,
}

impl TableViews {
    fn edges(&self) -> &EdgeTable2 {
        &self.edges
    }
}
