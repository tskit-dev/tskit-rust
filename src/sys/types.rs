macro_rules! metadata_body {
    ($self: expr) => {
        if $self.row.metadata_length > 0 {
            let metadata_size = usize::try_from($self.row.metadata_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    $self.row.metadata.cast::<u8>(),
                    metadata_size,
                ))
            }
        } else {
            None
        }
    };
}

/// Reference to a site stored in a tree sequence.
#[derive(Debug)]
pub struct SiteRef<'p, P> {
    row: &'p super::bindings::tsk_site_t,
    marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for SiteRef<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.position().eq(&other.position())
            && self.ancestral_state().eq(&other.ancestral_state())
            && self.metadata().eq(&other.metadata())
            // NOTE: the .eq() below is Iterator::eq
            && self.mutations().eq(other.mutations())
    }
}

pub(super) fn new_site_ref<'p, P>(
    _parent: &'p P,
    site: &'p super::bindings::tsk_site_t,
) -> SiteRef<'p, P> {
    SiteRef {
        row: site,
        marker: std::marker::PhantomData,
    }
}

impl<'parent, P> SiteRef<'parent, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::SiteId {
        self.row.id.into()
    }
    /// Position
    #[inline(always)]
    pub fn position(&self) -> super::newtypes::Position {
        self.row.position.into()
    }

    /// Return iterator over [`MutationRef`] at this site.
    /// Iteration order is identical to internal storage order.
    // NOTE: not populated by tsk_site_table_get_row,
    // which leaves the pointer NULL!
    pub fn mutations(
        &self,
    ) -> impl Iterator<Item = MutationRef<'parent, super::bindings::tsk_mutation_t>> {
        assert!(!self.row.mutations.is_null());
        let mslice = unsafe {
            std::slice::from_raw_parts(self.row.mutations, self.row.mutations_length as usize)
        };
        mslice.iter().map(|m| MutationRef {
            row: m,
            marker: std::marker::PhantomData,
        })
    }

    /// Ancestral state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no ancestral state
    /// * `Some(data)` if ancestral state is present
    pub fn ancestral_state(&self) -> Option<&[u8]> {
        if self.row.ancestral_state_length > 0 {
            let ancestral_state_size = usize::try_from(self.row.ancestral_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.row.ancestral_state.cast::<u8>(),
                    ancestral_state_size,
                ))
            }
        } else {
            None
        }
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }
}

/// Reference to a mutation stored in a tree sequence.
#[derive(Debug)]
pub struct MutationRef<'p, P> {
    row: &'p super::bindings::tsk_mutation_t,
    marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for MutationRef<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.site().eq(&other.site())
            && self.node().eq(&other.node())
            && self.parent().eq(&other.parent())
            && self.edge().eq(&other.edge())
            && self.time().eq(&other.time())
            && self.derived_state().eq(&other.derived_state())
            && self.metadata().eq(&other.metadata())
            && self.inherited_state().eq(&other.inherited_state())
    }
}

impl<'parent, P> MutationRef<'parent, P> {
    /// Mutation id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::MutationId {
        self.row.id.into()
    }
    /// Site id of this mutation
    #[inline(always)]
    pub fn site(&self) -> super::newtypes::SiteId {
        self.row.site.into()
    }

    /// Node id of this mutation
    #[inline(always)]
    pub fn node(&self) -> super::newtypes::NodeId {
        self.row.node.into()
    }

    /// Parent mutation of this mutation
    #[inline(always)]
    pub fn parent(&self) -> super::newtypes::MutationId {
        self.row.parent.into()
    }

    /// Origin time of mutation
    #[inline(always)]
    pub fn time(&self) -> super::newtypes::Time {
        self.row.time.into()
    }

    /// Edge of mutation
    // NOTE: this will be NULL when populated
    // by tsk_mutation_table_get_row_unsafe
    #[inline(always)]
    pub fn edge(&self) -> super::newtypes::EdgeId {
        self.row.edge.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }

    /// Derived state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no derived state
    /// * `Some(data)` if derived state is present
    pub fn derived_state(&self) -> Option<&[u8]> {
        if self.row.derived_state_length > 0 {
            let derived_state_size = usize::try_from(self.row.derived_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.row.derived_state.cast::<u8>(),
                    derived_state_size,
                ))
            }
        } else {
            None
        }
    }

    /// Inherited state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no inherited state
    /// * `Some(data)` if inherited state is present
    // NOTE: not populated by tsk_mutation_table_get_row_unsafe
    pub fn inherited_state(&self) -> Option<&[u8]> {
        if self.row.inherited_state_length > 0 {
            let inherited_state_size = usize::try_from(self.row.inherited_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.row.inherited_state.cast::<u8>(),
                    inherited_state_size,
                ))
            }
        } else {
            None
        }
    }
}

