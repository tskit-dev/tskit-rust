#[cfg(test)]
mod tests {
    use crate::test_fixtures::make_small_table_collection_two_trees;
    use crate::test_fixtures::treeseq_from_small_table_collection_two_trees;
    use crate::tsk_id_t;
    use crate::SimplificationOptions;
    use crate::TableAccess;
    use crate::TSK_NODE_IS_SAMPLE;
    use crate::TSK_NULL;

    #[test]
    fn test_simplify_tables() {
        let mut tables = make_small_table_collection_two_trees();
        let mut samples: Vec<tsk_id_t> = vec![];
        for (i, row) in tables.nodes_iter(false).enumerate() {
            if row.flags & TSK_NODE_IS_SAMPLE > 0 {
                samples.push(i as tsk_id_t);
            }
        }
        let idmap_option = tables
            .simplify(&samples, SimplificationOptions::default(), true)
            .unwrap();
        assert!(idmap_option.is_some());
        let idmap = idmap_option.unwrap();
        for i in samples.iter() {
            assert_ne!(idmap[*i as usize], TSK_NULL);
        }
    }

    #[test]
    fn test_simplify_treeseq() {
        let ts = treeseq_from_small_table_collection_two_trees();
        let samples = ts.sample_nodes();
        let (_, idmap_option) = ts
            .simplify(&samples, SimplificationOptions::default(), true)
            .unwrap();
        assert!(idmap_option.is_some());
        let idmap = idmap_option.unwrap();
        for &i in samples {
            assert_ne!(idmap[i as usize], TSK_NULL);
        }
    }
}
