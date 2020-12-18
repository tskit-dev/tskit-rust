use crate::bindings as ll_bindings;
use crate::tsk_size_t;

/// An immutable view of site table.
///
/// These are not created directly.
/// Instead, use [`TableCollection::populations`](crate::TableCollection::populations)
/// to get a reference to an existing population table;
pub struct PopulationTable<'a> {
    table_: &'a ll_bindings::tsk_population_table_t,
}

impl<'a> PopulationTable<'a> {
    pub(crate) fn new_from_table(mutations: &'a ll_bindings::tsk_population_table_t) -> Self {
        PopulationTable { table_: mutations }
    }

    /// Return the number of rows.
    pub fn num_rows(&'a self) -> tsk_size_t {
        self.table_.num_rows
    }
}
