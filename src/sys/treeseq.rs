use std::ffi::CString;

use super::bindings::tsk_treeseq_init;

use super::bindings;
use super::tskbox::TskBox;
use super::TskitError;

#[repr(transparent)]
pub struct TreeSequence(TskBox<bindings::tsk_treeseq_t>);

impl TreeSequence {
    pub fn new(
        tables: *mut bindings::tsk_table_collection_t,
        flags: super::flags::TreeSequenceFlags,
    ) -> Result<Self, TskitError> {
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
        samples: &[bindings::tsk_id_t],
        options: super::flags::SimplificationOptions,
        idmap: *mut bindings::tsk_id_t,
    ) -> Result<Self, TskitError> {
        // The output is an UNINITIALIZED treeseq,
        // else we leak memory.
        let mut ts = unsafe { TskBox::new_uninit() };
        // SAFETY: samples is not null, idmap is allowed to be.
        // self.as_ptr() is not null
        let rv = unsafe {
            bindings::tsk_treeseq_simplify(
                self.as_ref(),
                samples.as_ptr(),
                samples.len() as bindings::tsk_size_t,
                options.bits(),
                ts.as_mut_ptr(),
                idmap,
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

    pub fn num_trees(&self) -> bindings::tsk_size_t {
        // SAFETY: self pointer is not null
        unsafe { bindings::tsk_treeseq_get_num_trees(self.as_ref()) }
    }

    pub fn num_nodes_raw(&self) -> bindings::tsk_size_t {
        assert!(!self.as_ref().tables.is_null());
        // SAFETY: none of the pointers are null
        unsafe { (*(self.as_ref()).tables).nodes.num_rows }
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

    pub fn num_samples(&self) -> bindings::tsk_size_t {
        unsafe { bindings::tsk_treeseq_get_num_samples(self.as_ref()) }
    }
}
