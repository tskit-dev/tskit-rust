use crate::metadata;
use crate::metadata::MutationMetadata;
use crate::sys;
use crate::SizeType;
use crate::Time;
use crate::TskitError;
use crate::{MutationId, NodeId, SiteId};
use sys::bindings as ll_bindings;

/// An immutable view of site table.
///
/// # Examples
///
/// ```
/// use tskit::MutationTable;
///
/// let mut mutations = MutationTable::default();
/// let rowid = mutations.add_row(1, 2, 0, 1.0, None).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(mutations.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::MutationTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::MutationMetadata)]
/// #[serializer("serde_json")]
/// struct MutationMetadata {
///     value: i32,
/// }
///
/// let metadata = MutationMetadata{value: 42};
///
/// let mut mutations = MutationTable::default();
///
/// let rowid = mutations.add_row_with_metadata(0, 1, 5, 10.0, None, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match mutations.metadata::<MutationMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
/// # }
/// ```
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct MutationTable {
    table_: sys::MutationTable,
}

impl MutationTable {
    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        mutations: *mut ll_bindings::tsk_mutation_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(mutations).unwrap();
        let table_ = unsafe { sys::MutationTable::new_borrowed(ptr) };
        Ok(MutationTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_mutation_table_t {
        self.table_.as_ref()
    }

    /// Return the number of rows.
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    /// Return the ``site`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn site<M: Into<MutationId> + Copy>(&self, row: M) -> Option<SiteId> {
        self.table_.site(row.into())
    }

    /// Return the ``node`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn node<M: Into<MutationId> + Copy>(&self, row: M) -> Option<NodeId> {
        self.table_.node(row.into())
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent<M: Into<MutationId> + Copy>(&self, row: M) -> Option<MutationId> {
        self.table_.parent(row.into())
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time<M: Into<MutationId> + Copy>(&self, row: M) -> Option<Time> {
        self.table_.time(row.into())
    }

    /// Get the ``derived_state`` value from row ``row`` of the table.
    ///
    /// # Return
    ///
    /// Will return `None` if there is no derived state.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn derived_state<M: Into<MutationId>>(&self, row: M) -> Option<&[u8]> {
        self.table_.derived_state(row.into())
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is valid and decoding failed.
    /// * `None` if `row` is not valid.
    ///
    /// # Errors
    ///
    /// * [`TskitError::MetadataError`] if decoding fails.
    ///
    /// # Examples.
    ///
    /// The big-picture semantics are the same for all table types.
    /// See [`crate::IndividualTable::metadata`] for examples.
    pub fn metadata<T: metadata::MutationMetadata>(
        &self,
        row: impl Into<MutationId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`crate::Mutation`].
    pub fn iter(&self) -> impl Iterator<Item = crate::Mutation<'_, crate::sys::MutationTable>> {
        self.table_.iter()
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row)` if `r` is valid
    /// * `None` otherwise
    pub fn row<M: Into<MutationId> + Copy>(
        &self,
        r: M,
    ) -> Option<crate::Mutation<'_, crate::sys::MutationTable>> {
        self.table_.row(r.into())
    }

    build_table_column_slice_getter!(
        /// Get the node column as a slice
        => node, node_slice, NodeId);
    build_table_column_slice_getter!(
        /// Get the node column as a slice
        => node, node_slice_raw, crate::sys::bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the site column as a slice
        => site, site_slice, SiteId);
    build_table_column_slice_getter!(
        /// Get the site column as a slice
        => site, site_slice_raw, crate::sys::bindings::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice, Time);
    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice_raw, f64);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice
        => parent, parent_slice, MutationId);
    build_table_column_slice_getter!(
        /// Get the parent column as a slice
        => parent, parent_slice_raw, crate::sys::bindings::tsk_id_t);

    pub fn node_column(&self) -> impl crate::TableColumn<MutationId, NodeId> + '_ {
        crate::table_column::OpaqueTableColumn(self.node_slice())
    }

    pub fn site_column(&self) -> impl crate::TableColumn<MutationId, SiteId> + '_ {
        crate::table_column::OpaqueTableColumn(self.site_slice())
    }

    pub fn time_column(&self) -> impl crate::TableColumn<MutationId, Time> + '_ {
        crate::table_column::OpaqueTableColumn(self.time_slice())
    }

    pub fn parent_column(&self) -> impl crate::TableColumn<MutationId, MutationId> + '_ {
        crate::table_column::OpaqueTableColumn(self.parent_slice())
    }

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }

    pub fn add_row<S, N, P, T>(
        &mut self,
        site: S,
        node: N,
        parent: P,
        time: T,
        derived_state: Option<&[u8]>,
    ) -> Result<MutationId, TskitError>
    where
        S: Into<SiteId>,
        N: Into<NodeId>,
        P: Into<MutationId>,
        T: Into<Time>,
    {
        let rv = self.table_.add_row(
            site.into().into(),
            node.into().into(),
            parent.into().into(),
            time.into().into(),
            derived_state,
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row_with_metadata<S, N, P, T, M>(
        &mut self,
        site: S,
        node: N,
        parent: P,
        time: T,
        derived_state: Option<&[u8]>,
        metadata: &M,
    ) -> Result<MutationId, TskitError>
    where
        S: Into<SiteId>,
        N: Into<NodeId>,
        P: Into<MutationId>,
        T: Into<Time>,
        M: MutationMetadata,
    {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let rv = self.table_.add_row_with_metadata(
            site.into().into(),
            node.into().into(),
            parent.into().into(),
            time.into().into(),
            derived_state,
            md.as_slice(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }
}
