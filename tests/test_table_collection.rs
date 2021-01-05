use tskit::metadata::*;
use tskit::*;

mod test_adding_table_rows {
    use super::*;

    #[test]
    fn test_add_edges() {
        let mut tables = TableCollection::new(1000.).unwrap();
        for i in 0..5 {
            let _ = tables.add_edge(0., 1000., i, 2 * i).unwrap();
        }
        let edges = tables.edges();
        for i in 0..5 {
            assert_eq!(edges.parent(i).unwrap(), i);
            assert_eq!(edges.child(i).unwrap(), 2 * i);
        }
    }

    #[test]
    fn test_add_site() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_site(0.3, Some(b"Eggnog")).unwrap();
        tables.add_site(0.5, None).unwrap(); // No ancestral_state specified!!!
        let longer_metadata = "Hot Toddy";
        tables
            .add_site(0.9, Some(longer_metadata.as_bytes()))
            .unwrap();

        let sites = tables.sites();
        assert!(close_enough(sites.position(0).unwrap(), 0.3));
        assert!(close_enough(sites.position(1).unwrap(), 0.5));
        assert!(close_enough(sites.position(2).unwrap(), 0.9));

        match sites.ancestral_state(0).unwrap() {
            Some(astate) => assert_eq!(astate, b"Eggnog"),
            None => panic!(),
        };

        if sites.ancestral_state(1).unwrap().is_some() {
            panic!()
        }

        match sites.ancestral_state(2).unwrap() {
            Some(astate) => assert_eq!(astate, longer_metadata.as_bytes()),
            None => panic!(),
        };
    }

    fn close_enough(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_add_mutation() {
        let mut tables = TableCollection::new(1000.).unwrap();

        tables
            .add_mutation(0, 0, crate::TSK_NULL, 1.123, Some(b"pajamas"))
            .unwrap();
        tables
            .add_mutation(1, 1, crate::TSK_NULL, 2.123, None)
            .unwrap();
        tables
            .add_mutation(2, 2, crate::TSK_NULL, 3.123, Some(b"more pajamas"))
            .unwrap();
        let mutations = tables.mutations();
        assert!(close_enough(mutations.time(0).unwrap(), 1.123));
        assert!(close_enough(mutations.time(1).unwrap(), 2.123));
        assert!(close_enough(mutations.time(2).unwrap(), 3.123));
        assert_eq!(mutations.node(0).unwrap(), 0);
        assert_eq!(mutations.node(1).unwrap(), 1);
        assert_eq!(mutations.node(2).unwrap(), 2);
        assert_eq!(mutations.parent(0).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(1).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.parent(2).unwrap(), crate::TSK_NULL);
        assert_eq!(mutations.derived_state(0).unwrap().unwrap(), b"pajamas");

        if mutations.derived_state(1).unwrap().is_some() {
            panic!()
        }

        assert_eq!(
            mutations.derived_state(2).unwrap().unwrap(),
            b"more pajamas"
        );
    }

    struct F {
        x: i32,
        y: u32,
    }

    impl metadata::MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            let mut rv = vec![];
            rv.extend(self.x.to_le_bytes().iter().copied());
            rv.extend(self.y.to_le_bytes().iter().copied());
            Ok(rv)
        }
        fn decode(md: &[u8]) -> Result<Self, MetadataError> {
            use std::convert::TryInto;
            let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
            let (y_int_bytes, _) = rest.split_at(std::mem::size_of::<u32>());
            Ok(Self {
                x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
                y: u32::from_le_bytes(y_int_bytes.try_into().unwrap()),
            })
        }
    }

    #[test]
    fn test_add_mutation_with_metadata() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables
            .add_mutation_with_metadata(
                0,
                0,
                crate::TSK_NULL,
                1.123,
                None,
                Some(&F { x: -3, y: 666 }),
            )
            .unwrap();
        // The double unwrap is to first check for error
        // and then to process the Option.
        let md = tables.mutations().metadata::<F>(0).unwrap().unwrap();
        assert_eq!(md.x, -3);
        assert_eq!(md.y, 666);
    }

    #[test]
    fn test_add_population() {
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_population().unwrap();
        assert_eq!(tables.populations().num_rows(), 1);
    }
}

mod test_table_functions {

    use super::*;

    #[test]
    fn test_dump_tables() {
        let treefile = "trees.trees";
        let mut tables = TableCollection::new(1000.).unwrap();
        tables.add_population().unwrap();
        tables
            .add_node(
                crate::TSK_NODE_IS_SAMPLE,
                0.0,
                crate::TSK_NULL,
                crate::TSK_NULL,
            )
            .unwrap();
        tables
            .add_node(
                crate::TSK_NODE_IS_SAMPLE,
                1.0,
                crate::TSK_NULL,
                crate::TSK_NULL,
            )
            .unwrap();
        tables.add_edge(0., tables.sequence_length(), 1, 0).unwrap();
        tables.dump(&treefile, 0).unwrap();

        let tables2 = TableCollection::new_from_file(&treefile).unwrap();
        assert!(tables.equals(&tables2, 0));

        std::fs::remove_file(&treefile).unwrap();
    }

    #[test]
    fn test_clear() {
        let mut tables = TableCollection::new(1000.).unwrap();
        for i in 0..5 {
            let _ = tables.add_edge(0., 1000., i, 2 * i).unwrap();
        }
        assert_eq!(tables.edges().num_rows(), 5);
        tables.clear(0).unwrap();
        assert_eq!(tables.edges().num_rows(), 0);
    }
}
