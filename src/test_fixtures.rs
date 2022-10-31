#[cfg(test)]
use crate::*;

#[cfg(test)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub struct GenericMetadata {
    pub data: i64,
}

#[cfg(test)]
impl Default for GenericMetadata {
    fn default() -> Self {
        Self { data: 42 }
    }
}

#[cfg(test)]
impl crate::metadata::MetadataRoundtrip for GenericMetadata {
    fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
        handle_metadata_return!(bincode::serialize(&self))
    }

    fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
        handle_metadata_return!(bincode::deserialize(md))
    }
}

#[cfg(test)]
impl crate::metadata::MutationMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::SiteMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::EdgeMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::NodeMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::IndividualMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::PopulationMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::MigrationMetadata for GenericMetadata {}

#[cfg(test)]
pub fn make_small_table_collection() -> TableCollection {
    let mut tables = TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(0., 1000., 0, 1).unwrap();
    tables.add_edge(0., 1000., 0, 2).unwrap();
    tables.build_index().unwrap();
    tables
}

#[cfg(test)]
pub fn treeseq_from_small_table_collection() -> TreeSequence {
    let tables = make_small_table_collection();
    tables.tree_sequence(TreeSequenceFlags::default()).unwrap()
}

#[cfg(test)]
pub fn make_small_table_collection_two_trees() -> TableCollection {
    // The two trees are:
    //  0
    // +++
    // | |  1
    // | | +++
    // 2 3 4 5

    //     0
    //   +-+-+
    //   1   |
    // +-+-+ |
    // 2 4 5 3

    let mut tables = TableCollection::new(1000.).unwrap();
    tables
        .add_node(0, 2.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(0, 1.0, PopulationId::NULL, IndividualId::NULL)
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables
        .add_node(
            TSK_NODE_IS_SAMPLE,
            0.0,
            PopulationId::NULL,
            IndividualId::NULL,
        )
        .unwrap();
    tables.add_edge(500., 1000., 0, 1).unwrap();
    tables.add_edge(0., 500., 0, 2).unwrap();
    tables.add_edge(0., 1000., 0, 3).unwrap();
    tables.add_edge(500., 1000., 1, 2).unwrap();
    tables.add_edge(0., 1000., 1, 4).unwrap();
    tables.add_edge(0., 1000., 1, 5).unwrap();
    tables.full_sort(TableSortOptions::default()).unwrap();
    tables.build_index().unwrap();
    tables
}

#[cfg(test)]
pub fn treeseq_from_small_table_collection_two_trees() -> TreeSequence {
    let tables = make_small_table_collection_two_trees();
    tables.tree_sequence(TreeSequenceFlags::default()).unwrap()
}

#[cfg(test)]
pub mod bad_metadata {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct F {
        pub x: i32,
        pub y: u32,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct Ff {
        pub x: i32,
        pub y: u64,
    }

    impl crate::metadata::MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::serialize(&self))
        }

        fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::deserialize(md))
        }
    }

    impl crate::metadata::MetadataRoundtrip for Ff {
        fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::serialize(&self))
        }

        fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::deserialize(md))
        }
    }

    impl crate::metadata::MutationMetadata for F {}
    impl crate::metadata::MutationMetadata for Ff {}
}
