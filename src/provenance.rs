//! Optional module for table and tree sequence provenance tables.
//!
//! This module is enabled via the `"provenance"` feature and provides
//! the following:
//!
//! * trait [`Provenance`], which enables populating and accessing
//!   [`ProvenanceTable`].
//! * [`ProvenanceTableRow`], which is the value type returned by
//!   [`ProvenanceTable::iter`].
//!
//! See [`Provenance`] for examples.

use crate::bindings as ll_bindings;
use crate::{tsk_id_t, tsk_size_t, ProvenanceId, TskitError};

/// Enable provenance table access.
///
/// `tskit` provides implementations of this trait
/// for [`crate::TableCollection`] and [`crate::TreeSequence`].
#[cfg_attr(
    feature = "provenance",
    doc = r##"
# Examples

## For table collections

```
use tskit::provenance::Provenance;
let mut tables = tskit::TableCollection::new(1000.).unwrap();
tables.add_provenance(&String::from("Some provenance")).unwrap();

// Get reference to the table
let prov_ref = tables.provenances();

// Get the first row
let row_0 = prov_ref.row(0).unwrap();

assert_eq!(row_0.record, "Some provenance");

// Get the first record
let record_0 = prov_ref.record(0).unwrap();
assert_eq!(record_0, row_0.record);

// Get the first time stamp
let timestamp = prov_ref.timestamp(0).unwrap();
assert_eq!(timestamp, row_0.timestamp);

// You can get the `chrono::DateTime` object back from the `String`:

let dt = chrono::DateTime::parse_from_rfc3339(&timestamp).unwrap();

// You can get specific time types back, too:
use core::str::FromStr;
let dt_utc = chrono::DateTime::<chrono::Utc>::from_str(&timestamp).unwrap();
let dt_local = chrono::DateTime::<chrono::Local>::from_str(&timestamp).unwrap();
println!("local = {}", dt_local);

// Provenance transfers to the tree sequences
let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
assert_eq!(treeseq.provenances().record(0).unwrap(), "Some provenance");
// We can still compare to row_0 because it is a copy of the row data:
assert_eq!(treeseq.provenances().record(0).unwrap(), row_0.record);
```

## For tree sequences

```
use tskit::provenance::Provenance;
let mut tables = tskit::TableCollection::new(1000.).unwrap();
let mut treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES).unwrap();
treeseq.add_provenance(&String::from("All your provenance r belong 2 us.")).unwrap();

let prov_ref = treeseq.provenances();
let row_0 = prov_ref.row(0).unwrap();
assert_eq!(row_0.record, "All your provenance r belong 2 us.");
let record_0 = prov_ref.record(0).unwrap();
assert_eq!(record_0, row_0.record);
let timestamp = prov_ref.timestamp(0).unwrap();
assert_eq!(timestamp, row_0.timestamp);
let dt = chrono::DateTime::parse_from_rfc3339(&timestamp).unwrap();
use core::str::FromStr;
let dt_utc = chrono::DateTime::<chrono::Utc>::from_str(&timestamp).unwrap();
let dt_local = chrono::DateTime::<chrono::Local>::from_str(&timestamp).unwrap();
println!("local = {}", dt_local);
```

"##
)]
pub trait Provenance: crate::TableAccess {
    /// Add provenance record with a time stamp.
    ///
    /// All implementation of this trait provided by `tskit` use
    /// an `ISO 8601` format time stamp
    /// written using the [RFC 3339](https://tools.ietf.org/html/rfc3339)
    /// specification.
    /// This formatting approach has been the most straightforward method
    /// for supporting round trips to/from a [`ProvenanceTable`].
    /// The implementations used here use the [`chrono`](https://docs.rs/chrono) crate.
    ///
    /// # Parameters
    ///
    /// * `record`: the provenance record
    fn add_provenance(&mut self, record: &str) -> Result<ProvenanceId, TskitError>;
    /// Return an immutable reference to the table, type [`ProvenanceTable`]
    fn provenances(&self) -> ProvenanceTable;
    /// Return an iterator over the rows of the [`ProvenanceTable`].
    /// See [`ProvenanceTable::iter`] for details.
    fn provenances_iter(&self) -> ProvenanceTableIterator {
        crate::table_iterator::make_table_iterator::<ProvenanceTable>(self.provenances())
    }
}

