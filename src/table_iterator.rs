use crate::tsk_id_t;

pub struct TableIterator<T> {
    pub(crate) table: T,
    pub(crate) pos: tsk_id_t,
    pub(crate) decode_metadata: bool,
}

pub(crate) fn make_table_iterator<TABLE>(
    table: TABLE,
    decode_metadata: bool,
) -> TableIterator<TABLE> {
    TableIterator {
        table,
        pos: 0,
        decode_metadata,
    }
}
