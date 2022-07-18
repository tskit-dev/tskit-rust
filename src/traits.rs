//! Traits related to user-facing types

use crate::table_iterator::make_table_iterator;
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
    fn edges_iter(&self) -> Box<dyn Iterator<Item = crate::edge_table::EdgeTableRow> + '_> {
        Box::new(make_table_iterator::<EdgeTable>(self.edges()))
    }

    /// Get reference to the [``NodeTable``](crate::NodeTable).
    fn nodes(&self) -> NodeTable;

    /// Return an iterator over the nodes.
    fn nodes_iter(&self) -> Box<dyn Iterator<Item = crate::node_table::NodeTableRow> + '_> {
        Box::new(make_table_iterator::<NodeTable>(self.nodes()))
    }

    /// Get reference to the [``MutationTable``](crate::MutationTable).
    fn mutations(&self) -> MutationTable;

    /// Return an iterator over the mutations.
    fn mutations_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::mutation_table::MutationTableRow> + '_> {
        Box::new(make_table_iterator::<MutationTable>(self.mutations()))
    }

    /// Get reference to the [``SiteTable``](crate::SiteTable).
    fn sites(&self) -> &SiteTable;

    /// Return an iterator over the sites.
    fn sites_iter(&self) -> Box<dyn Iterator<Item = crate::site_table::SiteTableRow> + '_> {
        Box::new(make_table_iterator::<&SiteTable>(self.sites()))
    }

    /// Get reference to the [``PopulationTable``](crate::PopulationTable).
    fn populations(&self) -> &PopulationTable;

    /// Return an iterator over the populations.
    fn populations_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::population_table::PopulationTableRow> + '_> {
        Box::new(make_table_iterator::<&PopulationTable>(self.populations()))
    }

    /// Get reference to the [``MigrationTable``](crate::MigrationTable).
    fn migrations(&self) -> MigrationTable;

    /// Return an iterator over the migration events.
    fn migrations_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::migration_table::MigrationTableRow> + '_> {
        Box::new(make_table_iterator::<MigrationTable>(self.migrations()))
    }

    /// Get reference to the [``IndividualTable``](crate::IndividualTable).
    fn individuals(&self) -> IndividualTable;

    /// Return an iterator over the individuals.
    fn individuals_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::individual_table::IndividualTableRow> + '_> {
        Box::new(make_table_iterator::<IndividualTable>(self.individuals()))
    }

    #[cfg(any(feature = "provenance", doc))]
    /// Get reference to the [``ProvenanceTable``](crate::provenance::ProvenanceTable)
    fn provenances(&self) -> &crate::provenance::ProvenanceTable;

    #[cfg(any(feature = "provenance", doc))]
    /// Return an iterator over provenances
    fn provenances_iter(
        &self,
    ) -> Box<dyn Iterator<Item = crate::provenance::ProvenanceTableRow> + '_> {
        Box::new(crate::table_iterator::make_table_iterator::<
            &crate::provenance::ProvenanceTable,
        >(self.provenances()))
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
    fn samples_as_vector(&self) -> Vec<crate::NodeId> {
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
    /// use tskit::bindings::tsk_id_t;
    /// use tskit::TableAccess;
    /// use tskit::NodeListGenerator;
    ///
    /// let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// tables
    ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 0.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
    ///     .unwrap();
    /// tables
    ///     .add_node(tskit::TSK_NODE_IS_SAMPLE, 1.0, tskit::PopulationId::NULL,
    ///     tskit::IndividualId::NULL)
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
    ///     tables_type: &dyn tskit::TableAccess,
    ///     row: &tskit::NodeTableRow,
    /// ) -> bool {
    ///     for mrow in tables_type.mutations_iter() {
    ///         if mrow.node == row.id {
    ///             return true;
    ///         }
    ///     }
    ///     false
    /// }
    ///
    /// // Get all nodes that have a mutation:
    ///
    /// tables.add_mutation(0, 0, tskit::MutationId::NULL, 0.0, None).unwrap();
    /// let samples_with_mut = tables.create_node_id_vector(
    ///     |row: &tskit::NodeTableRow| node_has_mutation(&tables, row));
    /// assert_eq!(samples_with_mut[0], 0);
    /// ```

    fn create_node_id_vector(
        &self,
        f: impl FnMut(&crate::NodeTableRow) -> bool,
    ) -> Vec<crate::NodeId> {
        self.nodes().create_node_id_vector(f)
    }
}

