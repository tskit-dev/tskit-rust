use std::ffi::CString;

use super::bindings;
use super::Error;

#[repr(transparent)]
pub struct LLTreeSeq(bindings::tsk_treeseq_t);

impl LLTreeSeq {
    pub fn new(
        tables: *mut bindings::tsk_table_collection_t,
        flags: super::flags::TreeSequenceFlags,
    ) -> Result<Self, Error> {
        let mut inner = std::mem::MaybeUninit::<bindings::tsk_treeseq_t>::uninit();
        let flags = flags.bits() | bindings::TSK_TAKE_OWNERSHIP;
        match unsafe { bindings::tsk_treeseq_init(inner.as_mut_ptr(), tables, flags) } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(Self(unsafe { inner.assume_init() })),
        }
    }

    pub fn as_ref(&self) -> &bindings::tsk_treeseq_t {
        &self.0
    }

    pub fn as_ptr(&self) -> *const bindings::tsk_treeseq_t {
        &self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut bindings::tsk_treeseq_t {
        &mut self.0
    }

    pub fn simplify(
        &self,
        samples: &[bindings::tsk_id_t],
        options: super::flags::SimplificationOptions,
        idmap: *mut bindings::tsk_id_t,
    ) -> Result<Self, Error> {
        // The output is an UNINITIALIZED treeseq,
        // else we leak memory.
        let mut ts = std::mem::MaybeUninit::<bindings::tsk_treeseq_t>::uninit();
        // SAFETY: samples is not null, idmap is allowed to be.
        // self.as_ptr() is not null
        let rv = unsafe {
            bindings::tsk_treeseq_simplify(
                self.as_ptr(),
                samples.as_ptr(),
                samples.len() as bindings::tsk_size_t,
                options.bits(),
                ts.as_mut_ptr(),
                idmap,
            )
        };
        let init = unsafe { ts.assume_init() };
        if rv < 0 {
            // SAFETY: the ptr is not null
            // and tsk_treeseq_free uses safe methods
            // to clean up.
            unsafe { bindings::tsk_treeseq_free(ts.as_mut_ptr()) };
            Err(Error::Code(rv))
        } else {
            Ok(Self(init))
        }
    }

    pub fn dump(&self, filename: CString, options: bindings::tsk_flags_t) -> Result<i32, Error> {
        // SAFETY: self pointer is not null
        match unsafe { bindings::tsk_treeseq_dump(self.as_ptr(), filename.as_ptr(), options) } {
            code if code < 0 => Err(Error::Code(code)),
            code => Ok(code),
        }
    }

    pub fn num_trees(&self) -> bindings::tsk_size_t {
        // SAFETY: self pointer is not null
        unsafe { bindings::tsk_treeseq_get_num_trees(self.as_ptr()) }
    }

    pub fn num_nodes_raw(&self) -> bindings::tsk_size_t {
        assert!(!self.as_ptr().is_null());
        assert!(!unsafe { *self.as_ptr() }.tables.is_null());
        // SAFETY: none of the pointers are null
        unsafe { (*(*self.as_ptr()).tables).nodes.num_rows }
    }

    pub fn kc_distance(&self, other: &Self, lambda: f64) -> Result<f64, Error> {
        let mut kc: f64 = f64::NAN;
        let kcp: *mut f64 = &mut kc;
        // SAFETY: self pointer is not null
        match unsafe {
            bindings::tsk_treeseq_kc_distance(self.as_ptr(), other.as_ptr(), lambda, kcp)
        } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(kc),
        }
    }

    pub fn num_samples(&self) -> bindings::tsk_size_t {
        unsafe { bindings::tsk_treeseq_get_num_samples(self.as_ptr()) }
    }

    fn free(&mut self) -> Result<(), Error> {
        match unsafe { bindings::tsk_treeseq_free(self.as_mut_ptr()) } {
            code if code < 0 => Err(Error::Code(code)),
            _ => Ok(()),
        }
    }
}

impl Drop for LLTreeSeq {
    fn drop(&mut self) {
        match self.free() {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }
}
