macro_rules! general_data_body {
    ($self: expr, $field: ident, $name: ident, $len: ident, $cast: ty) => {
        if $self.$field.$len > 0 {
            let size = usize::try_from($self.$field.$len).unwrap();
            assert!(!$self.$field.$name.is_null());
            Some(unsafe { std::slice::from_raw_parts($self.$field.$name.cast::<$cast>(), size) })
        } else {
            None
        }
    };
    ($self: expr, $name: ident, $len: ident, $cast: ty) => {
        if $self.0.$len > 0 {
            let size = usize::try_from($self.0.$len).unwrap();
            assert!(!$self.0.$name.is_null());
            Some(unsafe { std::slice::from_raw_parts($self.0.$name.cast::<$cast>(), size) })
        } else {
            None
        }
    };
}

macro_rules! metadata_body {
    ($self: expr, $field: ident) => {
        general_data_body!($self, $field, metadata, metadata_length, u8)
    };
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
#[repr(transparent)]
pub struct SiteRef<'p>(&'p super::bindings::tsk_site_t);

impl<'p> std::cmp::PartialEq for SiteRef<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.position().eq(&other.position())
            && self.ancestral_state().eq(&other.ancestral_state())
            && self.metadata().eq(&other.metadata())
            // NOTE: the .eq() below is Iterator::eq
            && self.mutation_iter().eq(other.mutation_iter())
    }
}

pub(super) fn new_site_ref<'p>(site: &'p super::bindings::tsk_site_t) -> SiteRef<'p> {
    SiteRef(site)
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct MutationRefIterator<'ts>(&'ts [super::bindings::tsk_mutation_t]);

impl<'ts> Iterator for MutationRefIterator<'ts> {
    type Item = super::MutationRef<'ts>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.0.split_first() {
            self.0 = r;
            Some(MutationRef(l))
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0 = if n < self.0.len() { &self.0[n..] } else { &[] };
        self.next()
    }
}

impl<'ts> DoubleEndedIterator for MutationRefIterator<'ts> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.0.split_last() {
            self.0 = r;
            Some(MutationRef(l))
        } else {
            None
        }
    }
}

impl<'parent> SiteRef<'parent> {
    /// Row id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::SiteId {
        self.0.id.into()
    }
    /// Position
    #[inline(always)]
    pub fn position(&self) -> super::newtypes::Position {
        self.0.position.into()
    }

    /// Return iterator over [`MutationRef`] at this site.
    /// Iteration order is identical to internal storage order.
    // NOTE: not populated by tsk_site_table_get_row,
    // which leaves the pointer NULL!
    pub fn mutation_iter(&self) -> impl DoubleEndedIterator<Item = MutationRef<'parent>> + Clone {
        assert!(!self.0.mutations.is_null());
        let mslice = unsafe {
            std::slice::from_raw_parts(self.0.mutations, self.0.mutations_length as usize)
        };
        MutationRefIterator(mslice)
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
#[repr(transparent)]
pub struct MutationRef<'p>(&'p super::bindings::tsk_mutation_t);

impl<'p> std::cmp::PartialEq for MutationRef<'p> {
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

impl<'parent> MutationRef<'parent> {
    /// Mutation id
    #[inline(always)]
    pub fn id(&self) -> super::newtypes::MutationId {
        self.0.id.into()
    }
    /// Site id of this mutation
    #[inline(always)]
    pub fn site(&self) -> super::newtypes::SiteId {
        self.0.site.into()
    }

    /// Node id of this mutation
    #[inline(always)]
    pub fn node(&self) -> super::newtypes::NodeId {
        self.0.node.into()
    }

    /// Parent mutation of this mutation
    #[inline(always)]
    pub fn parent(&self) -> super::newtypes::MutationId {
        self.0.parent.into()
    }

    /// Origin time of mutation
    #[inline(always)]
    pub fn time(&self) -> super::newtypes::Time {
        self.0.time.into()
    }

