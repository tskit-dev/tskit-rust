macro_rules! general_data_body {
    ($self: expr, $name: ident, $len: ident, $cast: ty) => {
        if $self.row.$len > 0 {
            let size = usize::try_from($self.row.$len).unwrap();
            assert!(!$self.row.$name.is_null());
            Some(unsafe { std::slice::from_raw_parts($self.row.$name.cast::<$cast>(), size) })
        } else {
            None
        }
    };
}

macro_rules! metadata_body {
    ($self: expr) => {
        general_data_body!($self, metadata, metadata_length, u8)
    };
}

#[cfg(feature = "provenance")]
macro_rules! data_to_str {
    ($self: expr, $name: ident, $len: ident) => {{
        assert!($self.row.$len > 0);
        assert!(!$self.row.$name.is_null());
        let len = usize::try_from($self.row.$len).unwrap();
        // SAFETY: not NULL, char and u8 have same sizeof,
        // and data owned by parent object w/liftetime 'p
        let bytes = unsafe { std::slice::from_raw_parts($self.row.$name.cast::<u8>(), len) };
        std::str::from_utf8(bytes).unwrap()
    }};
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
        general_data_body!(self, ancestral_state, ancestral_state_length, u8)
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
        general_data_body!(self, derived_state, derived_state_length, u8)
    }

    /// Inherited state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no inherited state
    /// * `Some(data)` if inherited state is present
    // NOTE: not populated by tsk_mutation_table_get_row_unsafe
    pub fn inherited_state(&self) -> Option<&[u8]> {
        general_data_body!(self, inherited_state, inherited_state_length, u8)
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
        general_data_body!(self, ancestral_state, ancestral_state_length, u8)
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
        general_data_body!(self, derived_state, derived_state_length, u8)
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
    pub fn id(&self) -> super::newtypes::MigrationId {
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
        general_data_body!(self, location, location_length, super::newtypes::Location)
    }

    /// Individual parents
    pub fn parents(&self) -> Option<&[super::newtypes::IndividualId]> {
        general_data_body!(self, parents, parents_length, super::newtypes::IndividualId)
    }

    /// Individual nodes
    pub fn nodes(&self) -> Option<&[super::newtypes::NodeId]> {
        general_data_body!(self, nodes, nodes_length, super::newtypes::NodeId)
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
        data_to_str!(self, timestamp, timestamp_length)
    }

    /// Provenance record
    pub fn record(&self) -> &str {
        data_to_str!(self, record, record_length)
    }
}

// TODO: use comments to document
// the testing strategy

#[cfg(test)]
mod test_row_type_wrappers {
    use super::super::bindings::*;
    use super::super::newtypes::*;

    #[derive(Clone, Default)]
    struct MutationMockData {
        id: Option<tsk_id_t>,
        site: Option<tsk_id_t>,
        node: Option<tsk_id_t>,
        time: Option<f64>,
        edge: Option<tsk_id_t>,
        metadata: Option<Vec<u8>>,
        derived_state: Option<Vec<u8>>,
        inherited_state: Option<Vec<u8>>,
    }

    struct MutationMock {
        _metadata: Vec<u8>,
        _derived_state: Vec<u8>,
        _inherited_state: Vec<u8>,
        mutation: tsk_mutation_t,
    }

    impl MutationMock {
        fn new(input: MutationMockData) -> Self {
            let mut mutation =
                unsafe { std::mem::MaybeUninit::<tsk_mutation_t>::zeroed().assume_init() };
            mutation.id = input.id.unwrap_or_default();
            mutation.site = input.site.unwrap_or_default();
            mutation.edge = input.edge.unwrap_or_default();
            mutation.node = input.node.unwrap_or_default();
            mutation.time = input.time.unwrap_or_default();
            let _metadata = input.metadata.unwrap_or_default();
            let _inherited_state = input.inherited_state.unwrap_or_default();
            let _derived_state = input.derived_state.unwrap_or_default();
            if !_metadata.is_empty() {
                mutation.metadata = _metadata.as_ptr().cast::<libc::c_char>();
                mutation.metadata_length = _metadata.len() as tsk_size_t;
            }
            if !_derived_state.is_empty() {
                mutation.derived_state = _derived_state.as_ptr().cast::<libc::c_char>();
                mutation.derived_state_length = _derived_state.len() as tsk_size_t;
            }
            if !_inherited_state.is_empty() {
                mutation.inherited_state = _inherited_state.as_ptr().cast::<libc::c_char>();
                mutation.inherited_state_length = _inherited_state.len() as tsk_size_t;
            }
            MutationMock {
                _metadata,
                _derived_state,
                _inherited_state,
                mutation,
            }
        }

        fn mutation(&self) -> super::Mutation<'_, Self> {
            super::Mutation {
                row: self.mutation,
                marker: std::marker::PhantomData,
            }
        }

        fn mutation_ref(&self) -> super::MutationRef<'_, Self> {
            super::MutationRef {
                row: &self.mutation,
                marker: std::marker::PhantomData,
            }
        }
    }

    #[test]
    fn test_mutation_with_arrays_null() {
        let mutation_test = MutationMock::new(MutationMockData {
            id: Some(10),
            site: Some(11),
            node: Some(12),
            edge: Some(-1),
            time: Some(50.),
            ..Default::default()
        });
        let mutation = mutation_test.mutation();
        assert_eq!(mutation.id(), MutationId::from(10));
        assert_eq!(mutation.site(), SiteId::from(11));
        assert_eq!(mutation.node(), NodeId::from(12));
        assert_eq!(mutation.edge(), EdgeId::NULL);
        assert_eq!(mutation.time(), Time::from(50.));
        assert!(mutation.metadata().is_none());
        assert!(mutation.derived_state().is_none());

        let mutation_ref = mutation_test.mutation_ref();
        assert_eq!(mutation_ref.id(), MutationId::from(10));
        assert_eq!(mutation_ref.site(), SiteId::from(11));
        assert_eq!(mutation_ref.node(), NodeId::from(12));
        assert_eq!(mutation_ref.edge(), EdgeId::NULL);
        assert_eq!(mutation_ref.time(), Time::from(50.));
        assert!(mutation_ref.metadata().is_none());
        assert!(mutation_ref.derived_state().is_none());
        assert!(mutation_ref.inherited_state().is_none());
    }

    #[test]
    fn test_mutation_with_arrays() {
        let mutation_test = MutationMock::new(MutationMockData {
            metadata: Some("I is metadatum".as_bytes().to_vec()),
            derived_state: Some("G".as_bytes().to_vec()),
            inherited_state: Some("C".as_bytes().to_vec()),
            ..Default::default()
        });
        let mutation = mutation_test.mutation();
        assert_eq!(mutation.metadata(), Some("I is metadatum".as_bytes()));
        assert_eq!(mutation.derived_state(), Some("G".as_bytes()));

        let mutation_ref = mutation_test.mutation_ref();
        assert_eq!(mutation.metadata(), Some("I is metadatum".as_bytes()));
        assert_eq!(mutation.derived_state(), Some("G".as_bytes()));
        assert_eq!(mutation_ref.inherited_state(), Some("C".as_bytes()));
    }
}
