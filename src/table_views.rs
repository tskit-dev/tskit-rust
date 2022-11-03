use crate::edge_table::EdgeTable2;

///```compile_fail
///let mut tables = crate::TableCollection::new(100.0).unwrap();
///tables.add_edge(1., 2., 0, 1).unwrap();
///let edge_view = tables.edges();
///drop(tables);
///assert_eq!(tables.edges().num_rows(), 1);
///```
pub struct TableViews {
    pub(crate) edges: EdgeTable2,
}

impl TableViews {
    fn edges(&self) -> &EdgeTable2 {
        &self.edges
    }
}

#[test]
fn test_views() {
    let mut tables = crate::TableCollection::new(100.0).unwrap();
    tables.add_edge(1., 2., 0, 1).unwrap();
    assert_eq!(tables.edges().num_rows(), 1);
}
