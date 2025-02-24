use crate::metadata;
use crate::metadata::MutationMetadata;
use crate::sys;
use crate::SizeType;
use crate::Time;
use crate::TskitError;
use crate::{MutationId, NodeId, SiteId};
use ll_bindings::tsk_id_t;
use sys::bindings as ll_bindings;

/// Row of a [`MutationTable`]
#[derive(Debug)]
pub struct MutationTableRow {
    pub id: MutationId,
    pub site: SiteId,
    pub node: NodeId,
    pub parent: MutationId,
    pub time: Time,
    pub derived_state: Option<Vec<u8>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for MutationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.derived_state == other.derived_state
            && self.metadata == other.metadata
    }
}

fn make_mutation_table_row(table: &MutationTable, pos: tsk_id_t) -> Option<MutationTableRow> {
    let index = ll_bindings::tsk_size_t::try_from(pos).ok()?;
    match index {
        i if i < table.num_rows() => {
            let derived_state = table.derived_state(pos).map(|s| s.to_vec());
            Some(MutationTableRow {
                id: pos.into(),
                site: table.site(pos)?,
                node: table.node(pos)?,
                parent: table.parent(pos)?,
                time: table.time(pos)?,
                derived_state,
                metadata: table.raw_metadata(pos).map(|m| m.to_vec()),
            })
        }
        _ => None,
    }
}

pub(crate) type MutationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a MutationTable>;
pub(crate) type MutationTableIterator = crate::table_iterator::TableIterator<MutationTable>;

impl Iterator for MutationTableRefIterator<'_> {
    type Item = MutationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_mutation_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for MutationTableIterator {
    type Item = MutationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_mutation_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct MutationTableRowView<'a> {
    table: &'a MutationTable,
    pub id: MutationId,
    pub site: SiteId,
    pub node: NodeId,
    pub parent: MutationId,
    pub time: Time,
    pub derived_state: Option<&'a [u8]>,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> MutationTableRowView<'a> {
    fn new(table: &'a MutationTable) -> Self {
        Self {
            table,
            id: MutationId::NULL,
            site: SiteId::NULL,
            node: NodeId::NULL,
            parent: MutationId::NULL,
            time: f64::NAN.into(),
            derived_state: None,
            metadata: None,
        }
    }
}

impl PartialEq for MutationTableRowView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.derived_state == other.derived_state
            && self.metadata == other.metadata
    }
}

impl Eq for MutationTableRowView<'_> {}

impl PartialEq<MutationTableRow> for MutationTableRowView<'_> {
    fn eq(&self, other: &MutationTableRow) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.derived_state, other.derived_state)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<MutationTableRowView<'_>> for MutationTableRow {
    fn eq(&self, other: &MutationTableRowView) -> bool {
        self.id == other.id
            && self.site == other.site
            && self.node == other.node
            && self.parent == other.parent
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.derived_state, other.derived_state)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl streaming_iterator::StreamingIterator for MutationTableRowView<'_> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.site = self.table.site(self.id).unwrap_or(SiteId::NULL);
        self.node = self.table.node(self.id).unwrap_or(NodeId::NULL);
        self.parent = self.table.parent(self.id).unwrap_or(MutationId::NULL);
        self.time = self.table.time(self.id).unwrap_or_else(|| f64::NAN.into());
        self.derived_state = self.table.derived_state(self.id);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

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
    pub(crate) fn new_from_table(
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

    raw_metadata_getter_for_tables!(MutationId);

    /// Return the ``site`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn site<M: Into<MutationId> + Copy>(&self, row: M) -> Option<SiteId> {
        assert!(self.num_rows() == 0 || !self.as_ref().site.is_null());
        // SAFETY: either the column is empty or the pointer is not null,
        // in which case the correct lengths are from the low-level objects
        unsafe {
            sys::tsk_column_access::<SiteId, _, _, _>(
                row.into(),
                self.as_ref().site,
                self.num_rows(),
            )
        }
    }

    /// Return the ``node`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn node<M: Into<MutationId> + Copy>(&self, row: M) -> Option<NodeId> {
        assert!(self.num_rows() == 0 || !self.as_ref().node.is_null());
        // SAFETY: either the column is empty or the pointer is not null,
        // in which case the correct lengths are from the low-level objects
        unsafe {
            sys::tsk_column_access::<NodeId, _, _, _>(
                row.into(),
                self.as_ref().node,
                self.num_rows(),
            )
        }
    }

    /// Return the ``parent`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn parent<M: Into<MutationId> + Copy>(&self, row: M) -> Option<MutationId> {
        assert!(self.num_rows() == 0 || !self.as_ref().parent.is_null());
        // SAFETY: either the column is empty or the pointer is not null,
        // in which case the correct lengths are from the low-level objects
        unsafe {
            sys::tsk_column_access::<MutationId, _, _, _>(
                row.into(),
                self.as_ref().parent,
                self.num_rows(),
            )
        }
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time<M: Into<MutationId> + Copy>(&self, row: M) -> Option<Time> {
        assert!(self.num_rows() == 0 || !self.as_ref().time.is_null());
        // SAFETY: either the column is empty or the pointer is not null,
        // in which case the correct lengths are from the low-level objects
        unsafe {
            sys::tsk_column_access::<Time, _, _, _>(row.into(), self.as_ref().time, self.num_rows())
        }
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
        assert!(
            (self.num_rows() == 0 && self.as_ref().derived_state_length == 0)
                || (!self.as_ref().derived_state.is_null()
                    && !self.as_ref().derived_state_offset.is_null())
        );
        // SAFETY: either both columns are empty or both pointers at not NULL,
        // in which case the correct lengths are from the low-level objects
        unsafe {
            sys::tsk_ragged_column_access(
                row.into(),
                self.as_ref().derived_state,
                self.num_rows(),
                self.as_ref().derived_state_offset,
                self.as_ref().derived_state_length,
            )
        }
    }

    /// Retrieve decoded metadata for a `row`.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(T))` if `row` is valid and decoding succeeded.
    /// * `Some(Err(_))` if `row` is not valid and decoding failed.
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
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`MutationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = MutationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&MutationTable>(self)
    }

    pub fn lending_iter(&self) -> MutationTableRowView {
        MutationTableRowView::new(self)
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
    pub fn row<M: Into<MutationId> + Copy>(&self, r: M) -> Option<MutationTableRow> {
        let ri = r.into().into();
        table_row_access!(ri, self, make_mutation_table_row)
    }

    /// Return a view of row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<M: Into<MutationId> + Copy>(&self, r: M) -> Option<MutationTableRowView> {
        let view = MutationTableRowView {
            table: self,
            id: r.into(),
            site: self.site(r)?,
            node: self.node(r)?,
            parent: self.parent(r)?,
            time: self.time(r)?,
            derived_state: self.derived_state(r),
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
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