    /// Edge of mutation
    // NOTE: this will be NULL when populated
    // by tsk_mutation_table_get_row_unsafe
    #[inline(always)]
    pub fn edge(&self) -> super::newtypes::EdgeId {
        self.0.edge.into()
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
pub struct Site<'p> {
    pub(super) row: super::bindings::tsk_site_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Site<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.position().eq(&other.position())
            && self.ancestral_state().eq(&other.ancestral_state())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p> Site<'p> {
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
        metadata_body!(self, row)
    }

    /// Ancestral state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no ancestral state
    /// * `Some(data)` if ancestral state is present
    pub fn ancestral_state(&self) -> Option<&[u8]> {
        general_data_body!(self, row, ancestral_state, ancestral_state_length, u8)
    }

    /// Position
    #[inline(always)]
    pub fn position(&self) -> super::newtypes::Position {
        self.row.position.into()
    }
}

/// A lifetime-bound Mutation.
#[derive(Debug)]
pub struct Mutation<'p> {
    pub(super) row: super::bindings::tsk_mutation_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Mutation<'p> {
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

impl<'p> Mutation<'p> {
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
        metadata_body!(self, row)
    }

    /// Derived state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no derived state
    /// * `Some(data)` if derived state is present
    pub fn derived_state(&self) -> Option<&[u8]> {
        general_data_body!(self, row, derived_state, derived_state_length, u8)
    }
}

/// A lifetime-bound Edge.
#[derive(Debug)]
pub struct Edge<'p> {
    pub(super) row: super::bindings::tsk_edge_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Edge<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.parent().eq(&other.parent())
            && self.child().eq(&other.child())
            && self.left().eq(&other.left())
            && self.right().eq(&other.right())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p> Edge<'p> {
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
        metadata_body!(self, row)
    }
}

/// A lifetime-bound Migration.
#[derive(Debug)]
pub struct Migration<'p> {
    pub(super) row: super::bindings::tsk_migration_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Migration<'p> {
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

impl<'p> Migration<'p> {
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
        metadata_body!(self, row)
    }
}

/// A lifetime-bound Individual.
#[derive(Debug)]
pub struct Individual<'p> {
    pub(super) row: super::bindings::tsk_individual_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Individual<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.flags().eq(&other.flags())
            && self.location().eq(&other.location())
            && self.parents().eq(&other.parents())
            && self.metadata().eq(&other.metadata())
            && self.nodes().eq(&other.nodes())
    }
}

#[repr(transparent)]
struct IndividualNodeIdIterator<'ind>(&'ind [super::newtypes::NodeId]);

impl<'ind> Iterator for IndividualNodeIdIterator<'ind> {
    type Item = super::newtypes::NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.0.split_first() {
            self.0 = r;
            Some(*l)
        } else {
            None
        }
    }
}

impl<'p> Individual<'p> {
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
        metadata_body!(self, row)
    }

    /// Individual location
    pub fn location(&self) -> Option<&[super::newtypes::Location]> {
        general_data_body!(
            self,
            row,
            location,
            location_length,
            super::newtypes::Location
        )
    }

    /// Individual parents
    pub fn parents(&self) -> Option<&[super::newtypes::IndividualId]> {
        general_data_body!(
            self,
            row,
            parents,
            parents_length,
            super::newtypes::IndividualId
        )
    }

    /// Individual nodes
    pub fn nodes(&self) -> Option<&[super::newtypes::NodeId]> {
        general_data_body!(self, row, nodes, nodes_length, super::newtypes::NodeId)
    }

    /// Convert into an iterator over node ids.
    ///
    /// One use of this function is to flatten an iterator over individuals
    /// into an iterator over node ids from those individuals.
    pub fn into_node_id_iter<'iter>(self) -> impl Iterator<Item = super::newtypes::NodeId> + 'iter
    where
        'p: 'iter,
    {
        let nodes = if self.row.nodes_length > 0 {
            assert!(!self.row.nodes.is_null());
            // SAFETY: the pointer is not null.
            // The cast works b/c NodeId is a transparent newtype for tsk_id_t.
            unsafe {
                std::slice::from_raw_parts(
                    self.row.nodes.cast::<super::newtypes::NodeId>(),
                    self.row.nodes_length as usize,
                )
            }
        } else {
            &[]
        };

        IndividualNodeIdIterator(nodes)
    }
}