/// A lifetime-bound site.
#[derive(Debug)]
pub struct Site<'p, P> {
    pub(super) row: super::bindings::tsk_site_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Site<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.position().eq(&other.position())
            && self.ancestral_state().eq(&other.ancestral_state())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Site<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::SiteId {
        self.row.id.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }

    /// Ancestral state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no ancestral state
    /// * `Some(data)` if ancestral state is present
    pub fn ancestral_state(&self) -> Option<&[u8]> {
        if self.row.ancestral_state_length > 0 {
            let ancestral_state_size = usize::try_from(self.row.ancestral_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.row.ancestral_state.cast::<u8>(),
                    ancestral_state_size,
                ))
            }
        } else {
            None
        }
    }

    /// Position
    #[inline(always)]
    pub fn position(&self) -> super::newtypes::Position {
        self.row.position.into()
    }
}

/// A lifetime-bound Mutation.
#[derive(Debug)]
pub struct Mutation<'p, P> {
    pub(super) row: super::bindings::tsk_mutation_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Mutation<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.site().eq(&other.site())
            && self.node().eq(&other.node())
            && self.parent().eq(&other.parent())
            && self.edge().eq(&other.edge())
            && self.time().eq(&other.time())
            && self.derived_state().eq(&other.derived_state())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Mutation<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::SiteId {
        self.row.id.into()
    }

    /// Site id of this mutation
    #[inline(always)]
    pub fn site(&self) -> super::newtypes::SiteId {
        self.row.site.into()
    }

    /// Node id of this mutation
    #[inline(always)]
    pub fn node(&self) -> super::newtypes::NodeId {
        self.row.node.into()
    }

    /// Parent mutation of this mutation
    #[inline(always)]
    pub fn parent(&self) -> super::newtypes::MutationId {
        self.row.parent.into()
    }

    /// Origin time of mutation
    #[inline(always)]
    pub fn time(&self) -> super::newtypes::Time {
        self.row.time.into()
    }

    /// Edge of mutation
    // NOTE: this will be NULL when populated
    // by tsk_mutation_table_get_row_unsafe
    #[inline(always)]
    pub fn edge(&self) -> super::newtypes::EdgeId {
        self.row.edge.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }

    /// Derived state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no derived state
    /// * `Some(data)` if derived state is present
    pub fn derived_state(&self) -> Option<&[u8]> {
        if self.row.derived_state_length > 0 {
            let derived_state_size = usize::try_from(self.row.derived_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.row.derived_state.cast::<u8>(),
                    derived_state_size,
                ))
            }
        } else {
            None
        }
    }
}

/// A lifetime-bound Edge.
#[derive(Debug)]
pub struct Edge<'p, P> {
    pub(super) row: super::bindings::tsk_edge_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Edge<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.parent().eq(&other.parent())
            && self.child().eq(&other.child())
            && self.left().eq(&other.left())
            && self.right().eq(&other.right())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Edge<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::EdgeId {
        self.row.id.into()
    }

    /// Left coordinate of edge
    #[inline(always)]
    pub fn left(&self) -> super::newtypes::Position {
        self.row.left.into()
    }

    /// Righ coordinate of edge
    #[inline(always)]
    pub fn right(&self) -> super::newtypes::Position {
        self.row.right.into()
    }

    /// Parent node
    #[inline(always)]
    pub fn parent(&self) -> super::newtypes::NodeId {
        self.row.parent.into()
    }

    /// Child node
    #[inline(always)]
    pub fn child(&self) -> super::newtypes::NodeId {
        self.row.child.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }
}

/// A lifetime-bound Migration.
#[derive(Debug)]
pub struct Migration<'p, P> {
    pub(super) row: super::bindings::tsk_migration_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Migration<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.source().eq(&other.source())
            && self.dest().eq(&other.dest())
            && self.node().eq(&other.node())
            && self.left().eq(&other.left())
            && self.time().eq(&other.time())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Migration<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::SiteId {
        self.row.id.into()
    }

    /// Left position of migrated segment
    #[inline(always)]
    pub fn left(&self) -> super::newtypes::Position {
        self.row.left.into()
    }

    /// Right position of migrated segment
    #[inline(always)]
    pub fn right(&self) -> super::newtypes::Position {
        self.row.right.into()
    }

    /// Source population
    #[inline(always)]
    pub fn source(&self) -> super::newtypes::PopulationId {
        self.row.source.into()
    }

    /// Destination population
    #[inline(always)]
    pub fn dest(&self) -> super::newtypes::PopulationId {
        self.row.dest.into()
    }

    /// Time of migration event
    #[inline(always)]
    pub fn time(&self) -> super::newtypes::Time {
        self.row.time.into()
    }

    /// Node that migrated
    #[inline(always)]
    pub fn node(&self) -> super::newtypes::NodeId {
        self.row.node.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }
}

