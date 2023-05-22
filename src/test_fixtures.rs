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
