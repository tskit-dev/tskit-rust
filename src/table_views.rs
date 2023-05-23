use std::ptr::NonNull;

#[cfg(feature = "provenance")]
use crate::provenance::ProvenanceTable;
use crate::sys::bindings as ll_bindings;
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
    pub(crate) fn new_from_mbox_table_collection(
        tables: &mut mbox::MBox<ll_bindings::tsk_table_collection_t>,
    ) -> Result<Self, TskitError> {
        Ok(Self {
            edges: crate::EdgeTable::new_from_table(&mut tables.as_mut().edges)?,
            nodes: crate::NodeTable::new_from_table(&mut tables.as_mut().nodes)?,
            sites: crate::SiteTable::new_from_table(&mut tables.as_mut().sites)?,
            mutations: crate::MutationTable::new_from_table(&mut tables.as_mut().mutations)?,
            individuals: crate::IndividualTable::new_from_table(&mut tables.as_mut().individuals)?,
            populations: crate::PopulationTable::new_from_table(&mut tables.as_mut().populations)?,
            migrations: crate::MigrationTable::new_from_table(&mut tables.as_mut().migrations)?,
            #[cfg(feature = "provenance")]
            provenances: crate::provenance::ProvenanceTable::new_from_table(
                &mut tables.as_mut().provenances,
            )?,
        })
    }

    pub(crate) fn new_from_NonNull_table_collection(
        tables: &mut NonNull<ll_bindings::tsk_table_collection_t>,
    ) -> Result<Self, TskitError> {
        Ok(Self {
            edges: crate::EdgeTable::new_from_table(&mut unsafe { tables.as_mut() }.edges)?,
            nodes: crate::NodeTable::new_from_table(&mut unsafe { tables.as_mut() }.nodes)?,
            sites: crate::SiteTable::new_from_table(&mut unsafe { tables.as_mut() }.sites)?,
            mutations: crate::MutationTable::new_from_table(
                &mut unsafe { tables.as_mut() }.mutations,
            )?,
            individuals: crate::IndividualTable::new_from_table(
                &mut unsafe { tables.as_mut() }.individuals,
            )?,
            populations: crate::PopulationTable::new_from_table(
                &mut unsafe { tables.as_mut() }.populations,
            )?,
            migrations: crate::MigrationTable::new_from_table(
                &mut unsafe { tables.as_mut() }.migrations,
            )?,
            #[cfg(feature = "provenance")]
            provenances: crate::provenance::ProvenanceTable::new_from_table(
                &mut unsafe { tables.as_mut() }.provenances,
            )?,
        })
    }

    pub(crate) fn new_from_tree_sequence(
        treeseq: *mut ll_bindings::tsk_treeseq_t,
    ) -> Result<Self, TskitError> {
        if treeseq.is_null() {
            return Err(TskitError::from(
                crate::error::TskitErrorData::LibraryError(
                    "tree sequence pointer is null".to_string(),
                ),
            ));
        }
        let mut n = NonNull::new(unsafe { *treeseq }.tables).ok_or_else(|| {
            TskitError::from(crate::error::TskitErrorData::LibraryError(
                "tree sequence contains NULL pointer to table collection".to_string(),
            ))
        })?;
        Self::new_from_NonNull_table_collection(&mut n)
    }

    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    pub fn edges(&self) -> &EdgeTable {
        &self.edges
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

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    pub fn mutations(&self) -> &MutationTable {
        &self.mutations
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    pub fn individuals(&self) -> &IndividualTable {
        &self.individuals
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    pub fn populations(&self) -> &PopulationTable {
        &self.populations
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    pub fn migrations(&self) -> &MigrationTable {
        &self.migrations
    }

    #[cfg(feature = "provenance")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    pub fn provenances(&self) -> &ProvenanceTable {
        &self.provenances
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
