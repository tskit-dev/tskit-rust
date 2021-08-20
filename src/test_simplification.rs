#[cfg(test)]
mod tests {
    use crate::test_fixtures::make_small_table_collection_two_trees;
    use crate::test_fixtures::treeseq_from_small_table_collection_two_trees;
    use crate::NodeId;
    use crate::SimplificationOptions;
    use crate::TableAccess;
    use crate::TSK_NODE_IS_SAMPLE;

    #[test]
    fn test_simplify_tables() {
        let mut tables = make_small_table_collection_two_trees();
        let mut samples: Vec<NodeId> = vec![];
        for (i, row) in tables.nodes_iter().enumerate() {
            if row.flags & TSK_NODE_IS_SAMPLE > 0 {
                samples.push((i as i32).into());
            }
        }
        let idmap_option = tables
            .simplify(&samples, SimplificationOptions::default(), true)
            .unwrap();
        assert!(idmap_option.is_some());
        let idmap = idmap_option.unwrap();
        for i in samples.iter() {
            assert_ne!(idmap[i.0 as usize], NodeId::NULL);
            assert!(!idmap[i.0 as usize].is_null());
        }
    }

    #[test]
    fn test_simplify_treeseq() {
        let ts = treeseq_from_small_table_collection_two_trees();
        let samples = ts.sample_nodes();
        let (_, idmap_option) = ts
            .simplify(samples, SimplificationOptions::default(), true)
            .unwrap();
        assert!(idmap_option.is_some());
        let idmap = idmap_option.unwrap();
        for &i in samples {
            assert_ne!(idmap[usize::from(i)], NodeId::NULL);
        }
    }
}
