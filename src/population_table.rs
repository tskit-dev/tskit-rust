use crate::bindings as ll_bindings;
use crate::metadata;
use crate::sys;
use crate::tsk_id_t;
use crate::PopulationId;
use crate::SizeType;
use crate::TskitError;

/// Row of a [`PopulationTable`]
#[derive(Eq, Debug)]
pub struct PopulationTableRow {
    pub id: PopulationId,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for PopulationTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.metadata == other.metadata
    }
}

fn make_population_table_row(table: &PopulationTable, pos: tsk_id_t) -> Option<PopulationTableRow> {
    let index = ll_bindings::tsk_size_t::try_from(pos).ok()?;

    match index {
        i if i < table.num_rows() => {
            let metadata = table.raw_metadata(pos).map(|m| m.to_vec());
            Some(PopulationTableRow {
                id: pos.into(),
                metadata,
            })
        }
        _ => None,
    }
}

pub(crate) type PopulationTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a PopulationTable>;
pub(crate) type PopulationTableIterator = crate::table_iterator::TableIterator<PopulationTable>;

impl<'a> Iterator for PopulationTableRefIterator<'a> {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for PopulationTableIterator {
    type Item = PopulationTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_population_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

#[derive(Debug)]
pub struct PopulationTableRowView<'a> {
    table: &'a PopulationTable,
    pub id: PopulationId,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> PopulationTableRowView<'a> {
    fn new(table: &'a PopulationTable) -> Self {
        Self {
            table,
            id: PopulationId::NULL,
            metadata: None,
        }
    }
}

impl<'a> PartialEq for PopulationTableRowView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.metadata == other.metadata
    }
}

impl<'a> Eq for PopulationTableRowView<'a> {}

impl<'a> PartialEq<PopulationTableRow> for PopulationTableRowView<'a> {
    fn eq(&self, other: &PopulationTableRow) -> bool {
        self.id == other.id && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl PartialEq<PopulationTableRowView<'_>> for PopulationTableRow {
    fn eq(&self, other: &PopulationTableRowView) -> bool {
        self.id == other.id && optional_container_comparison!(self.metadata, other.metadata)
    }
}

impl<'a> streaming_iterator::StreamingIterator for PopulationTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// A population table.
///
/// # Examples
///
/// # Standalone tables
///
/// ```
/// use tskit::PopulationTable;
///
/// let mut populations = PopulationTable::default();
/// let rowid = populations.add_row().unwrap();
/// assert_eq!(rowid, 0);
/// assert_eq!(populations.num_rows(), 1);
/// ```
///
/// An example with metadata.
/// This requires the cargo feature `"derive"` for `tskit`.
///
/// ```
/// # #[cfg(any(feature="doc", feature="derive"))] {
/// use tskit::PopulationTable;
///
/// #[derive(serde::Serialize,
///          serde::Deserialize,
///          tskit::metadata::PopulationMetadata)]
/// #[serializer("serde_json")]
/// struct PopulationMetadata {
///     name: String,
/// }
///
/// let metadata = PopulationMetadata{name: "YRB".to_string()};
///
/// let mut populations = PopulationTable::default();
///
/// let rowid = populations.add_row_with_metadata(&metadata).unwrap();
/// assert_eq!(rowid, 0);
///
/// match populations.metadata::<PopulationMetadata>(rowid) {
///     // rowid is in range, decoding succeeded
///     Some(Ok(decoded)) => assert_eq!(&decoded.name, "YRB"),
///     // rowid is in range, decoding failed
///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
///     None => panic!("row id out of range")
/// }
/// # }
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct PopulationTable {
    table_: sys::LLPopulationTable,
}

impl PopulationTable {
    pub(crate) fn as_ptr(&self) -> *const ll_bindings::tsk_population_table_t {
        self.table_.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut ll_bindings::tsk_population_table_t {
        self.table_.as_mut_ptr()
    }

    pub(crate) fn new_from_table(
        populations: *mut ll_bindings::tsk_population_table_t,
    ) -> Result<Self, TskitError> {
        let table_ = sys::LLPopulationTable::new_non_owning(populations)?;
        Ok(PopulationTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_population_table_t {
        self.table_.as_ref()
    }

    pub fn new() -> Result<Self, TskitError> {
        let table_ = sys::LLPopulationTable::new_owning(0)?;
        Ok(Self { table_ })
    }

    raw_metadata_getter_for_tables!(PopulationId);

    /// Return the number of rows.
    pub fn num_rows(&self) -> SizeType {
        self.as_ref().num_rows.into()
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
    pub fn metadata<T: metadata::PopulationMetadata>(
        &self,
        row: PopulationId,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.raw_metadata(row)?;
        Some(decode_metadata_row!(T, buffer).map_err(TskitError::from))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`PopulationTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = PopulationTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&PopulationTable>(self)
    }

    pub fn lending_iter(&self) -> PopulationTableRowView {
        PopulationTableRowView::new(self)
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
    pub fn row<P: Into<PopulationId> + Copy>(&self, r: P) -> Option<PopulationTableRow> {
        let ri = r.into().into();
        table_row_access!(ri, self, make_population_table_row)
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
    pub fn row_view<P: Into<PopulationId> + Copy>(&self, r: P) -> Option<PopulationTableRowView> {
        match SizeType::try_from(r.into()).ok() {
            Some(row) if row < self.num_rows() => {
                let view = PopulationTableRowView {
                    table: self,
                    id: r.into(),
                    metadata: self.raw_metadata(r.into()),
                };
                Some(view)
            }
            _ => None,
        }
    }

    pub fn clear(&mut self) -> Result<(), TskitError> {
        self.table_.clear().map_err(|e| e.into())
    }

    population_table_add_row!(=> add_row, self, self.as_mut_ptr());
    population_table_add_row_with_metadata!(=> add_row_with_metadata, self, self.as_mut_ptr());
}

impl Default for PopulationTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

pub type OwningPopulationTable = PopulationTable;
