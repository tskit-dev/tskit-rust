use std::ffi::CString;

use super::bindings::tsk_treeseq_init;

use super::bindings;
use super::tskbox::TskBox;
use super::TskitError;

#[repr(transparent)]
pub struct TreeSequence(TskBox<bindings::tsk_treeseq_t>);

pub struct TreeSeqIndividualIter<'ts> {
    treeseq: &'ts TreeSequence,
    current_row: bindings::tsk_id_t,
}

impl<'ts> Iterator for TreeSeqIndividualIter<'ts> {
    type Item = super::Individual<'ts, TreeSequence>;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.current_row;
        self.current_row += 1;
        self.treeseq.individual(r)
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

    pub fn site<'ts>(&'ts self, site: bindings::tsk_id_t) -> Option<crate::SiteRef<'ts, Self>> {
        let num_sites = unsafe { (*(self.as_ref()).tables).sites.num_rows };
        assert!(!self.as_ref().tree_sites_mem.is_null());
        let sites =
            unsafe { std::slice::from_raw_parts(self.as_ref().tree_sites_mem, num_sites as usize) };
        sites
            .get(site as usize)
            .map(|s| super::new_site_ref(self, s))
    }

    pub fn site_iter<'ts>(&'ts self) -> impl Iterator<Item = crate::SiteRef<'ts, Self>> {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: none of the pointers are null
        let num_sites = unsafe { (*(self.as_ref()).tables).sites.num_rows };
        assert!(!self.as_ref().tree_sites_mem.is_null());
        let sites =
            unsafe { std::slice::from_raw_parts(self.as_ref().tree_sites_mem, num_sites as usize) };
        sites.iter().map(|s| super::new_site_ref(self, s))
    }

    pub fn individual<'ts>(
        &'ts self,
        row: bindings::tsk_id_t,
    ) -> Option<crate::Individual<'ts, Self>> {
        let mut individual = unsafe {
            std::mem::MaybeUninit::<super::bindings::tsk_individual_t>::zeroed().assume_init()
        };
        let rv = unsafe {
            super::bindings::tsk_treeseq_get_individual(
                self.as_ref(),
                row,
                &mut individual as *mut super::bindings::tsk_individual_t,
            )
        };
        if rv == 0 {
            Some(super::Individual {
                row: individual,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn individual_iter<'ts>(
        &'ts self,
    ) -> impl Iterator<Item = super::types::Individual<'ts, Self>> {
        TreeSeqIndividualIter {
            treeseq: self,
            current_row: 0,
        }
    }
}
