//! Traits related to user-facing types

use crate::edge_table::EdgeTableIterator;
use crate::individual_table::IndividualTableIterator;
use crate::migration_table::MigrationTableIterator;
use crate::mutation_table::MutationTableIterator;
use crate::node_table::NodeTableIterator;
use crate::population_table::PopulationTableIterator;
use crate::site_table::SiteTableIterator;
use crate::table_iterator::make_table_iterator;
use crate::tsk_id_t;
use crate::EdgeTable;
use crate::IndividualTable;
use crate::MigrationTable;
use crate::MutationTable;
use crate::NodeTable;
use crate::PopulationTable;
use crate::SiteTable;

/// Provide pointer access to underlying C types
pub trait TskitTypeAccess<T> {
    /// Return const pointer
    fn as_ptr(&self) -> *const T;
    /// Return mutable pointer
    fn as_mut_ptr(&mut self) -> *mut T;
}

/// Immutable access to tables.
///
/// For objects that contain the full suite of tables,
/// implementing this trait provides immutable access
/// to their data.
///
/// For most types, the provided implementations of `_iter`
/// methods should do.
///
/// # Examples
///
/// ```
/// use tskit::TableAccess;
///
/// let mut tables = tskit::TableCollection::new(1.).unwrap();
/// // The borrows are immuatble, so we can
/// // take multiple table references from the same object.
/// let e = tables.edges();
/// let n = tables.nodes();
/// ```
///
/// The borrow checker will keep you from getting in trouble:
///
/// ```compile_fail
/// use tskit::TableAccess;
///
/// let mut tables = tskit::TableCollection::new(1.).unwrap();
/// let e = tables.edges();
/// tables.add_edge(0.0, 1.0, 0, 1).unwrap();
/// let p = e.parent(0).unwrap();   // FAIL!
/// ```
pub trait TableAccess {
    /// Get reference to the [``EdgeTable``](crate::EdgeTable).
    fn edges(&self) -> EdgeTable;

    /// Return an iterator over the edges.
    /// See [`EdgeTable::iter`] for details.
    fn edges_iter(&self, decode_metadata: bool) -> EdgeTableIterator {
        make_table_iterator::<EdgeTable>(self.edges(), decode_metadata)
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    fn nodes(&self) -> NodeTable;

    /// Return an iterator over the nodes.
    /// See [`NodeTable::iter`] for details.
    fn nodes_iter(&self, decode_metadata: bool) -> NodeTableIterator {
        make_table_iterator::<NodeTable>(self.nodes(), decode_metadata)
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    fn mutations(&self) -> MutationTable;

    /// Return an iterator over the mutations.
    /// See [`MutationTable::iter`] for details.
    fn mutations_iter(&self, decode_metadata: bool) -> MutationTableIterator {
        make_table_iterator::<MutationTable>(self.mutations(), decode_metadata)
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    fn sites(&self) -> SiteTable;

    /// Return an iterator over the sites.
    /// See [`SiteTable::iter`] for details.
    fn sites_iter(&self, decode_metadata: bool) -> SiteTableIterator {
        make_table_iterator::<SiteTable>(self.sites(), decode_metadata)
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    fn populations(&self) -> PopulationTable;

    /// Return an iterator over the populations.
    /// See [`PopulationTable::iter`] for details.
    fn populations_iter(&self, decode_metadata: bool) -> PopulationTableIterator {
        make_table_iterator::<PopulationTable>(self.populations(), decode_metadata)
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    fn migrations(&self) -> MigrationTable;

    /// Return an iterator over the migration events.
    /// See [`MigrationTable::iter`] for details.
    fn migrations_iter(&self, decode_metadata: bool) -> MigrationTableIterator {
        make_table_iterator::<MigrationTable>(self.migrations(), decode_metadata)
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    fn individuals(&self) -> IndividualTable;

    /// Return an iterator over the individuals.
    /// See [`IndividualTable::iter`] for details.
    fn individuals_iter(&self, decode_metadata: bool) -> IndividualTableIterator {
        make_table_iterator::<IndividualTable>(self.individuals(), decode_metadata)
    }
}

/// Interface for returning lists of node ids from
/// types implementing [`TableAccess`].
pub trait NodeListGenerator: TableAccess {
    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::TSK_NODE_IS_SAMPLE`]
    /// is `true`.
    ///
    /// The provided implementation dispatches to
    /// [`crate::NodeTable::samples_as_vector`].
    fn samples_as_vector(&self) -> Vec<tsk_id_t> {
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
    /// use tskit::TSK_NULL;
    /// use tskit::tsk_id_t;
    /// use tskit::TableAccess;
    /// use tskit::NodeListGenerator;
    ///
    /// let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// tables
    ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 0.0, TSK_NULL, TSK_NULL)
    ///     .unwrap();
    /// tables
    ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 1.0, TSK_NULL, TSK_NULL)
    ///     .unwrap();
    /// let samples = tables.create_node_id_vector(
    ///     |row: &tskit::NodeTableRow| row.time > 0.,
    /// );
    /// assert_eq!(samples[0], 1);
    ///
    /// // Get all nodes that have a mutation:
    ///
    /// fn node_has_mutation(
    ///     // dyn trait here means this
    ///     // will work with TreeSequence, too.
    ///     tabled_type: &dyn tskit::TableAccess,
    ///     row: &tskit::NodeTableRow,
    /// ) -> bool {
    ///     for mrow in tabled_type.mutations_iter(false) {
    ///         if mrow.node == row.id {
    ///             return true;
    ///         }
    ///     }
    ///     false
    /// }
    ///
    /// // Get all nodes that have a mutation:
    ///
    /// tables.add_mutation(0, 0, TSK_NULL, 0.0, None).unwrap();
    /// let samples_with_mut = tables.create_node_id_vector(
    ///     |row: &tskit::NodeTableRow| node_has_mutation(&tables, row));
    /// assert_eq!(samples_with_mut[0], 0);
    /// ```

    fn create_node_id_vector(&self, f: impl FnMut(&crate::NodeTableRow) -> bool) -> Vec<tsk_id_t> {
        self.nodes().create_node_id_vector(f)
    }
}
