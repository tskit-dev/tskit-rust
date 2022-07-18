use crate::{
    EdgeTable, IndividualTable, MigrationTable, MutationTable, NodeTable, PopulationTable,
    SiteTable,
};

pub(crate) fn partial_cmp_equal<T: PartialOrd>(lhs: &T, rhs: &T) -> bool {
    matches!(lhs.partial_cmp(rhs), Some(std::cmp::Ordering::Equal))
}

pub(crate) struct TableReferences {
    pub edges: EdgeTable,
    pub nodes: NodeTable,
    pub sites: SiteTable,
    pub mutations: MutationTable,
    pub individuals: IndividualTable,
    pub migrations: MigrationTable,
    pub populations: PopulationTable,
    #[cfg(feature = "provenance")]
    pub provenances: crate::provenance::ProvenanceTable,
}

impl Default for TableReferences {
    fn default() -> Self {
        Self {
            edges: EdgeTable::new_null(),
            nodes: NodeTable::new_null(),
            sites: SiteTable::new_null(),
            mutations: MutationTable::new_null(),
            individuals: IndividualTable::new_null(),
            migrations: MigrationTable::new_null(),
            populations: PopulationTable::new_null(),
            #[cfg(feature = "provenance")]
            provenances: crate::provenance::ProvenanceTable::new_null(),
        }
    }
}
