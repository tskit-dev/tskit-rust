use std::ptr::NonNull;

use crate::bindings as ll_bindings;
use crate::metadata;
use crate::IndividualFlags;
use crate::IndividualId;
use crate::Location;
use crate::{tsk_id_t, tsk_size_t, TskitError};
use ll_bindings::{tsk_individual_table_free, tsk_individual_table_init};

/// Row of a [`IndividualTable`]
#[derive(Debug)]
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

pub struct IndividualTableRowView<'a> {
    table: &'a IndividualTable,
    pub id: IndividualId,
    pub flags: IndividualFlags,
    pub location: Option<&'a [Location]>,
    pub parents: Option<&'a [IndividualId]>,
    pub metadata: Option<&'a [u8]>,
}

impl<'a> IndividualTableRowView<'a> {
    fn new(table: &'a IndividualTable) -> Self {
        Self {
            table,
            id: (-1_i32).into(),
            flags: 0.into(),
            location: None,
            parents: None,
            metadata: None,
        }
    }
}

impl<'a> streaming_iterator::StreamingIterator for IndividualTableRowView<'a> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.flags = self.table.flags(self.id).unwrap_or_else(|| 0.into());
        self.location = self.table.location(self.id);
        self.parents = self.table.parents(self.id);
        self.metadata = self.table.raw_metadata(self.id);
    }
}

/// An immutable view of a individual table.
///
/// These are not created directly but are accessed
/// by types implementing [`std::ops::Deref`] to
/// [`crate::table_views::TableViews`]
pub struct IndividualTable {
    table_: NonNull<ll_bindings::tsk_individual_table_t>,
}

fn make_individual_table_row(table: &IndividualTable, pos: tsk_id_t) -> Option<IndividualTableRow> {
    let table_ref = table.as_ref();
    Some(IndividualTableRow {
        id: pos.into(),
        flags: table.flags(pos)?,
        location: table.location(pos).map(|s| s.to_vec()),
        parents: table.parents(pos).map(|s| s.to_vec()),
        metadata: table_row_decode_metadata!(table, table_ref, pos).map(|m| m.to_vec()),
    })
}

pub(crate) type IndividualTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a IndividualTable>;
pub(crate) type IndividualTableIterator = crate::table_iterator::TableIterator<IndividualTable>;

impl<'a> Iterator for IndividualTableRefIterator<'a> {
    type Item = IndividualTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_individual_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl Iterator for IndividualTableIterator {
    type Item = IndividualTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_individual_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl IndividualTable {
    pub(crate) fn new_from_table(
        individuals: *mut ll_bindings::tsk_individual_table_t,
    ) -> Result<Self, TskitError> {
        let n = NonNull::new(individuals).ok_or_else(|| {
            TskitError::LibraryError("null pointer to tsk_individual_table_t".to_string())
        })?;
        Ok(IndividualTable { table_: n })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_individual_table_t {
        // SAFETY: NonNull
        unsafe { self.table_.as_ref() }
    }

    raw_metadata_getter_for_tables!(IndividualId);

    /// Return the number of rows
    pub fn num_rows(&self) -> crate::SizeType {
        self.as_ref().num_rows.into()
    }

    /// Return the flags for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(flags)` if `row` is valid.
    /// * `None` otherwise.
    pub fn flags<I: Into<IndividualId> + Copy>(&self, row: I) -> Option<IndividualFlags> {
        unsafe_tsk_column_access_and_map_into!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ref(),
            flags
        )
    }

    /// Return the locations for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(location)` if `row` is valid.
    /// * `None` otherwise.
    pub fn location<I: Into<IndividualId> + Copy>(&self, row: I) -> Option<&[Location]> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ref(),
            location,
            location_offset,
            location_length,
            Location
        )
    }

    /// Return the parents for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(parents)` if `row` is valid.
    /// * `None` otherwise.
    pub fn parents<I: Into<IndividualId> + Copy>(&self, row: I) -> Option<&[IndividualId]> {
        unsafe_tsk_ragged_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.as_ref(),
            parents,
            parents_offset,
            parents_length,
            IndividualId
        )
    }

    /// Return the metadata for a given row.
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
    /// #
    /// # let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #    x: i32,
    /// # }
    /// # let metadata = IndividualMetadata{x: 1};
    /// # assert!(tables.add_individual_with_metadata(0, None, None,
    /// #                                             &metadata).is_ok());
    /// // We know the metadata are here, so we unwrap the Option and the Result:
    /// let decoded = tables.individuals().metadata::<IndividualMetadata>(0.into()).unwrap().unwrap();
    /// assert_eq!(decoded.x, 1);
    /// # }
    /// ```
    ///
    /// ## Checking for errors and absence of metadata
    ///
    /// The `Option<Result<_>>` return value allows all
    /// three return possibilities to be easily covered:
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// #
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
    /// match tables.individuals().metadata::<IndividualMetadata>(0.into())
    /// {
    ///     Some(Ok(metadata)) => assert_eq!(metadata.x, 1),
    ///     Some(Err(_)) => panic!("got an error??"),
    ///     None => panic!("Got None??"),
    /// };
    /// # }
    /// ```
    ///
    /// ## Attempting to use the wrong type.
    ///
    /// Let's define a mutation metadata type with the exact same fields
    /// as our individual metadata defined above:
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
    /// #[serializer("serde_json")]
    /// struct MutationMetadata {
    ///    x: i32,
    /// }
    /// # }
    /// ```
    ///
    /// This type has the wrong trait bound and will cause compilation to fail:
    ///
    #[cfg_attr(
        feature = "derive",
        doc = r##"
