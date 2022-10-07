use crate::bindings as ll_bindings;
use crate::metadata;
use crate::IndividualFlags;
use crate::IndividualId;
use crate::Location;
use crate::{tsk_id_t, tsk_size_t, TskitError};
use ll_bindings::{tsk_individual_table_free, tsk_individual_table_init};

/// Row of a [`IndividualTable`]
pub struct IndividualTableRow {
    pub id: IndividualId,
    pub flags: IndividualFlags,
    pub location: Option<Vec<Location>>,
    pub parents: Option<Vec<IndividualId>>,
    pub metadata: Option<Vec<u8>>,
}

impl PartialEq for IndividualTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.parents == other.parents
            && self.metadata == other.metadata
            && match &self.location {
                None => other.location.is_none(),
                Some(a) => match &other.location {
                    None => false,
                    Some(b) => {
                        if a.len() != b.len() {
                            false
                        } else {
                            for (i, j) in a.iter().enumerate() {
                                if !crate::util::partial_cmp_equal(&b[i], j) {
                                    return false;
                                }
                            }
                            true
                        }
                    }
                },
            }
    }
}

/// An immutable view of a individual table.
///
/// These are not created directly.
/// Instead, use [`TableAccess::individuals`](crate::TableAccess::individuals)
/// to get a reference to an existing node table;
pub struct IndividualTable<'a> {
    table_: &'a ll_bindings::tsk_individual_table_t,
}

