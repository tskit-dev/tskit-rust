use crate::metadata;
use crate::sys;
use crate::IndividualFlags;
use crate::IndividualId;
use crate::Location;
use crate::TskitError;
use ll_bindings::tsk_id_t;
use sys::bindings as ll_bindings;

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
            && self.location == other.location
    }
}

#[derive(Debug)]
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

impl PartialEq for IndividualTableRowView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && self.parents == other.parents
            && self.metadata == other.metadata
            && self.location == other.location
    }
}

impl Eq for IndividualTableRowView<'_> {}

impl PartialEq<IndividualTableRow> for IndividualTableRowView<'_> {
    fn eq(&self, other: &IndividualTableRow) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && optional_container_comparison!(self.parents, other.parents)
            && optional_container_comparison!(self.metadata, other.metadata)
            && optional_container_comparison!(self.location, other.location)
    }
}

impl PartialEq<IndividualTableRowView<'_>> for IndividualTableRow {
    fn eq(&self, other: &IndividualTableRowView) -> bool {
        self.id == other.id
            && self.flags == other.flags
            && optional_container_comparison!(self.parents, other.parents)
            && optional_container_comparison!(self.metadata, other.metadata)
            && optional_container_comparison!(self.location, other.location)
    }
}

impl crate::StreamingIterator for IndividualTableRowView<'_> {
    type Item = Self;

    row_lending_iterator_get!();

    fn advance(&mut self) {
        self.id = (i32::from(self.id) + 1).into();
        self.flags = self.table.flags(self.id).unwrap_or_else(|| 0.into());
        self.location = self.table.location(self.id);
        self.parents = self.table.parents(self.id);
        self.metadata = self.table.table_.raw_metadata(self.id);
    }
}

/// An individual table.
///
/// # Examples
///
/// ```
/// use tskit::IndividualTable;
///
/// let mut individuals = IndividualTable::default();
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
/// use tskit::IndividualTable;
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
/// let mut individuals = IndividualTable::default();
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
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct IndividualTable {
    table_: sys::IndividualTable,
}

fn make_individual_table_row(table: &IndividualTable, pos: tsk_id_t) -> Option<IndividualTableRow> {
    Some(IndividualTableRow {
        id: pos.into(),
        flags: table.flags(pos)?,
        location: table.location(pos).map(|s| s.to_vec()),
        parents: table.parents(pos).map(|s| s.to_vec()),
        metadata: table.table_.raw_metadata(pos).map(|m| m.to_vec()),
    })
}

pub(crate) type IndividualTableRefIterator<'a> =
    crate::table_iterator::TableIterator<&'a IndividualTable>;
pub(crate) type IndividualTableIterator = crate::table_iterator::TableIterator<IndividualTable>;

impl Iterator for IndividualTableRefIterator<'_> {
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
    // # Safety
    //
    // * this fn must NEVER by part of the public API
    // * all returned values must only be visible to the public API
    //   by REFERENCE (& or &mut)
    // * the input ptr must not be NULL
    // * the input ptr must point to an initialized table
    pub(crate) unsafe fn new_from_table(
        individuals: *mut ll_bindings::tsk_individual_table_t,
    ) -> Result<Self, TskitError> {
        let ptr = std::ptr::NonNull::new(individuals).unwrap();
        let table_ = unsafe { sys::IndividualTable::new_borrowed(ptr) };
        Ok(IndividualTable { table_ })
    }

    pub(crate) fn as_ref(&self) -> &ll_bindings::tsk_individual_table_t {
        self.table_.as_ref()
    }

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
        self.table_.flags(row.into())
    }

    /// Return the locations for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(location)` if `row` is valid.
    /// * `None` otherwise.
    pub fn location<I: Into<IndividualId> + Copy>(&self, row: I) -> Option<&[Location]> {
        self.table_.location(row.into())
    }

    /// Return the parents for a given row.
    ///
    /// # Returns
    ///
    /// * `Some(parents)` if `row` is valid.
    /// * `None` otherwise.
    pub fn parents<I: Into<IndividualId> + Copy>(&self, row: I) -> Option<&[IndividualId]> {
        self.table_.parents(row.into())
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
    /// let decoded = tables.individuals().metadata::<IndividualMetadata>(0).unwrap().unwrap();
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
    /// match tables.individuals().metadata::<IndividualMetadata>(0)
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
match tables.individuals().metadata::<MutationMetadata>(0)
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
    /// match tables.individuals().metadata::<IndividualMetadataToo>(0)
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
        row: impl Into<IndividualId>,
    ) -> Option<Result<T, TskitError>> {
        let buffer = self.table_.raw_metadata(row)?;
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
        let ri = r.into().into();
        table_row_access!(ri, self, make_individual_table_row)
    }

    /// Return a view of `r` of the table.
    ///
    /// # Parameters
    ///
    /// * `r`: the row id.
    ///
    /// # Returns
    ///
    /// * `Some(row view)` if `r` is valid
    /// * `None` otherwise
    pub fn row_view<I: Into<IndividualId> + Copy>(&self, r: I) -> Option<IndividualTableRowView> {
        let view = IndividualTableRowView {
            table: self,
            id: r.into(),
            flags: self.flags(r)?,
            location: self.location(r),
            parents: self.parents(r),
            metadata: self.table_.raw_metadata(r.into()),
        };
        Some(view)
    }

    build_table_column_slice_getter!(
        /// Get the flags column as a slice
        => flags, flags_slice, IndividualFlags);
    build_table_column_slice_getter!(
        /// Get the flags column as a slice
        => flags, flags_slice_raw, ll_bindings::tsk_flags_t);

    /// Clear all data from the table
    pub fn clear(&mut self) -> Result<i32, TskitError> {
        handle_tsk_return_value!(self.table_.clear())
    }

    pub fn add_row<F, L, P>(
        &mut self,
        flags: F,
        location: L,
        parents: P,
    ) -> Result<IndividualId, TskitError>
    where
        F: Into<IndividualFlags>,
        L: crate::IndividualLocation,
        P: crate::IndividualParents,
    {
        let location = unsafe {
            std::slice::from_raw_parts(
                location.get_slice().as_ptr().cast::<f64>(),
                location.get_slice().len(),
            )
        };
        let parents = unsafe {
            std::slice::from_raw_parts(
                parents.get_slice().as_ptr().cast::<tsk_id_t>(),
                parents.get_slice().len(),
            )
        };
        let rv = self
            .table_
            .add_row(flags.into().bits(), location, parents)?;
        handle_tsk_return_value!(rv, rv.into())
    }

    pub fn add_row_with_metadata<F, L, P, M>(
        &mut self,
        flags: F,
        location: L,
        parents: P,
        metadata: &M,
    ) -> Result<IndividualId, TskitError>
    where
        F: Into<IndividualFlags>,
        L: crate::IndividualLocation,
        P: crate::IndividualParents,
        M: crate::metadata::IndividualMetadata,
    {
        let md = crate::metadata::EncodedMetadata::new(metadata)?;
        let location = unsafe {
            std::slice::from_raw_parts(
                location.get_slice().as_ptr().cast::<f64>(),
                location.get_slice().len(),
            )
        };
        let parents = unsafe {
            std::slice::from_raw_parts(
                parents.get_slice().as_ptr().cast::<tsk_id_t>(),
                parents.get_slice().len(),
            )
        };
        let rv = self.table_.add_row_with_metadata(
            flags.into().bits(),
            location,
            parents,
            md.as_slice(),
        )?;
        handle_tsk_return_value!(rv, rv.into())
    }
}
