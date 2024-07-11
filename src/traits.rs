//! Traits related to user-facing types

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
impl_individual_parents!(&[crate::sys::bindings::tsk_id_t], self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(&Vec<crate::sys::bindings::tsk_id_t>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(Vec<crate::sys::bindings::tsk_id_t>, self, unsafe {
    std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len())
});
impl_individual_parents!(
    N,
    usize,
    &[crate::sys::bindings::tsk_id_t; N],
    self,
    unsafe { std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len()) }
);
impl_individual_parents!(
    N,
    usize,
    [crate::sys::bindings::tsk_id_t; N],
    self,
    unsafe { std::slice::from_raw_parts(self.as_ptr() as *const crate::IndividualId, self.len()) }
);
impl_individual_parents!(N, usize, &[crate::IndividualId; N], self, self.as_slice());
impl_individual_parents!(N, usize, [crate::IndividualId; N], self, self.as_slice());

pub trait TableAccess {
    fn edges(&self) -> &crate::EdgeTable;
    fn nodes(&self) -> &crate::NodeTable;
    fn sites(&self) -> &crate::SiteTable;
    fn mutations(&self) -> &crate::MutationTable;
    fn migrations(&self) -> &crate::MigrationTable;
}

impl TableAccess for crate::TableCollection {
    fn edges(&self) -> &crate::EdgeTable {
        self.edges()
    }

    fn nodes(&self) -> &crate::NodeTable {
        self.nodes()
    }

    fn sites(&self) -> &crate::SiteTable {
        self.sites()
    }

    fn mutations(&self) -> &crate::MutationTable {
        self.mutations()
    }

    fn migrations(&self) -> &crate::MigrationTable {
        self.migrations()
    }
}

impl TableAccess for crate::TreeSequence {
    fn edges(&self) -> &crate::EdgeTable {
        self.tables().edges()
    }

    fn nodes(&self) -> &crate::NodeTable {
        self.tables().nodes()
    }

    fn sites(&self) -> &crate::SiteTable {
        self.tables().sites()
    }

    fn mutations(&self) -> &crate::MutationTable {
        self.tables().mutations()
    }

    fn migrations(&self) -> &crate::MigrationTable {
        self.tables().migrations()
    }
}
