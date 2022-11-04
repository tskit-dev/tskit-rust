use std::ptr::NonNull;

use crate::edge_table::EdgeTable2;

///```compile_fail
///let mut tables = tskit::TableCollection::new(100.0).unwrap();
///tables.add_edge(1., 2., 0, 1).unwrap();
///let edge_view = tables.edges();
///drop(tables);
///assert_eq!(tables.edges().num_rows(), 1);
///```
///
///```compile_fail
///let mut tables = tskit::TableCollection::new(100.0).unwrap();
///tables.add_edge(1., 2., 0, 1).unwrap();
/////get the iterator
///let mut e = tables.edges().iter();
///// drop our tables, breaking the liftime association
///drop(tables);
///// fail!
///while let(Some(x)) = e.next() {}
///```
pub struct TableViews {
    pub(crate) edges: EdgeTable2,
}

impl TableViews {
    pub(crate) fn new_from_tables(tables: *mut crate::bindings::tsk_table_collection_t) -> Self {
        let edges = crate::edge_table::EdgeTable2 {
            table_: NonNull::new(&mut unsafe { (*tables).edges }).unwrap(),
        };
        Self { edges }
    }

    pub fn edges(&self) -> &EdgeTable2 {
        &self.edges
    }
}

#[test]
fn test_views() {
    let mut tables = crate::TableCollection::new(100.0).unwrap();
    tables.add_edge(1., 2., 0, 1).unwrap();
    assert_eq!(tables.edges().num_rows(), 1);
    assert_eq!(tables.edges().iter().count(), 1);
}