/// Abstraction of individual location.
///
/// This trait exists to streamline the API of
/// [`TableCollection::add_individual`](crate::TableCollection::add_individual)
/// and
/// [`TableCollection::add_individual_with_metadata`](crate::TableCollection::add_individual_with_metadata).
pub trait IndividualLocation {
    fn get_slice(&self) -> &[crate::Location];
}

macro_rules! impl_individual_location {
    ($for: ty, $self_:ident,$body: expr) => {
        impl IndividualLocation for $for {
            fn get_slice(&$self_) -> &[crate::Location] {
                $body
            }
        }
    };
    ($n: ident, $nty: ty, $for: ty, $self_:ident,$body: expr) => {
        impl<const $n: $nty> IndividualLocation for $for {
            fn get_slice(&$self_) -> &[crate::Location] {
                $body
            }
        }
    };
}

impl_individual_location!(
    Option<&[crate::Location]>,
    self,
    match self {
        Some(s) => s,
        None => &[],
    }
);
impl_individual_location!(&[crate::Location], self, self);
impl_individual_location!(&Vec<crate::Location>, self, self.as_slice());
impl_individual_location!(Vec<crate::Location>, self, self.as_slice());
impl_individual_location!(&[f64], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::Location, self.len())
});
impl_individual_location!(&Vec<f64>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::Location, self.len())
});
impl_individual_location!(Vec<f64>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::Location, self.len())
});
impl_individual_location!(N, usize, &[f64; N], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::Location, self.len())
});
impl_individual_location!(N, usize, [f64; N], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::Location, self.len())
});
impl_individual_location!(N, usize, &[crate::Location; N], self, self.as_slice());
impl_individual_location!(N, usize, [crate::Location; N], self, self.as_slice());

/// Abstraction of individual parents.
///
/// This trait exists to streamline the API of
/// [`TableCollection::add_individual`](crate::TableCollection::add_individual)
/// and
/// [`TableCollection::add_individual_with_metadata`](crate::TableCollection::add_individual_with_metadata).
pub trait IndividualParents {
    fn get_slice(&self) -> &[crate::IndividualId];
}

macro_rules! impl_individual_parents {
    ($for: ty, $self_:ident,$body: expr) => {
        impl IndividualParents for $for {
            fn get_slice(&$self_) -> &[crate::IndividualId] {
                $body
            }
        }
    };
    ($n: ident, $nty: ty, $for: ty, $self_:ident,$body: expr) => {
        impl<const $n: $nty> IndividualParents for $for {
            fn get_slice(&$self_) -> &[crate::IndividualId] {
                $body
            }
        }
    };
}

impl_individual_parents!(
    Option<&[crate::IndividualId]>,
    self,
    match self {
        Some(s) => s,
        None => &[],
    }
);
impl_individual_parents!(&[crate::IndividualId], self, self);
impl_individual_parents!(&Vec<crate::IndividualId>, self, self.as_slice());
impl_individual_parents!(Vec<crate::IndividualId>, self, self.as_slice());
impl_individual_parents!(&[crate::bindings::tsk_id_t], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(&Vec<crate::bindings::tsk_id_t>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(Vec<crate::bindings::tsk_id_t>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(N, usize, &[crate::bindings::tsk_id_t; N], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(N, usize, [crate::bindings::tsk_id_t; N], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(N, usize, &[crate::IndividualId; N], self, self.as_slice());
impl_individual_parents!(N, usize, [crate::IndividualId; N], self, self.as_slice());