```compile_fail
# #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::MutationMetadata)]
# #[serializer("serde_json")]
# struct MutationMetadata {
#    x: i32,
# }
# 
# let mut tables = tskit::TableCollection::new(10.).unwrap();
match tables.individuals().metadata::<MutationMetadata>(0.into())
{
    Some(Ok(metadata)) => assert_eq!(metadata.x, 1),
    Some(Err(_)) => panic!("got an error??"),
    None => panic!("Got None??"),
};
```
"##
    )]
    ///
    /// ## Limitations: different type, same trait bound
    ///
    /// Finally, let us consider a different struct that has identical
    /// fields to `IndividualMetadata` defined above and also implements
    /// the correct trait:
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// #[serializer("serde_json")]
    /// struct IndividualMetadataToo {
    ///    x: i32,
    /// }
    /// # }
    /// ```
    ///
    /// Let's walk through a detailed example:
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// #
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadata {
    /// #     x: i32,
    /// # }
    /// # #[derive(serde::Serialize, serde::Deserialize, tskit::metadata::IndividualMetadata)]
    /// # #[serializer("serde_json")]
    /// # struct IndividualMetadataToo {
    /// #    x: i32,
    /// # }
    /// // create a mutable table collection
    /// let mut tables = tskit::TableCollection::new(100.).unwrap();
    /// // Create some metadata based on our FIRST type
    /// let metadata = IndividualMetadata { x: 1 };
    /// // Add a row with our metadata
    /// assert!(tables.add_individual_with_metadata(0, None, None, &metadata).is_ok());
    /// // Trying to fetch using our SECOND type as the generic type works!
    /// match tables.individuals().metadata::<IndividualMetadataToo>(0.into())
    /// {
    ///     Some(Ok(metadata)) => assert_eq!(metadata.x, 1),
    ///     Some(Err(_)) => panic!("got an error??"),
    ///     None => panic!("Got None??"),
    /// };
    /// # }
    /// ```
    ///
    /// What is going on here?
    /// Both types satisfy the same trait bound ([`metadata::IndividualMetadata`])
    /// and their data fields look identical to `serde_json`.
    /// Thus, one is exchangeable for the other because they have the exact same
    /// *behavior*.
    ///
    /// However, it is also true that this is (often/usually/always) not exactly what we want.
    /// We are experimenting with encapsulation APIs involving traits with
    /// [associated
    /// types](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#specifying-placeholder-types-in-trait-definitions-with-associated-types) to enforce at *compile time* that exactly one type (`struct/enum`, etc.) is a valid
    /// metadata type for a table.
    pub fn metadata<T: metadata::IndividualMetadata>(
        &self,
        row: IndividualId,
    ) -> Option<Result<T, TskitError>> {
        let table_ref = self.as_ref();
        let buffer = metadata_to_vector!(self, table_ref, row.0)?;
        Some(decode_metadata_row!(T, buffer).map_err(|e| e.into()))
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`IndividualTableRow`].
    ///
    pub fn iter(&self) -> impl Iterator<Item = IndividualTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&IndividualTable>(self)
    }

    pub fn lending_iter(&self) -> IndividualTableRowView {
        IndividualTableRowView::new(self)
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
    pub fn row<I: Into<IndividualId> + Copy>(&self, r: I) -> Option<IndividualTableRow> {
        let ri = r.into().0;
        table_row_access!(ri, self, make_individual_table_row)
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
    /// match individuals.metadata::<IndividualMetadata>(rowid) {
    ///     // rowid is in range, decoding succeeded
    ///     Some(Ok(decoded)) => assert_eq!(decoded.value, 42),
    ///     // rowid is in range, decoding failed
    ///     Some(Err(e)) => panic!("error decoding metadata: {:?}", e),
    ///     None => panic!("row id out of range")
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
