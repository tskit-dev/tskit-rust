use crate::bindings as ll_bindings;
use crate::metadata;
use crate::NodeFlags;
use crate::SizeType;
use crate::Time;
use crate::{tsk_id_t, TskitError};
use crate::{IndividualId, NodeId, PopulationId};
use ll_bindings::{tsk_node_table_free, tsk_node_table_init};

/// Row of a [`NodeTable`]
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
    let table_ref = table.table_;
    Some(NodeTableRow {
        id: pos.into(),
        time: table.time(pos).ok()?,
        flags: table.flags(pos).ok()?,
        population: table.population(pos).ok()?,
        individual: table.individual(pos).ok()?,
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type NodeTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a NodeTable<'a>>;
pub(crate) type NodeTableIterator<'a> = crate::table_iterator::TableIterator<NodeTable<'a>>;

impl<'a> Iterator for NodeTableRefIterator<'a> {
    type Item = NodeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_node_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for NodeTableIterator<'a> {
    type Item = crate::node_table::NodeTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_node_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immtable view of a node table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::nodes`](crate::TableAccess::nodes)
/// to get a reference to an existing node table;
pub struct NodeTable<'a> {
    table_: &'a ll_bindings::tsk_node_table_t,
}

impl<'a> NodeTable<'a> {
    pub(crate) fn new_from_table(nodes: &'a ll_bindings::tsk_node_table_t) -> Self {
        NodeTable { table_: nodes }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
    }

    /// Return the ``time`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn time<N: Into<NodeId> + Copy>(&'a self, row: N) -> Result<Time, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.time) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(e),
        }
    }

    /// Return the ``flags`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn flags<N: Into<NodeId> + Copy>(&'a self, row: N) -> Result<NodeFlags, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.flags) {
            Ok(f) => Ok(NodeFlags::from(f)),
            Err(e) => Err(e),
        }
    }

    /// Mutable access to node flags.
    ///
    /// # Note
    ///
    /// Internally, we rely on a conversion of u64 to usize.
    /// This conversion is fallible on some platforms.
    /// If the conversion fails, an empty slice is returned.
    pub fn flags_array_mut(&mut self) -> &mut [NodeFlags] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.table_.flags.cast::<NodeFlags>(),
                usize::try_from(self.table_.num_rows).unwrap_or(0),
            )
        }
    }

    /// Mutable access to node times.
    ///
    /// # Note
    ///
    /// Internally, we rely on a conversion of u64 to usize.
    /// This conversion is fallible on some platforms.
    /// If the conversion fails, an empty slice is returned.
    pub fn time_array_mut(&mut self) -> &mut [Time] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.table_.time.cast::<Time>(),
                usize::try_from(self.table_.num_rows).unwrap_or(0),
            )
        }
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn population<N: Into<NodeId> + Copy>(
        &'a self,
        row: N,
    ) -> Result<PopulationId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.population,
            PopulationId
        )
    }

    /// Return the ``population`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn deme<N: Into<NodeId> + Copy>(&'a self, row: N) -> Result<PopulationId, TskitError> {
        self.population(row)
    }

    /// Return the ``individual`` value from row ``row`` of the table.
    ///
    /// # Errors
    ///
    /// Will return [``IndexError``](crate::TskitError::IndexError)
    /// if ``row`` is out of range.
    pub fn individual<N: Into<NodeId> + Copy>(
        &'a self,
        row: N,
    ) -> Result<IndividualId, TskitError> {
        unsafe_tsk_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.individual,
            IndividualId
        )
    }

    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: NodeId,
    ) -> Option<Result<T, TskitError>> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`NodeTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = NodeTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&NodeTable<'a>>(self)
    }

    /// Return row `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn row<N: Into<NodeId> + Copy>(&self, r: N) -> Result<NodeTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_node_table_row)
    }

    /// Obtain a vector containing the indexes ("ids")
    /// of all nodes for which [`crate::TSK_NODE_IS_SAMPLE`]
    /// is `true`.
    pub fn samples_as_vector(&self) -> Vec<NodeId> {
        let mut samples: Vec<NodeId> = vec![];
        for row in self.iter() {
            if row.flags.contains(NodeFlags::IS_SAMPLE) {
                samples.push(row.id);
            }
        }
        samples
    }

    /// Obtain a vector containing the indexes ("ids") of all nodes
    /// satisfying a certain criterion.
    pub fn create_node_id_vector(
        &self,
        mut f: impl FnMut(&crate::NodeTableRow) -> bool,
    ) -> Vec<NodeId> {
        let mut samples: Vec<NodeId> = vec![];
        for row in self.iter() {
            if f(&row) {
                samples.push(row.id);
            }
        }
        samples
    }
}

build_owned_table_type!(
    /// A standalone node table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwnedNodeTable;
    ///
    /// let mut nodes = OwnedNodeTable::default();
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
    /// use tskit::OwnedNodeTable;
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
    /// let mut nodes = OwnedNodeTable::default();
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
    => OwnedNodeTable,
    NodeTable,
    tsk_node_table_t,
    tsk_node_table_init,
    tsk_node_table_free,
    ll_bindings::tsk_node_table_clear
);

impl OwnedNodeTable {
    node_table_add_row!(=> add_row, self, (*self.table));
    node_table_add_row_with_metadata!(=> add_row_with_metadata, self, (*self.table));
}

#[cfg(test)]
mod test_owned_node_table {
    use super::*;

    #[test]
    fn test_add_row() {
        let mut nodes = OwnedNodeTable::default();
        let rowid = nodes.add_row(0, 1.1, -1, -1).unwrap();
        assert_eq!(rowid, 0);
        assert_eq!(nodes.num_rows(), 1);
    }
}
