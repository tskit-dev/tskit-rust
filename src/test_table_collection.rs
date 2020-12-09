#[cfg(test)]
mod tests {
    use crate::*;
    use std::mem::MaybeUninit;

    #[test]
    fn test_add_edge_table_rows() -> () {
        let mut edges: MaybeUninit<tsk_edge_table_t> = MaybeUninit::uninit();
        unsafe {
            let mut rv = tsk_edge_table_init(edges.as_mut_ptr(), 0);
            rv = tsk_edge_table_add_row(edges.as_mut_ptr(), 0., 10., 0, 1, std::ptr::null(), 0);
            assert_eq!((*edges.as_ptr()).num_rows, 1);
            tsk_edge_table_free(edges.as_mut_ptr());
        }
    }
}
