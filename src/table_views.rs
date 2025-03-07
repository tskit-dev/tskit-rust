#[cfg(feature = "provenance")]
use crate::provenance::ProvenanceTable;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SiteTable;
use crate::TskitError;

pub struct TableViews {
    edges: EdgeTable,
    nodes: NodeTable,
    sites: SiteTable,
    mutations: MutationTable,
    individuals: IndividualTable,
    populations: PopulationTable,
    migrations: MigrationTable,
    #[cfg(feature = "provenance")]
    provenances: ProvenanceTable,
}

impl TableViews {
    pub(crate) fn new_from_ll_table_collection(
        tables: &mut crate::sys::TableCollection,
    ) -> Result<Self, TskitError> {
        Ok(Self {
            edges: crate::EdgeTable::new_from_table(tables.edges_mut())?,
            nodes: crate::NodeTable::new_from_table(tables.nodes_mut())?,
            sites: crate::SiteTable::new_from_table(tables.sites_mut())?,
            mutations: crate::MutationTable::new_from_table(tables.mutations_mut())?,
            individuals: crate::IndividualTable::new_from_table(tables.individuals_mut())?,
            populations: crate::PopulationTable::new_from_table(tables.populations_mut())?,
            migrations: crate::MigrationTable::new_from_table(tables.migrations_mut())?,
            #[cfg(feature = "provenance")]
            provenances: crate::provenance::ProvenanceTable::new_from_table(
                tables.provenances_mut(),
            )?,
        })
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    pub fn edges(&self) -> &EdgeTable {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut EdgeTable {
        &mut self.edges
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    pub fn nodes(&self) -> &NodeTable {
        &self.nodes
    }

    /// Get mutable reference to the [``NodeTable``](crate::NodeTable).
    pub fn nodes_mut(&mut self) -> &mut NodeTable {
        &mut self.nodes
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    pub fn sites(&self) -> &SiteTable {
        &self.sites
    }

    pub fn sites_mut(&mut self) -> &mut SiteTable {
        &mut self.sites
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    pub fn mutations(&self) -> &MutationTable {
        &self.mutations
    }

    pub fn mutations_mut(&mut self) -> &mut MutationTable {
        &mut self.mutations
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    pub fn individuals(&self) -> &IndividualTable {
        &self.individuals
    }

    pub fn individuals_mut(&mut self) -> &mut IndividualTable {
        &mut self.individuals
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    pub fn populations(&self) -> &PopulationTable {
        &self.populations
    }

    pub fn populations_mut(&mut self) -> &mut PopulationTable {
        &mut self.populations
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    pub fn migrations(&self) -> &MigrationTable {
        &self.migrations
    }

    pub fn migrations_mut(&mut self) -> &mut MigrationTable {
        &mut self.migrations
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances(&self) -> &ProvenanceTable {
        &self.provenances
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances_mut(&mut self) -> &mut ProvenanceTable {
        &mut self.provenances
    }

    /// Return an iterator over the edges.
    pub fn edges_iter(&self) -> impl Iterator<Item = crate::EdgeTableRow> + '_ {
        self.edges.iter()
    }

    /// Return an iterator over the nodes.
    pub fn nodes_iter(&self) -> impl Iterator<Item = crate::NodeTableRow> + '_ {
        self.nodes.iter()
    }

    /// Return an iterator over the sites.
    pub fn sites_iter(&self) -> impl Iterator<Item = crate::SiteTableRow> + '_ {
        self.sites.iter()
    }

    /// Return an iterator over the mutations.
    pub fn mutations_iter(&self) -> impl Iterator<Item = crate::MutationTableRow> + '_ {
        self.mutations.iter()
    }

    /// Return an iterator over the individuals.
    pub fn individuals_iter(&self) -> impl Iterator<Item = crate::IndividualTableRow> + '_ {
        self.individuals.iter()
    }

    /// Return an iterator over the populations.
    pub fn populations_iter(&self) -> impl Iterator<Item = crate::PopulationTableRow> + '_ {
        self.populations.iter()
    }

    /// Return an iterator over the migrations.
    pub fn migrations_iter(&self) -> impl Iterator<Item = crate::MigrationTableRow> + '_ {
        self.migrations.iter()
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Return an iterator over provenances
    pub fn provenances_iter(
        &self,
    ) -> impl Iterator<Item = crate::provenance::ProvenanceTableRow> + '_ {
        self.provenances.iter()
    }

    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::NodeFlags::is_sample`]
    /// is `true`.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::samples_as_vector`].
    pub fn samples_as_vector(&self) -> Vec<crate::NodeId> {
        self.nodes().samples_as_vector()
    }

    /// Obtain a vector containing the indexes ("ids") of all nodes
    /// satisfying a certain criterion.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::create_node_id_vector`].
    ///
    /// # Parameters
    ///
    /// * `f`: a function.  The function is passed the current table
    ///    collection and each [`crate::node_table::NodeTableRow`].
    ///    If `f` returns `true`, the index of that row is included
    ///    in the return value.
    ///
    /// # Examples
    ///
    /// Get all nodes with time > 0.0:
    ///
    /// ```
    /// let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// tables
    ///     .add_node(tskit::NodeFlags::new_sample(), 0.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
    ///     .unwrap();
    /// tables
    ///     .add_node(tskit::NodeFlags::new_sample(), 1.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
    ///     .unwrap();
    /// let samples = tables.create_node_id_vector(
    ///     |row: &tskit::NodeTableRow| row.time > 0.,
    /// );
    /// assert_eq!(samples[0], 1);
    /// ```
    pub fn create_node_id_vector(
        &self,
        f: impl FnMut(&crate::NodeTableRow) -> bool,
    ) -> Vec<crate::NodeId> {
        self.nodes().create_node_id_vector(f)
    }
}

#[cfg(test)]
mod table_views_tests {
    #[test]
    fn test_treeseq() {
        let mut tables = crate::TableCollection::new(100.).unwrap();
        tables.add_node(0, 0.0, -1, -1).unwrap(); // child
        tables.add_node(0, 1.0, -1, -1).unwrap(); // parent
        tables.add_edge(0., 100., 1, 0).unwrap();
        let ts = tables
            .tree_sequence(crate::TreeSequenceFlags::BUILD_INDEXES)
            .unwrap();
        assert_eq!(ts.edges().num_rows(), 1);
        assert_eq!(ts.nodes().num_rows(), 2);

        #[cfg(feature = "provenance")]
        {
            let mut ts = ts;
            assert_eq!(ts.provenances().num_rows(), 0);
            ts.add_provenance("bananas").unwrap();
            assert_eq!(ts.provenances().num_rows(), 1);
        }
    }
}
