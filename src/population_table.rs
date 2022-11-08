use std::ptr::NonNull;

use crate::bindings as ll_bindings;
use crate::metadata;
use crate::tsk_id_t;
use crate::PopulationId;
use crate::SizeType;
use crate::TskitError;
use ll_bindings::{tsk_population_table_free, tsk_population_table_init};

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
    let table_ref = table.as_ref();
    let index = ll_bindings::tsk_size_t::try_from(pos).ok()?;

    match index {
        i if i < table.num_rows() => {
            let metadata = table_row_decode_metadata!(table, table_ref, pos).map(|s| s.to_vec());
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

impl<'a> streaming_iterator::StreamingIterator for PopulationTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// An immutable view of site table.
///
/// These are not created directly but are accessed
/// by types implementing [`std::ops::Deref`] to
/// [`crate::table_views::TableViews`]
#[repr(transparent)]
pub struct PopulationTable {
    table_: NonNull<ll_bindings::tsk_population_table_t>,
}

impl PopulationTable {
    pub(crate) fn new_from_table(
        populations: *mut ll_bindings::tsk_population_table_t,
    ) -> Result<Self, TskitError> {
        let n = NonNull::new(populations).ok_or_else(|| {
            TskitError::LibraryError("null pointer to tsk_population_table_t".to_string())
        })?;
        Ok(PopulationTable { table_: n })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_population_table_t {
        // SAFETY: NonNull
        unsafe { self.table_.as_ref() }
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
        let table_ref = self.as_ref();
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
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
        let ri = r.into().0;
        table_row_access!(ri, self, make_population_table_row)
    }
}

build_owned_table_type!(
/// A standalone population table that owns its data.
///
/// # Examples
///
/// ```
/// use tskit::OwnedPopulationTable;
///
/// let mut populations = OwnedPopulationTable::default();
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
/// use tskit::OwnedPopulationTable;
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
/// let mut populations = OwnedPopulationTable::default();
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
    => OwnedPopulationTable,
    PopulationTable,
    tsk_population_table_t,
    tsk_population_table_init,
    tsk_population_table_free,
    ll_bindings::tsk_population_table_clear
);

impl OwnedPopulationTable {
    population_table_add_row!(=> add_row, self, *self.table);
    population_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
