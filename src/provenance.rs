//! Optional module for table and tree sequence provenance tables.
//!
//! This module is enabled via the `"provenance"` feature and provides
//! the following:
//!
//! * [`crate::TableCollection::add_provenance`]
//! * [`crate::TableAccess::provenances`]
//! * [`crate::TableAccess::provenances_iter`]
//! * [`crate::TreeSequence::add_provenance`]
//! * [`ProvenanceTable`].
//! * [`ProvenanceTableRow`], which is the value type returned by
//!   [`ProvenanceTable::iter`].
//!

use crate::bindings as ll_bindings;
use crate::SizeType;
use crate::{tsk_id_t, tsk_size_t, ProvenanceId, TskitError};
use ll_bindings::{tsk_provenance_table_free, tsk_provenance_table_init};

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

impl std::fmt::Display for ProvenanceTableRow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "id: {}, timestamp: {}, record: {}",
            self.id, self.timestamp, self.record,
        )
    }
}

fn make_provenance_row(table: &ProvenanceTable, pos: tsk_id_t) -> Option<ProvenanceTableRow> {
    // panic is okay here, as we are handling a bad
    // input value before we first call this to
    // set up the iterator
    let p = crate::SizeType::try_from(pos).unwrap();
    if p < table.num_rows() {
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
        let rv = make_provenance_row(self.table, self.pos);
        self.pos += 1;
        rv
    }
}

impl<'a> Iterator for ProvenanceTableIterator<'a> {
    type Item = ProvenanceTableRow;

    fn next(&mut self) -> Option<Self::Item> {
        let rv = make_provenance_row(&self.table, self.pos);
        self.pos += 1;
        rv
    }
}

/// An immutable view of a provenance table.
///
/// These are not created directly.
/// Instead, use [`crate::TableAccess::provenances`]
/// to get a reference to an existing provenance table;
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
    pub fn num_rows(&'a self) -> SizeType {
        self.table_.num_rows.into()
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
            Ok(None) => Ok(String::from("")),
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
            match make_provenance_row(self, row.into().0) {
                Some(x) => Ok(x),
                None => Err(TskitError::IndexError),
            }
        }
    }

    /// Return an iterator over rows of the table.
    /// The value of the iterator is [`ProvenanceTableRow`].
    pub fn iter(&self) -> impl Iterator<Item = ProvenanceTableRow> + '_ {
        crate::table_iterator::make_table_iterator::<&ProvenanceTable<'a>>(self)
    }
}

build_owned_table_type!(
    /// A provenance table that owns its own data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(any(doc, feature = "provenance"))] {
    /// use tskit::provenance::OwnedProvenanceTable;
    /// let mut provenances = OwnedProvenanceTable::default();
    /// let id = provenances.add_row("message").unwrap();
    /// assert_eq!(id, 0);
    /// assert_eq!(provenances.num_rows(), 1);
    /// # }
    /// ```
    => OwnedProvenanceTable,
    ProvenanceTable,
    tsk_provenance_table_t,
    tsk_provenance_table_init,
    tsk_provenance_table_free,
    ll_bindings::tsk_provenance_table_clear
);

impl OwnedProvenanceTable {
    provenance_table_add_row!(=> add_row, self, *self.table);
}

#[cfg(test)]
mod test_provenances {
    use super::*;
    use crate::test_fixtures::make_empty_table_collection;
    use crate::TableAccess;

    #[test]
    fn test_empty_record_string() {
        // check for tables...
        let mut tables = make_empty_table_collection(1.0);
        let s = String::from("");
        let row_id = tables.add_provenance(&s).unwrap();
        let _ = tables.provenances().row(row_id).unwrap();

        // and for tree sequences...
        tables.build_index().unwrap();
        let mut ts = tables
            .tree_sequence(crate::TreeSequenceFlags::default())
            .unwrap();
        let row_id = ts.add_provenance(&s).unwrap();
        let _ = ts.provenances().row(row_id).unwrap();
    }

    #[test]
    fn test_add_rows() {
        let records = vec!["banana".to_string(), "split".to_string()];
        let mut tables = make_empty_table_collection(1.);
        for (i, r) in records.iter().enumerate() {
            let row_id = tables.add_provenance(r).unwrap();
            assert!(row_id == ProvenanceId(i as crate::tsk_id_t));
            assert_eq!(tables.provenances().record(row_id).unwrap(), *r);
        }
        assert_eq!(
            usize::try_from(tables.provenances().num_rows()).unwrap(),
            records.len()
        );
        for (i, row) in tables.provenances_iter().enumerate() {
            assert_eq!(records[i], row.record);
        }
        for (i, row) in tables.provenances().iter().enumerate() {
            assert_eq!(records[i], row.record);
        }

        assert!(tables.provenances().row(0).unwrap() == tables.provenances().row(0).unwrap());
        assert!(tables.provenances().row(0).unwrap() != tables.provenances().row(1).unwrap());
    }
}