/// A lifetime-bound Individual.
#[derive(Debug)]
pub struct Individual<'p, P> {
    pub(super) row: super::bindings::tsk_individual_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Individual<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.flags().eq(&other.flags())
            && self.location().eq(&other.location())
            && self.parents().eq(&other.parents())
            && self.metadata().eq(&other.metadata())
            && self.nodes().eq(&other.nodes())
    }
}

impl<'p, P> Individual<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::IndividualId {
        self.row.id.into()
    }

    /// Individual flags
    #[inline(always)]
    pub fn flags(&self) -> super::flags::IndividualFlags {
        self.row.flags.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }

    /// Individual location
    pub fn location(&self) -> Option<&[super::newtypes::Location]> {
        if self.row.location_length > 0 {
            assert!(!self.row.location.is_null());
            let len = usize::try_from(self.row.location_length).ok()?;
            Some(unsafe {
                std::slice::from_raw_parts(
                    self.row.location.cast::<super::newtypes::Location>(),
                    len,
                )
            })
        } else {
            None
        }
    }

    /// Individual parents
    pub fn parents(&self) -> Option<&[super::newtypes::IndividualId]> {
        if self.row.parents_length > 0 {
            assert!(!self.row.parents.is_null());
            let len = usize::try_from(self.row.parents_length).ok()?;
            Some(unsafe {
                std::slice::from_raw_parts(
                    self.row.parents.cast::<super::newtypes::IndividualId>(),
                    len,
                )
            })
        } else {
            None
        }
    }

    /// Individual nodes
    pub fn nodes(&self) -> Option<&[super::newtypes::NodeId]> {
        if self.row.nodes_length > 0 {
            assert!(!self.row.nodes.is_null());
            let len = usize::try_from(self.row.nodes_length).ok()?;
            Some(unsafe {
                std::slice::from_raw_parts(self.row.nodes.cast::<super::newtypes::NodeId>(), len)
            })
        } else {
            None
        }
    }
}

/// A lifetime-bound Node.
#[derive(Debug)]
pub struct Node<'p, P> {
    pub(super) row: super::bindings::tsk_node_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Node<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.flags().eq(&other.flags())
            && self.time().eq(&other.time())
            && self.population().eq(&other.population())
            && self.individual().eq(&other.individual())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Node<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::NodeId {
        self.row.id.into()
    }

    /// Node flags
    #[inline(always)]
    pub fn flags(&self) -> super::flags::NodeFlags {
        self.row.flags.into()
    }

    /// Node time
    #[inline(always)]
    pub fn time(&self) -> super::newtypes::Time {
        self.row.time.into()
    }

    /// Node population
    #[inline(always)]
    pub fn population(&self) -> super::newtypes::PopulationId {
        self.row.population.into()
    }

    /// Node individual
    #[inline(always)]
    pub fn individual(&self) -> super::newtypes::IndividualId {
        self.row.population.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }
}

/// A lifetime-bound Population.
#[derive(Debug)]
pub struct Population<'p, P> {
    pub(super) row: super::bindings::tsk_population_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

impl<'p, P> std::cmp::PartialEq for Population<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id()) && self.metadata().eq(&other.metadata())
    }
}

impl<'p, P> Population<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::PopulationId {
        self.row.id.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self)
    }
}

#[derive(Debug)]
#[cfg(feature = "provenance")]
pub struct Provenance<'p, P> {
    pub(super) row: super::bindings::tsk_provenance_t,
    pub(super) marker: std::marker::PhantomData<&'p P>,
}

#[cfg(feature = "provenance")]
impl<'p, P> std::cmp::PartialEq for Provenance<'p, P> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.timestamp().eq(other.timestamp())
            && self.record().eq(other.record())
    }
}

#[cfg(feature = "provenance")]
impl<'p, P> Provenance<'p, P> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::ProvenanceId {
        self.row.id.into()
    }

    /// Provenance time stamp
    pub fn timestamp(&self) -> &str {
        assert!(self.row.timestamp_length > 0);
        assert!(!self.row.timestamp.is_null());
        let len = usize::try_from(self.row.timestamp_length).unwrap();
        // SAFETY: not NULL, char and u8 have same sizeof,
        // and data owned by parent object w/liftetime 'p
        let bytes = unsafe { std::slice::from_raw_parts(self.row.timestamp.cast::<u8>(), len) };
        std::str::from_utf8(bytes).unwrap()
    }

    /// Provenance record
    pub fn record(&self) -> &str {
        assert!(self.row.record_length > 0);
        assert!(!self.row.timestamp.is_null());
        let len = usize::try_from(self.row.record_length).unwrap();
        // SAFETY: not NULL, char and u8 have same sizeof,
        // and data owned by parent object w/liftetime 'p
        let bytes = unsafe { std::slice::from_raw_parts(self.row.record.cast::<u8>(), len) };
        std::str::from_utf8(bytes).unwrap()
    }
}
