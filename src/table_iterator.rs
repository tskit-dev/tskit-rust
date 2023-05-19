pub struct TableIterator<T> {
    pub(crate) table: T,
    pub(crate) pos: crate::sys::bindings::tsk_id_t,
}

pub(crate) fn make_table_iterator<TABLE>(table: TABLE) -> TableIterator<TABLE> {
    TableIterator { table, pos: 0 }
}