/// A lifetime-bound Node.
#[derive(Debug)]
pub struct Node<'p> {
    pub(super) row: super::bindings::tsk_node_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Node<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.flags().eq(&other.flags())
            && self.time().eq(&other.time())
            && self.population().eq(&other.population())
            && self.individual().eq(&other.individual())
            && self.metadata().eq(&other.metadata())
    }
}

impl<'p> Node<'p> {
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
        self.row.individual.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the object has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        metadata_body!(self, row)
    }
}

/// A lifetime-bound Population.
#[derive(Debug)]
pub struct Population<'p> {
    pub(super) row: super::bindings::tsk_population_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

impl<'p> std::cmp::PartialEq for Population<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id()) && self.metadata().eq(&other.metadata())
    }
}

impl<'p> Population<'p> {
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
        metadata_body!(self, row)
    }
}

#[derive(Debug)]
#[cfg(feature = "provenance")]
pub struct Provenance<'p> {
    pub(super) row: super::bindings::tsk_provenance_t,
    pub(super) marker: std::marker::PhantomData<&'p ()>,
}

#[cfg(feature = "provenance")]
impl<'p> std::cmp::PartialEq for Provenance<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
            && self.timestamp().eq(other.timestamp())
            && self.record().eq(other.record())
    }
}

#[cfg(feature = "provenance")]
impl<'p> Provenance<'p> {
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

#[test]
fn test_transmute() {
    let mut mutation =
        unsafe { std::mem::MaybeUninit::<super::bindings::tsk_mutation_t>::zeroed().assume_init() };
    mutation.site = 11;
    mutation.edge = -1;
    let mref =
        unsafe { std::mem::transmute::<super::bindings::tsk_mutation_t, Mutation<'_>>(mutation) };
    assert_eq!(mref.id(), 0);
    assert_eq!(mref.site(), 11);
    assert_eq!(mref.edge(), -1);
    let mref2 = super::Mutation::<'_> {
        row: mutation,
        marker: std::marker::PhantomData,
    };
    assert_eq!(mref, mref2);
}

// NOTE: the tests below accomplish a lot.
// The use of transmute check that our wrapper types
// compile down to the same layout as the C types.
// (If not, we'd get segfaults, failing asserts, etc.,
// during testing.)
// Calling the member functions and asserting equality
// to specific new types makes sure that we are calling
// the correct internal field AND converting to the correct type.

// TODO: we need to add tests for all of the wrappers.
#[cfg(test)]
mod test_row_type_wrappers {
    use super::super::bindings::*;
    use super::super::newtypes::*;

    macro_rules! sizeof_and_layout {
        ($a: ty, $b: ty) => {
            assert_eq!(std::mem::size_of::<$a>(), std::mem::size_of::<$b>());
            let ll_layout = std::alloc::Layout::new::<$a>();
            let wrapper_layout = std::alloc::Layout::new::<$b>();
            assert_eq!(ll_layout, wrapper_layout);
        };
    }

    macro_rules! set_scalar_field {
        ($x: ident, $($field: ident),* ; $($id: literal),*)=> {
            $($x.$field = $id;)*
        };
    }

    // NOTE: for this macro to work,
    // x.field must be set equal to the pointer
    // in a local variable ALSO called field
    macro_rules! set_pointer_field {
        ($x: ident, $($field: ident),* ; $($length: ident),* ; $($cast: ty),*) => {
            $(
                $x.$field = $field.as_ptr().cast::<$cast>();
                $x.$length = u64::try_from($field.len()).unwrap();
            )*
        };
    }

