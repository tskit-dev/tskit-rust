use std::ffi::CString;

use super::bindings::tsk_treeseq_init;

use super::bindings;
use super::tskbox::TskBox;
use super::TskitError;

#[repr(transparent)]
pub struct TreeSequence(TskBox<bindings::tsk_treeseq_t>);

/// The representation of a mutation as stored in a tree sequence.
#[repr(transparent)]
pub struct Mutation<'treeseq>(&'treeseq bindings::tsk_mutation_t);

impl<'treeseq> Mutation<'treeseq> {
    /// Mutation id
    pub fn id(&self) -> crate::MutationId {
        self.0.id.into()
    }
    /// Site id of this mutation
    pub fn site(&self) -> crate::SiteId {
        self.0.site.into()
    }

    /// Node id of this mutation
    pub fn node(&self) -> crate::NodeId {
        self.0.node.into()
    }

    /// Parent mutation of this mutation
    pub fn parent(&self) -> crate::MutationId {
        self.0.parent.into()
    }

    /// Origin time of mutation
    pub fn time(&self) -> crate::Time {
        self.0.time.into()
    }

    /// Edge of mutation
    // NOTE: this will be NULL when populated
    // by tsk_mutation_table_get_row_unsafe
    pub fn edge(&self) -> crate::EdgeId {
        self.0.edge.into()
    }

    /// Metadata
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no metadata
    /// * `Some(data)` if metadata are present
    pub fn metadata(&self) -> Option<&[u8]> {
        if self.0.metadata_length > 0 {
            let metadata_size = usize::try_from(self.0.metadata_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.0.metadata.cast::<u8>(),
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
        if self.0.derived_state_length > 0 {
            let derived_state_size = usize::try_from(self.0.derived_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.0.derived_state.cast::<u8>(),
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
        if self.0.inherited_state_length > 0 {
            let inherited_state_size = usize::try_from(self.0.inherited_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.0.inherited_state.cast::<u8>(),
                    inherited_state_size,
                ))
            }
        } else {
            None
        }
    }
}

/// The representation of a site as stored in a tree sequence.
#[repr(transparent)]
pub struct Site<'treeseq>(&'treeseq bindings::tsk_site_t);

impl<'treeseq> Site<'treeseq> {
    /// Row id
    pub fn id(&self) -> crate::SiteId {
        self.0.id.into()
    }
    /// Position
    pub fn position(&self) -> crate::Position {
        self.0.position.into()
    }

    /// Return iterator over [`Mutation`] at this site.
    /// Iteration order is identical to internal storage order.
    // NOTE: not populated by tsk_site_table_get_row,
    // which leaves the pointer NULL!
    pub fn mutations(&self) -> impl Iterator<Item = Mutation<'treeseq>> {
        assert!(!self.0.mutations.is_null());
        let mslice = unsafe {
            std::slice::from_raw_parts(self.0.mutations, self.0.mutations_length as usize)
        };
        mslice.iter().map(Mutation)
    }

    /// Ancestral state
    ///
    /// # Return
    ///
    /// * `None` if the mutation has no ancestral state
    /// * `Some(data)` if ancestral state is present
    pub fn ancestral_state(&self) -> Option<&[u8]> {
        if self.0.ancestral_state_length > 0 {
            let ancestral_state_size = usize::try_from(self.0.ancestral_state_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.0.ancestral_state.cast::<u8>(),
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
        if self.0.metadata_length > 0 {
            let metadata_size = usize::try_from(self.0.metadata_length).unwrap();
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.0.metadata.cast::<u8>(),
                    metadata_size,
                ))
            }
        } else {
            None
        }
    }
}

impl TreeSequence {
    pub fn new(
        tables: super::TableCollection,
        flags: super::flags::TreeSequenceFlags,
    ) -> Result<Self, TskitError> {
        let tables = tables.into_raw();
        let inner = TskBox::new(|t: *mut bindings::tsk_treeseq_t| unsafe {
            tsk_treeseq_init(t, tables, flags.bits() | bindings::TSK_TAKE_OWNERSHIP)
        })?;
        Ok(Self(inner))
    }