fn make_individual_table_row(table: &IndividualTable, pos: tsk_id_t) -> Option<IndividualTableRow> {
    if let Ok(p) = crate::SizeType::try_from(pos) {
        if p < table.num_rows() {
            let table_ref = table.table_;
            let rv = IndividualTableRow {
                id: pos.into(),
                flags: table.flags(pos).unwrap(),
                location: table.location(pos).unwrap().map(|s| s.to_vec()),
                parents: table.parents(pos).unwrap().map(|s| s.to_vec()),
                metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
            };
            Some(rv)
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) type IndividualTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a IndividualTable<'a>>;
pub(crate) type IndividualTableIterator<'a> =
    crate::table_iterator::TableIterator<IndividualTable<'a>>;

impl<'a> Iterator for IndividualTableRefIterator<'a> {
    type Item = IndividualTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_individual_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for IndividualTableIterator<'a> {
    type Item = IndividualTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_individual_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> IndividualTable<'a> {
    pub(crate) fn new_from_table(individuals: &'a ll_bindings::tsk_individual_table_t) -> Self {
        IndividualTable {
            table_: individuals,
        }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> crate::SizeType {
        self.table_.num_rows.into()
    }

    /// Return the flags for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn flags<I: Into<IndividualId> + Copy>(
        &self,
        row: I,
    ) -> Result<IndividualFlags, TskitError> {
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.table_.flags) {
            Ok(f) => Ok(IndividualFlags::from(f)),
            Err(e) => Err(e),
        }
    }

    /// Return the locations for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn location<I: Into<IndividualId> + Copy>(
        &self,
        row: I,
    ) -> Result<Option<&[Location]>, TskitError> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.location,
            self.table_.location_offset,
            self.table_.location_length,
            Location
        )
    }

    /// Return the parents for a given row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    pub fn parents<I: Into<IndividualId> + Copy>(
        &self,
        row: I,
    ) -> Result<Option<&[IndividualId]>, TskitError> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.parents,
            self.table_.parents_offset,
            self.table_.parents_length,
            IndividualId
        )
    }

    /// Return the metadata for a given row.
    ///
    /// # Returns
    ///
    /// The result type is `Option<T>`
    /// where `T`: [`tskit::metadata::IndividualMetadata`](crate::metadata::IndividualMetadata).
    /// `Some(T)` if there is metadata.  `None` if the metadata field is empty for a given
    /// row.
    ///
    /// # Errors
    ///
    /// * [`TskitError::IndexError`] if `row` is out of range.
    ///
    /// # Examples
    ///
    /// For all examples, this is our metadata type.
    /// We will add all instances with a value of `x = 1`.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// #[serializer("serde_json")]
    /// struct IndividualMetadata {
    ///    x: i32,
    /// }
    /// # }
    /// ```
    ///
    /// ## Without matches
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # use tskit::TableAccess;
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #    x: i32,
    /// # }
    /// # let metadata = IndividualMetadata{x: 1};
    /// # assert!(tables.add_individual_with_metadata(0, None, None,
    /// #                                             &metadata).is_ok());
    /// // We know the metadata are here, so we unwrap the Result and the Option
    /// let decoded = tables.individuals().metadata::<IndividualMetadata>(0.into()).unwrap().unwrap();
    /// assert_eq!(decoded.x, 1);
    /// # }
    /// ```
    ///
    /// ## Checking for errors and absence of metadata
    ///
    /// Handling both the possibility of error and optional metadata leads to some verbosity:
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// # use tskit::TableAccess;
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #     x: i32,
    /// # }
    /// # let metadata = IndividualMetadata { x: 1 };
    /// # assert!(tables
    /// #     .add_individual_with_metadata(0, None, None, &metadata)
    /// #     .is_ok());
    /// // First, check the Result.
    /// let decoded_option = match tables
    ///     .individuals()
    ///     .metadata::<IndividualMetadata>(0.into())
    /// {
    ///     Ok(metadata_option) => metadata_option,
    ///     Err(e) => panic!("error: {:?}", e),
    /// };
    /// // Now, check the contents of the Option
    /// match decoded_option {
    ///     Some(metadata) => assert_eq!(metadata.x, 1),
    ///     None => panic!("we expected Some(metadata)?"),
    /// }
    /// # }
    /// ```
    pub fn metadata<T: metadata::MetadataRoundtrip>(
        &'a self,
        row: IndividualId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = self.table_;
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`IndividualTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = IndividualTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&IndividualTable<'a>>(self)
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
    pub fn row<I: Into<IndividualId> + Copy>(
        &self,
        r: I,
    ) -> Result<IndividualTableRow, TskitError> {
        let ri = r.into();
        if ri < 0 {
            return Err(crate::TskitError::IndexError);
        }
        table_row_access!(ri.0, self, make_individual_table_row)
    }
}

build_owned_table_type!(
    /// A standalone individual table that owns its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use tskit::OwnedIndividualTable;
    ///
    /// let mut individuals = OwnedIndividualTable::default();
    /// let rowid = individuals.add_row(0, None, None).unwrap();
    /// assert_eq!(rowid, 0);
    /// assert_eq!(individuals.num_rows(), 1);
    /// ```
    ///
    /// An example with metadata.
    /// This requires the cargo feature `"derive"` for `tskit`.
    ///
    /// ```
    /// # #[cfg(any(feature="doc", feature="derive"))] {
    /// use tskit::OwnedIndividualTable;
    ///
    /// #[derive(serde::Serialize,
    ///          serde::Deserialize,
    ///          tskit::metadata::IndividualMetadata)]
    /// #[serializer("serde_json")]
    /// struct IndividualMetadata {
    ///     value: i32,
    /// }
    ///
    /// let metadata = IndividualMetadata{value: 42};
    ///
    /// let mut individuals = OwnedIndividualTable::default();
    ///
    /// let rowid = individuals.add_row_with_metadata(0, None, None, &metadata).unwrap();
    /// assert_eq!(rowid, 0);
    ///
    /// if let Some(decoded) = individuals.metadata::<IndividualMetadata>(rowid).unwrap() {
    ///     assert_eq!(decoded.value, 42);
    /// } else {
    ///     panic!("hmm...we expected some metadata!");
    /// }
    ///
    /// # }
    /// ```
    => OwnedIndividualTable,
    IndividualTable,
    tsk_individual_table_t,
    tsk_individual_table_init,
    tsk_individual_table_free,
    crate::bindings::tsk_individual_table_clear
);

impl OwnedIndividualTable {
    individual_table_add_row!(=> add_row, self, *self.table);
    individual_table_add_row_with_metadata!(=> add_row_with_metadata, self, *self.table);
}
