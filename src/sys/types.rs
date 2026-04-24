/// Reference to a site stored in a tree sequence.
#[repr(transparent)]
pub struct SiteRef<'p, P> {
    site: &'p super::bindings::tsk_site_t,
    marker: std::marker::PhantomData<&'p P>,
}

pub fn new_site_ref<'p, P>(
    _parent: &'p P,
    site: &'p super::bindings::tsk_site_t,
) -> SiteRef<'p, P> {
    SiteRef {
        site,
        marker: std::marker::PhantomData,
    }
}

impl<'parent, P> SiteRef<'parent, P> {
    /// Row id
    pub fn id(&self) -> crate::SiteId {
        self.site.id.into()
    }
    /// Position
    pub fn position(&self) -> crate::Position {
        self.site.position.into()
    }

    /// Return iterator over [`MutationRef`] at this site.
    /// Iteration order is identical to internal storage order.
    // NOTE: not populated by tsk_site_table_get_row,
    // which leaves the pointer NULL!
    pub fn mutations(
        &self,
    ) -> impl Iterator<Item = MutationRef<'parent, super::bindings::tsk_mutation_t>> {
        assert!(!self.site.mutations.is_null());
        let mslice = unsafe {
            std::slice::from_raw_parts(self.site.mutations, self.site.mutations_length as usize)
        };
        mslice.iter().map(|m| MutationRef {
            mutation: m,
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
        if self.site.ancestral_state_length > 0 {
            let ancestral_state_size = usize::try_from(self.site.ancestral_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.site.ancestral_state.cast::<u8>(),
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
    /// * `None` if the site has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        if self.site.metadata_length > 0 {
            let metadata_size = usize::try_from(self.site.metadata_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.site.metadata.cast::<u8>(),
                    metadata_size,
                ))
            }
        } else {
            None
        }
    }
}

/// Reference to a mutation stored in a tree sequence.
#[repr(transparent)]
pub struct MutationRef<'p, P> {
    mutation: &'p super::bindings::tsk_mutation_t,
    marker: std::marker::PhantomData<&'p P>,
}

impl<'parent, P> MutationRef<'parent, P> {
    /// Mutation id
    pub fn id(&self) -> crate::MutationId {
        self.mutation.id.into()
    }
    /// Site id of this mutation
    pub fn site(&self) -> crate::SiteId {
        self.mutation.site.into()
    }

    /// Node id of this mutation
    pub fn node(&self) -> crate::NodeId {
        self.mutation.node.into()
    }

    /// Parent mutation of this mutation
    pub fn parent(&self) -> crate::MutationId {
        self.mutation.parent.into()
    }

    /// Origin time of mutation
    pub fn time(&self) -> crate::Time {
        self.mutation.time.into()
    }

    /// Edge of mutation
    // NOTE: this will be NULL when populated
    // by tsk_mutation_table_get_row_unsafe
    pub fn edge(&self) -> crate::EdgeId {
        self.mutation.edge.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        if self.mutation.metadata_length > 0 {
            let metadata_size = usize::try_from(self.mutation.metadata_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.mutation.metadata.cast::<u8>(),
                    metadata_size,
                ))
            }
        } else {
            None
        }
    }

    /// Derived state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no derived state
    /// * `Some(data)` if derived state is present
    pub fn derived_state(&self) -> Option<&[u8]> {
        if self.mutation.derived_state_length > 0 {
            let derived_state_size = usize::try_from(self.mutation.derived_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.mutation.derived_state.cast::<u8>(),
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
        if self.mutation.inherited_state_length > 0 {
            let inherited_state_size =
                usize::try_from(self.mutation.inherited_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.mutation.inherited_state.cast::<u8>(),
                    inherited_state_size,
                ))
            }
        } else {
            None
        }
    }
}