    #[test]
    fn test_size_of_and_layout() {
        sizeof_and_layout!(tsk_mutation_t, super::Mutation<'_>);
        // NOTE: the generic "parent" type has no effect
        sizeof_and_layout!(tsk_mutation_t, super::Mutation<'_>);
        sizeof_and_layout!(&tsk_mutation_t, super::MutationRef<'_>);
        sizeof_and_layout!(tsk_site_t, super::Site<'_>);
        sizeof_and_layout!(&tsk_site_t, super::SiteRef<'_>);
        sizeof_and_layout!(tsk_edge_t, super::Edge<'_>);
        sizeof_and_layout!(tsk_node_t, super::Node<'_>);
        sizeof_and_layout!(tsk_individual_t, super::Individual<'_>);
        sizeof_and_layout!(tsk_migration_t, super::Migration<'_>);
        #[cfg(feature = "provenance")]
        sizeof_and_layout!(tsk_provenance_t, super::Provenance<'_>);
    }

    #[test]
    fn test_tsk_mutation_t_wrappers() {
        let mut ll_mutation =
            unsafe { std::mem::MaybeUninit::<tsk_mutation_t>::zeroed().assume_init() };
        set_scalar_field!(ll_mutation, id, site, node, edge, time ; 10, 11, 12, -1, 50.);
        let mutation =
            unsafe { std::mem::transmute::<tsk_mutation_t, super::Mutation<'_>>(ll_mutation) };
        assert_eq!(mutation, mutation);
        assert_eq!(mutation.id(), MutationId::from(10));
        assert_eq!(mutation.site(), SiteId::from(11));
        assert_eq!(mutation.node(), NodeId::from(12));
        assert_eq!(mutation.edge(), EdgeId::NULL);
        assert_eq!(mutation.time(), Time::from(50.));
        assert!(mutation.metadata().is_none());
        assert!(mutation.derived_state().is_none());

        // make a copy, modify it, make sure !=
        {
            let mut ll_mutation2 = ll_mutation;
            set_scalar_field!(ll_mutation2, site; 100);
            let mutation2 =
                unsafe { std::mem::transmute::<tsk_mutation_t, super::Mutation<'_>>(ll_mutation2) };
            assert_ne!(mutation, mutation2);
        }

        // populate the array data types
        {
            let metadata = b"I is metadatum".to_vec();
            let derived_state = b"G".to_vec();
            let inherited_state = b"CAGT".to_vec();
            set_pointer_field!(ll_mutation, metadata, derived_state, inherited_state ;
                metadata_length, derived_state_length, inherited_state_length ;
                libc::c_char, libc::c_char, libc::c_char);
            let mutation =
                unsafe { std::mem::transmute::<tsk_mutation_t, super::Mutation<'_>>(ll_mutation) };
            assert_eq!(mutation.metadata(), Some(metadata.as_slice()));
            assert_eq!(mutation.derived_state(), Some(derived_state.as_slice()));

            let mutation_ref = super::MutationRef(&ll_mutation);
            assert_eq!(mutation.metadata(), Some(metadata.as_slice()));
            assert_eq!(mutation.derived_state(), Some(derived_state.as_slice()));
            assert_eq!(
                mutation_ref.inherited_state(),
                Some(inherited_state.as_slice())
            );
        }
    }

    #[test]
    fn test_tsk_node_t_wrappers() {
        let mut ll_node = unsafe { std::mem::MaybeUninit::<tsk_node_t>::zeroed().assume_init() };
        set_scalar_field!(ll_node, individual, population ; 33, 101);
        let node = unsafe { std::mem::transmute::<tsk_node_t, super::Node<'_>>(ll_node) };
        assert_eq!(node.individual(), 33);
        assert_eq!(node.population(), 101);
    }
}