    pub fn as_ref(&self) -> &bindings::tsk_treeseq_t {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> &mut bindings::tsk_treeseq_t {
        self.0.as_mut()
    }

    pub fn simplify(
        &self,
        samples: &[super::newtypes::NodeId],
        options: super::flags::SimplificationOptions,
        idmap: Option<&mut [super::newtypes::NodeId]>,
    ) -> Result<Self, TskitError> {
        // The output is an UNINITIALIZED treeseq,
        // else we leak memory.
        let mut ts = unsafe { TskBox::new_uninit() };
        // SAFETY: samples is not null, idmap is allowed to be.
        // self.as_ptr() is not null
        let rv = unsafe {
            bindings::tsk_treeseq_simplify(
                self.as_ref(),
                // The cast is safe/sound b/c NodeId is repr(transparent)
                samples.as_ptr().cast::<_>(),
                samples.len().try_into().unwrap(),
                options.bits(),
                ts.as_mut_ptr(),
                match idmap {
                    Some(s) => s.as_mut_ptr().cast::<_>(),
                    None => std::ptr::null_mut(),
                },
            )
        };
        if rv < 0 {
            // SAFETY: the ptr is not null
            // and tsk_treeseq_free uses safe methods
            // to clean up.
            unsafe { bindings::tsk_treeseq_free(ts.as_mut_ptr()) };
            Err(TskitError::ErrorCode { code: rv })
        } else {
            Ok(Self(ts))
        }
    }

    pub fn dump(
        &self,
        filename: CString,
        options: bindings::tsk_flags_t,
    ) -> Result<i32, TskitError> {
        // SAFETY: self pointer is not null
        match unsafe { bindings::tsk_treeseq_dump(self.as_ref(), filename.as_ptr(), options) } {
            code if code < 0 => Err(TskitError::ErrorCode { code }),
            code => Ok(code),
        }
    }

    pub fn num_trees(&self) -> super::newtypes::SizeType {
        // SAFETY: self pointer is not null
        unsafe { bindings::tsk_treeseq_get_num_trees(self.as_ref()) }.into()
    }

    pub fn num_nodes_raw(&self) -> bindings::tsk_size_t {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: none of the pointers are null
        unsafe { (*(self.as_ref()).tables).nodes.num_rows }
    }

    fn num_edges_raw(&self) -> bindings::tsk_size_t {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: none of the pointers are null
        unsafe { (*(self.as_ref()).tables).edges.num_rows }
    }

    pub fn kc_distance(&self, other: &Self, lambda: f64) -> Result<f64, TskitError> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        // SAFETY: self pointer is not null
        match unsafe {
            bindings::tsk_treeseq_kc_distance(self.as_ref(), other.as_ref(), lambda, kcp)
        } {
            code if code < 0 => Err(TskitError::ErrorCode { code }),
            _ => Ok(kc),
        }
    }

    pub fn num_samples(&self) -> super::newtypes::SizeType {
        unsafe { bindings::tsk_treeseq_get_num_samples(self.as_ref()) }.into()
    }

    pub fn edge_insertion_order(&self) -> &[super::newtypes::EdgeId] {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: all array lengths are the number of rows in the table
        // SAFETY: no pointers can be null
        // SAFETY: tables are indexed in order to create a treeseq
        unsafe {
            super::generate_slice(
                (*self.as_ref().tables).indexes.edge_insertion_order,
                self.num_edges_raw(),
            )
        }
    }

    pub fn edge_removal_order(&self) -> &[super::newtypes::EdgeId] {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: all array lengths are the number of rows in the table
        // SAFETY: no pointers can be null
        // SAFETY: tables are indexed in order to create a treeseq
        unsafe {
            super::generate_slice(
                (*self.as_ref().tables).indexes.edge_removal_order,
                self.num_edges_raw(),
            )
        }
    }

    pub fn site<'ts>(&'ts self, site: bindings::tsk_id_t) -> Option<Site<'ts>> {
        let num_sites = unsafe { (*(self.as_ref()).tables).sites.num_rows };
        assert!(!self.as_ref().tree_sites_mem.is_null());
        let sites =
            unsafe { std::slice::from_raw_parts(self.as_ref().tree_sites_mem, num_sites as usize) };
        sites.get(site as usize).map(Site)
    }

    pub fn site_iter<'ts>(&'ts self) -> impl Iterator<Item = Site<'ts>> {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: none of the pointers are null
        let num_sites = unsafe { (*(self.as_ref()).tables).sites.num_rows };
        assert!(!self.as_ref().tree_sites_mem.is_null());
        let sites =
            unsafe { std::slice::from_raw_parts(self.as_ref().tree_sites_mem, num_sites as usize) };
        sites.iter().map(Site)
    }
}
