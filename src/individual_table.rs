use crate::bindings as ll_bindings;
use crate::metadata;
use crate::IndividualFlags;
use crate::IndividualId;
use crate::Location;
use crate::{tsk_id_t, tsk_size_t, TskitError};

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
pub struct IndividualTable {
    table_: *const ll_bindings::tsk_individual_table_t,
}

fn make_individual_table_row(table: &IndividualTable, pos: tsk_id_t) -> Option<IndividualTableRow> {
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
        let table_ref = &unsafe { *table.table_ };
        let rv = IndividualTableRow {
            id: pos.into(),
            flags: table.flags(pos).unwrap(),
            location: table.location(pos).unwrap(),
            parents: table.parents(pos).unwrap(),
            metadata: table_row_decode_metadata!(table_ref, pos),
        };
        Some(rv)
    } else {
        None
    }
}

pub(crate) type IndividualTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a IndividualTable>;
pub(crate) type IndividualTableIterator<'a> = crate::table_iterator::TableIterator<IndividualTable>;

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

impl IndividualTable {
    pub(crate) fn as_ll_ref(&self) -> &ll_bindings::tsk_individual_table_t {
        unsafe { &(*self.table_) }
    }

    pub(crate) fn new_from_table(individuals: &ll_bindings::tsk_individual_table_t) -> Self {
        IndividualTable {
            table_: individuals,
        }
    }

    pub(crate) fn new_null() -> Self {
        Self {
            table_: std::ptr::null(),
        }
    }

    pub(crate) fn set_ptr(&mut self, ptr: *const ll_bindings::tsk_individual_table_t) {
        assert!(!ptr.is_null());
        self.table_ = ptr;
    }

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ll_ref().num_rows.into()
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
        match unsafe_tsk_column_access!(row.into().0, 0, self.num_rows(), self.as_ll_ref().flags) {
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
    ) -> Result<Option<Vec<Location>>, TskitError> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().location,
            self.as_ll_ref().location_offset,
            self.as_ll_ref().location_length,
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
    ) -> Result<Option<Vec<IndividualId>>, TskitError> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ll_ref().parents,
            self.as_ll_ref().parents_offset,
            self.as_ll_ref().parents_length,
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
        &self,
        row: IndividualId,
    ) -> Result<Option<T>, TskitError> {
        let table_ref = &unsafe { *self.table_ };
        let buffer = metadata_to_vector!(table_ref, row.0)?;
        decode_metadata_row!(T, buffer)
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`IndividualTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = IndividualTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&IndividualTable>(self)
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
