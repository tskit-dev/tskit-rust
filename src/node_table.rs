use crate::bindings as ll_bindings;
use crate::metadata;
use crate::sys;
use crate::NodeFlags;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{IndividualId, NodeId, PopulationId};

/// Row of a [`NodeTable`]
#[derive(Debug)]
pub struct NodeTableRow {
    pub id: NodeId,
    pub time: Time,
    pub flags: NodeFlags,
    pub population: PopulationId,
    pub individual: IndividualId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for NodeTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.population == other.population
            && self.individual == other.individual
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.metadata == other.metadata
    }
}

fn make_node_table_row(table: &NodeTable, pos: tsk_id_t) -> Option<NodeTableRow> {
    Some(NodeTableRow {
        id: pos.into(),
        time: table.time(pos)?,
        flags: table.flags(pos)?,
        population: table.population(pos)?,
        individual: table.individual(pos)?,
        metadata: table.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type NodeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a NodeTable>;
pub(crate) type NodeTableIterator = crate::table_iterator::TableIterator<NodeTable>;

impl<'a> Iterator for NodeTableRefIterator<'a> {
    type Item = NodeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_node_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for NodeTableIterator {
    type Item = crate::node_table::NodeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_node_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct NodeTableRowView<'a> {
    table: &'a NodeTable,
    pub id: NodeId,
    pub time: Time,
    pub flags: NodeFlags,
    pub population: PopulationId,
    pub individual: IndividualId,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> NodeTableRowView<'a> {
    fn new(table: &'a NodeTable) -> Self {
        Self {
            table,
            id: NodeId::NULL,
            time: f64::NAN.into(),
            flags: 0.into(),
            population: PopulationId::NULL,
            individual: IndividualId::NULL,
            metadata: None,
        }
    }
}

impl<'a> PartialEq for NodeTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.population == other.population
            && self.individual == other.individual
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && self.metadata == other.metadata
    }
}

impl<'a> Eq for NodeTableRowView<'a> {}

impl<'a> PartialEq<NodeTableRow> for NodeTableRowView<'a> {
    fn eq(&self, other: &NodeTableRow) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.population == other.population
            && self.individual == other.individual
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<NodeTableRowView<'_>> for NodeTableRow {
    fn eq(&self, other: &NodeTableRowView) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.population == other.population
            && self.individual == other.individual
            && crate::util::partial_cmp_equal(&self.time, &other.time)
            && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for NodeTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.time = self.table.time(self.id).unwrap_or_else(|| f64::NAN.into());
        self.flags = self.table.flags(self.id).unwrap_or_else(|| 0.into());
        self.population = self.table.population(self.id).unwrap_or(PopulationId::NULL);
        self.individual = self.table.individual(self.id).unwrap_or(IndividualId::NULL);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// A node table.
///
/// # Examples
///
/// ```
/// use tskit::NodeTable;
///
/// let mut nodes = NodeTable::default();
/// let rowid = nodes.add_row(0, 1.1, -1, -1).unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(nodes.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::NodeTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::NodeMetadata)]
/// #[serializer("serde_json")]
/// struct NodeMetadata {
///     value: i32,
/// }
///
/// let metadata = NodeMetadata{value: 42};
///
/// let mut nodes = NodeTable::default();
///
/// let rowid = nodes.add_row_with_metadata(0, 1., -1, -1, &metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match nodes.metadata::<NodeMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
///
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct NodeTable {
    table_: sys::LLNodeTable,
}

impl NodeTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_node_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_node_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        nodes: *mut ll_bindings::tsk_node_table_t,
    ) -> Result<Self, TskitError> {
        let table_ = sys::LLNodeTable::new_non_owning(nodes)?;
        Ok(NodeTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_node_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::LLNodeTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
    }

    raw_metadata_getter_for_tables!(NodeId);

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(time)` if `row` is valid.
    /// * `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(0, 10.0, -1, -1).unwrap();
    /// if let Some(time) = tables.nodes().time(0) {
    /// // then node id 0 is a valid row id
    /// # assert_eq!(time, 10.0);
    /// }
    /// # else {
    /// #   panic!("expected 0 to be a valid row id")
    /// # }
    /// ```
    pub fn time<N: Into<NodeId> + Copy>(&self, row: N) -> Option<Time> {
        sys::tsk_column_access::<Time, _, _, _>(row.into(), self.as_ref().time, self.num_rows())
    }

    /// Return the ``flags`` value from row ``row`` of the table.
    ///
    /// # Returns
    ///
    /// * `Some(flags)` if `row` is valid.
    /// * `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// if let Some(flags) = tables.nodes().flags(0) {
    /// // then node id 0 is a valid row id
    /// # assert!(flags.is_sample());
    /// }
    /// # else {
    /// #   panic!("expected 0 to be a valid row id")
    /// # }
    /// ```
    pub fn flags<N: Into<NodeId> + Copy>(&self, row: N) -> Option<NodeFlags> {
        sys::tsk_column_access::<NodeFlags, _, _, _>(
            row.into(),
            self.as_ref().flags,
            self.num_rows(),
        )
    }

    #[deprecated(since = "0.12.0", note = "use flags_slice_mut instead")]
    pub fn flags_array_mut(&mut self) -> &mut [NodeFlags] {
        sys::generate_slice_mut(self.as_ref().flags, self.num_rows())
    }

    #[deprecated(since = "0.12.0", note = "use time_slice_mut instead")]
    pub fn time_array_mut(&mut self) -> &mut [Time] {
        sys::generate_slice_mut(self.as_ref().flags, self.num_rows())
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(0, 10.0, -1, -1).unwrap();
    /// if let Some(pop) = tables.nodes().population(0) {
    /// // then node id 0 is a valid row id
    /// # assert!(pop.is_null());
    /// }
    /// # else {
    /// #   panic!("expected 0 to be a valid row id")
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn population<N: Into<NodeId> + Copy>(&self, row: N) -> Option<PopulationId> {
        sys::tsk_column_access::<PopulationId, _, _, _>(
            row.into(),
            self.as_ref().population,
            self.num_rows(),
        )
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Examples
    ///
    /// See [`NodeTable::population`] for examples.
    ///
    /// # Returns
    ///
    /// * `Some(population)` if `row` is valid.
    /// * `None` otherwise.
    pub fn deme<N: Into<NodeId> + Copy>(&self, row: N) -> Option<PopulationId> {
        self.population(row)
    }

    /// Return the ``individual`` value from row ``row`` of the table.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(0, 10.0, -1, -1).unwrap();
    /// if let Some(individual) = tables.nodes().individual(0) {
    /// // then node id 0 is a valid row id
    /// # assert!(individual.is_null());
    /// }
    /// # else {
    /// #   panic!("expected 0 to be a valid row id")
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// * `Some(individual)` if `row` is valid.
    /// * `None` otherwise.
    pub fn individual<N: Into<NodeId> + Copy>(&self, row: N) -> Option<IndividualId> {
        sys::tsk_column_access::<IndividualId, _, _, _>(
            row.into(),
            self.as_ref().individual,
            self.num_rows(),
        )
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
    pub fn metadata<T: metadata::NodeMetadata>(
        &self,
        row: NodeId,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`NodeTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = NodeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&NodeTable>(self)
    }

    pub fn lending_iter(&self) -> NodeTableRowView {
        NodeTableRowView::new(self)
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
    pub fn row<N: Into<NodeId> + Copy>(&self, r: N) -> Option<NodeTableRow> {
        let ri = r.into().into();
        table_row_access!(ri, self, make_node_table_row)
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
    pub fn row_view<N: Into<NodeId> + Copy>(&self, r: N) -> Option<NodeTableRowView> {
        let view = NodeTableRowView {
            table: self,
            id: r.into(),
            time: self.time(r)?,
            flags: self.flags(r)?,
            population: self.population(r)?,
            individual: self.individual(r)?,
            metadata: self.raw_metadata(r.into()),
        };
        Some(view)
    }
    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::TSK_NODE_IS_SAMPLE`]
    /// is `true`.
    pub fn samples_as_vector(&self) -> Vec<NodeId> {
        self.create_node_id_vector(|row| row.flags.contains(NodeFlags::IS_SAMPLE))
    }

    /// Obtain a vector containing the indexes ("ids") of all nodes
    /// satisfying a certain criterion.
    pub fn create_node_id_vector(&self, mut f: impl FnMut(&NodeTableRow) -> bool) -> Vec<NodeId> {
        self.iter()
            .filter(|row| f(row))
            .map(|row| row.id)
            .collect::<Vec<_>>()
    }

    pub fn clear(&mut self) -> Result<(), TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    fn add_row_details<F, T, P, I>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
        metadata: *const i8,
        metadata_len: u64,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
    {
        let rv = unsafe {
            crate::bindings::tsk_node_table_add_row(
                self.as_mut_ptr(),
                flags.into().bits(),
                time.into().into(),
                population.into().into(),
                individual.into().into(),
                metadata,
                metadata_len,
            )
        };
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row<F, T, P, I>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
    {
        self.add_row_details(flags, time, population, individual, std::ptr::null(), 0)
    }

    pub fn add_row_with_metadata<F, T, P, I, M>(
        &mut self,
        flags: F,
        time: T,
        population: P,
        individual: I,
        metadata: &M,
    ) -> Result<NodeId, TskitError>
    where
        F: Into<crate::NodeFlags>,
        T: Into<crate::Time>,
        P: Into<crate::PopulationId>,
        I: Into<crate::IndividualId>,
        M: crate::metadata::NodeMetadata,
    {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let mdlen = md.len()?;
        self.add_row_details(
            flags,
            time,
            population,
            individual,
            md.as_ptr(),
            mdlen.into(),
        )
    }

    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice, Time);
    build_table_column_slice_getter!(
        /// Get the time column as a slice
        => time, time_slice_raw, f64);
    build_table_column_slice_mut_getter!(
    /// Get the time column as a mutable slice
    ///
    /// # Examples
    ///
    /// For a [`crate::TableCollection`], accessing the table creates a temporary
    /// that will be dropped, causing this code to not compile:
    ///
    /// ```compile_fail
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// let time = tables.nodes().time_slice_mut();
    /// println!("{}", time.len()); // ERROR: the temporary node table is dropped by now
    /// ```
    ///
    /// Treating the returned slice as an iterable succeeds:
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// for time in tables.nodes_mut().time_slice_mut() {
    ///     *time = 55.0.into(); // change each node's time value
    /// }
    /// assert!(tables.nodes_mut().time_slice_mut().iter().all(|t| t == &55.0));
    /// ```
    ///
    /// # Panics
    ///
    /// Internally, we rely on a conversion of u64 to usize.
    /// This conversion is fallible on some platforms.
    /// If the conversion fails, this function will panic.
        => time, time_slice_mut, Time);
    build_table_column_slice_mut_getter!(
        /// Get the time column as a mutable slice
        => time, time_slice_raw_mut, f64);
    build_table_column_slice_getter!(
        /// Get the flags column as a slice
        => flags, flags_slice, NodeFlags);
    build_table_column_slice_getter!(
        /// Get the flags column as a slice
        => flags, flags_slice_raw, ll_bindings::tsk_flags_t);
    build_table_column_slice_mut_getter!(
    /// Get the flags column as a mutable slice
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// let flags = tables.nodes_mut().flags_slice_mut();
    /// for flag in flags {
    /// // Can do something...
    /// # assert!(flag.is_sample());
    /// }
    /// ```
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// for flag in  tables.nodes_mut().flags_slice_mut() {
    /// # assert!(flag.is_sample());
    /// }
    /// ```
    ///
    /// The returned slice is *mutable*, allowing one to do things like
    /// clear the sample status of all nodes:
    ///
    /// A copy of the flags can be obtained by collecting results into `Vec`:
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// for flag in tables.nodes_mut().flags_slice_mut() {
    ///     flag.remove(tskit::NodeFlags::IS_SAMPLE);
    /// }
    /// assert!(!tables.nodes_mut().flags_slice_mut().iter().any(|f| f.is_sample()));
    /// assert!(tables.nodes().samples_as_vector().is_empty());
    /// ```
    ///
    /// ```
    /// # use tskit::prelude::*;
    /// # let mut tables = tskit::TableCollection::new(10.).unwrap();
    /// # tables.add_node(tskit::NodeFlags::IS_SAMPLE, 10.0, -1, -1).unwrap();
    /// let flags = tables.nodes_mut().flags_slice_mut().to_vec();
    /// # assert!(flags.iter().all(|f| f.is_sample()));
    /// ```
    ///
    /// ## Standalone tables
    ///
    /// The ownership semantics differ when tables are not part of a
    /// table collection:
    ///
    /// ```
    /// let mut nodes = tskit::NodeTable::default();
    /// assert!(nodes.add_row(tskit::NodeFlags::IS_SAMPLE, 10., -1, -1).is_ok());
    /// # assert_eq!(nodes.num_rows(), 1);
    /// let flags = nodes.flags_slice_mut();
    /// # assert_eq!(flags.len(), 1);
    /// assert!(flags.iter().all(|f| f.is_sample()));
    ///
    /// // while we are at it, let's use our node
    /// // table to populate a table collection.
    /// #
    /// let mut tables = tskit::TableCollection::new(10.0).unwrap();
    /// tables.set_nodes(&nodes);
    /// assert_eq!(tables.nodes().num_rows(), 1);
    /// assert_eq!(tables.nodes_mut().flags_slice_mut().iter().filter(|f| f.is_sample()).count(), 1);
    /// ```
    ///
    /// # Panics
    ///
    /// Internally, we rely on a conversion of u64 to usize.
    /// This conversion is fallible on some platforms.
    /// If the conversion fails, this function will panic.
        => flags, flags_slice_mut, NodeFlags);
    build_table_column_slice_mut_getter!(
        /// Get the flags column as a mutable slice
        => flags, flags_slice_raw_mut, ll_bindings::tsk_flags_t);
    build_table_column_slice_getter!(
        /// Get the individual column as a slice
        => individual, individual_slice, IndividualId);
    build_table_column_slice_getter!(
        /// Get the individual column as a slice
        => individual, individual_slice_raw, crate::tsk_id_t);
    build_table_column_slice_getter!(
        /// Get the population column as a slice
        => population, population_slice, PopulationId);
    build_table_column_slice_getter!(
        /// Get the population column as a slice
        => population, population_slice_raw, crate::tsk_id_t);
}

pub type OwningNodeTable = NodeTable;

impl Default for NodeTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod test_owned_node_table {
    use super::*;

    #[test]
    fn test_add_row() {
        let mut nodes = NodeTable::default();
        let rowid = nodes.add_row(0, 1.1, -1, -1).unwrap();
        assert_eq!(rowid, 0);
        assert_eq!(nodes.num_rows(), 1);
    }
}