#[derive(Eq)]
/// Row of a [`ProvenanceTable`].
pub struct ProvenanceTableRow {
    /// The row id
    pub id: ProvenanceId,
    /// ISO-formatted time stamp
    pub timestamp: String,
    /// The provenance record
    pub record: String,
}

impl PartialEq for ProvenanceTableRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.timestamp == other.timestamp && self.record == other.record
    }
}

impl std::fmt::Display for ProvenanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ProvenanceId({})", self.0)
    }
}

impl std::fmt::Display for ProvenanceTableRow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "id: {}, timestamp: {}, record: {}",
            self.id, self.timestamp, self.record,
        )
    }
}

fn make_provenance_table_row(table: &ProvenanceTable, pos: tsk_id_t) -> Option<ProvenanceTableRow> {
    if pos < table.num_rows() as tsk_id_t {
        Some(ProvenanceTableRow {
            id: pos.into(),
            timestamp: table.timestamp(pos).unwrap(),
            record: table.record(pos).unwrap(),
        })
    } else {
        None
    }
}

type ProvenanceTableRefIterator<'a> = crate::table_iterator::TableIterator<&'a ProvenanceTable<'a>>;
type ProvenanceTableIterator<'a> = crate::table_iterator::TableIterator<ProvenanceTable<'a>>;

impl<'a> Iterator for ProvenanceTableRefIterator<'a> {
    type Item = ProvenanceTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_provenance_table_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for ProvenanceTableIterator<'a> {
    type Item = ProvenanceTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_provenance_table_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of a provenance table.
///
/// These are not created directly.
/// Instead, use [`Provenance::provenances`]
/// to get a reference to an existing node table;
///
/// # Notes
///
/// * The type is enabled by the `"provenance"` feature.
///
pub struct ProvenanceTable<'a> {
    table_: &'a ll_bindings::tsk_provenance_table_t,
}

impl<'a> ProvenanceTable<'a> {
    pub(crate) fn new_from_table(provenances: &'a ll_bindings::tsk_provenance_table_t) -> Self {
        ProvenanceTable {
            table_: provenances,
        }
    }

    /// Return the number of rows
    pub fn num_rows(&'a self) -> ll_bindings::tsk_size_t {
        self.table_.num_rows
    }

    /// Get the ISO-formatted time stamp for row `row`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn timestamp<P: Into<ProvenanceId> + Copy>(&'a self, row: P) -> Result<String, TskitError> {
        match unsafe_tsk_ragged_char_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.timestamp,
            self.table_.timestamp_offset,
            self.table_.timestamp_length
        ) {
            Ok(Some(string)) => Ok(string),
            Ok(None) => Err(crate::TskitError::ValueError {
                got: String::from("None"),
                expected: String::from("String"),
            }),
            Err(e) => Err(e),
        }
    }

    /// Get the provenance record for row `row`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn record<P: Into<ProvenanceId> + Copy>(&'a self, row: P) -> Result<String, TskitError> {
        match unsafe_tsk_ragged_char_column_access!(
            row.into().0,
            0,
            self.num_rows(),
            self.table_.record,
            self.table_.record_offset,
            self.table_.record_length
        ) {
            Ok(Some(string)) => Ok(string),
            Ok(None) => Err(crate::TskitError::ValueError {
                got: String::from("None"),
                expected: String::from("String"),
            }),
            Err(e) => Err(e),
        }
    }

    /// Obtain a [`ProvenanceTableRow`] for row `row`.
    ///
    /// # Errors
    ///
    /// [`TskitError::IndexError`] if `r` is out of range.
    pub fn row<P: Into<ProvenanceId> + Copy>(
        &'a self,
        row: P,
    ) -> Result<ProvenanceTableRow, TskitError> {
        if row.into() < 0 {
            Err(TskitError::IndexError)
        } else {
            match make_provenance_table_row(self, row.into().0) {
                Some(x) => Ok(x),
                None => Err(TskitError::IndexError),
            }
        }
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`ProvenanceTableRow`].
    pub fn iter(&self) -> ProvenanceTableRefIterator {
        crate::table_iterator::make_table_iterator::<&ProvenanceTable<'a>>(&self)
    }
}
