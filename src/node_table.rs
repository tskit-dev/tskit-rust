use crate::bindings as ll_bindings;
use crate::metadata;
use crate::SizeType;
use crate::Time;
use crate::{tsk_flags_t, tsk_id_t, TskitError};
use crate::{IndividualId, NodeId, PopulationId};

/// Row of a [`NodeTable`]
pub struct NodeTableRow {
    pub id: NodeId,
    pub time: Time,
    pub flags: tsk_flags_t,
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
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        Some(NodeTableRow {
            id: pos.into(),
            time: table.time(pos).unwrap(),
            flags: table.flags(pos).unwrap(),
            population: table.population(pos).unwrap(),
            individual: table.individual(pos).unwrap(),
            metadata: table_row_decode_metadata!(table, pos),
        })
    } else {
        None
    }
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
/// Instead, use [`TableCollection::nodes`](crate::TableCollection::nodes)
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
    pub fn flags<N: Into<NodeId> + Copy>(&'a self, row: N) -> Result<tsk_flags_t, TskitError> {
        unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.flags)
    }

    /// Mutable access to node flags.
    pub fn flags_array_mut(&mut self) -> &mut [tsk_flags_t] {
        unsafe { std::slice::from_raw_parts_mut(self.table_.flags, self.table_.num_rows as usize) }
    }

    /// Mutable access to node times.
    pub fn time_array_mut(&mut self) -> &mut [Time] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.table_.time.cast::<Time>(),
                self.table_.num_rows as usize,
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
    ) -> Result<Option<T>, TskitError> {
        let buffer = metadata_to_vector!(self, row.0)?;
        decode_metadata_row!(T, buffer)
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
            if row.flags & crate::TSK_NODE_IS_SAMPLE > 0 {
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
