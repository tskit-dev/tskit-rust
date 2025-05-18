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

mod private {
    pub trait NewTypeMarker: TryInto<usize, Error = crate::TskitError> {}
    pub trait TableColumnMarker {}
}

impl private::NewTypeMarker for crate::EdgeId {}
impl private::NewTypeMarker for crate::NodeId {}
impl private::NewTypeMarker for crate::SiteId {}
impl private::NewTypeMarker for crate::MutationId {}
impl private::NewTypeMarker for crate::MigrationId {}
impl private::NewTypeMarker for crate::IndividualId {}
impl private::NewTypeMarker for crate::PopulationId {}
#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
impl private::NewTypeMarker for crate::ProvenanceId {}

/// Interface of a non-ragged table column.
///
/// Unlike slice views of table columns, this API
/// allows indexed via row id types and [`crate::SizeType`].
///
/// # Notes
///
/// * This trait is sealed.
///
/// # For C programmers
///
/// The `C` programming language allows implicit casts between
/// integer types.
/// This implicit behavior allows one to index a table column
/// using a row id type ([`crate::bindings::tsk_id_t`]) because
/// the compiler will cast it to `size_t`.
///
/// `rust` does not allow implicit casts, which makes working
/// with table columns as slices awkward.
/// One has to manually cast the id type and the resulting code isn't
/// nice to read.
///
/// This trait solves that problem by requiring that [`std::ops::Index`]
/// by implemented for types that one would like to use as indexes
/// in the `tskit` world.
pub trait TableColumn<I, T>:
    std::ops::Index<I, Output = T>
    + std::ops::Index<usize, Output = T>
    + std::ops::Index<crate::SizeType, Output = T>
    + private::TableColumnMarker
{
    /// Get the underlying slice
    fn as_slice(&self) -> &[T];

    /// Get with a table row identifier such as [`crate::NodeId`]
    fn get_with_id(&self, at: I) -> Option<&T>;

    /// The "standard" get function
    fn get(&self, at: usize) -> Option<&T> {
        self.as_slice().get(at)
    }

    /// Get with [`crate::SizeType`]
    fn get_with_size_type(&self, at: crate::SizeType) -> Option<&T> {
        self.as_slice().get(usize::try_from(at).ok()?)
    }

    /// Iterator over the data.
    fn iter<'a, 'b>(&'a self) -> impl Iterator<Item = &'b T>
    where
        'a: 'b,
        T: 'b,
    {
        self.as_slice().iter()
    }

    /// Column length
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Query if column is empty
    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
}

impl<T> private::TableColumnMarker for crate::table_column::OpaqueTableColumn<'_, T> {}

impl<I, T> std::ops::Index<I> for crate::table_column::OpaqueTableColumn<'_, T>
where
    I: private::NewTypeMarker,
{
    type Output = T;
    fn index(&self, index: I) -> &Self::Output {
        &self.0[index.try_into().unwrap()]
    }
}

impl<I, T> TableColumn<I, T> for crate::table_column::OpaqueTableColumn<'_, T>
where
    I: private::NewTypeMarker,
{
    fn as_slice(&self) -> &[T] {
        self.0
    }

    fn get_with_id(&self, at: I) -> Option<&T> {
        self.0.get(at.try_into().ok()?)
    }
}
