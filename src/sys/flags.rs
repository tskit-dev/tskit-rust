use crate::sys::bindings as ll_bindings;
use crate::RawFlags;

use super::bindings::tsk_flags_t;

use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;

macro_rules! make_constant_self {
    ($(#[$attr:meta])* => $name: ident, $constant: ident) => {
        $(#[$attr])*
        pub const $name: Self = Self(ll_bindings::$constant);
    }
}

macro_rules! impl_from_for_flag_types {
    ($flagstype: ty) => {
        impl From<$crate::RawFlags> for $flagstype {
            fn from(value: $crate::RawFlags) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! flag_builder_api {
    ($(#[$attr:meta])* => $name: ident, $flag: ident) => {
        $(#[$attr])*
        pub fn $name(self) -> Self {
            self | Self::$flag
        }
    };
}

macro_rules! bits {
    () => {
        pub fn bits(&self) -> RawFlags {
            self.0
        }
    };
}

macro_rules! all {
    () => {
        pub fn all() -> Self {
            Self(RawFlags::MAX)
        }
    };
}

macro_rules! contains {
    () => {
        pub fn contains<I>(&self, bit: I) -> bool
        where
            I: Into<Self> + Copy,
        {
            (self.0 & bit.into().0) != 0
        }
    };
}

macro_rules! impl_bit_ops {
    ($type: ty) => {
        impl BitXorAssign for $type {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0
            }
        }

        impl BitAndAssign for $type {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
        }

        impl BitOrAssign for $type {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0
            }
        }

        impl BitXor for $type {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl BitOr for $type {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl BitAnd for $type {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
    };
}

macro_rules! flags_api {
    ($type: ty) => {
        impl_from_for_flag_types!($type);
        impl_bit_ops!($type);
    };
}

/// Control the behavior of table simplification.
///
/// Inclusion of values sets an option to `true`.
/// The default behavior is to perform the algorithm
/// as described in Kelleher *et al.* (2018), 10.1371/journal.pcbi.1006581.
///
/// The documentation for each field is taken from the `tskit` primary
/// docs.
///
/// # Examples
///
/// ## Building up flags
///
/// ### Default flags
///
/// ```
/// # use tskit::SimplificationOptions;
/// let flags = SimplificationOptions::default();
/// ```
///
/// ### Using a "builder" API
///
/// ```
/// # use tskit::SimplificationOptions;
/// let flags =
/// SimplificationOptions::default().keep_unary().filter_populations().filter_sites();
/// assert!(flags.contains(SimplificationOptions::KEEP_UNARY));
/// assert!(flags.contains(SimplificationOptions::FILTER_POPULATIONS));
/// assert!(flags.contains(SimplificationOptions::FILTER_SITES));
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SimplificationOptions(RawFlags);

impl SimplificationOptions {
    make_constant_self!(=> FILTER_SITES, TSK_SIMPLIFY_FILTER_SITES);
    make_constant_self!(
    /// If True, remove any populations that are not referenced by
    /// nodes after simplification; new population IDs are allocated
    /// sequentially from zero.
    /// If False, the population table will not be altered in any way.
        => FILTER_POPULATIONS, TSK_SIMPLIFY_FILTER_POPULATIONS);
    make_constant_self!(
    /// If True, remove any individuals that are not referenced by nodes
    /// after simplification; new individual IDs are allocated sequentially
    /// from zero. If False, the individual table will not be altered in any way.
    => FILTER_INDIVIDUALS,TSK_SIMPLIFY_FILTER_INDIVIDUALS);
    make_constant_self!(
    /// Whether to reduce the topology down to the trees that are present at sites.
    => REDUCE_TO_SITE_TOPOLOGY,TSK_SIMPLIFY_REDUCE_TO_SITE_TOPOLOGY);
    make_constant_self!(
    /// If True, preserve unary nodes (i.e. nodes with exactly one child)
    /// that exist on the path from samples to root.
    => KEEP_UNARY,TSK_SIMPLIFY_KEEP_UNARY);
    make_constant_self!(
    /// Whether to retain history ancestral to the MRCA of the samples.
    => KEEP_INPUT_ROOTS, TSK_SIMPLIFY_KEEP_INPUT_ROOTS);
    make_constant_self!(
    ///  If True, preserve unary nodes that exist on the path from samples
    ///  to root, but only if they are associated with an individual
    ///  in the individuals table.
    ///  Cannot be specified at the same time as `KEEP_UNARY`.
    => KEEP_UNARY_IN_INDIVIDUALS,TSK_SIMPLIFY_KEEP_UNARY_IN_INDIVIDUALS);

    flag_builder_api!(
    /// Update to set [`KEEP_INPUT_ROOTS`](crate::SimplificationOptions::KEEP_INPUT_ROOTS).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().keep_input_roots();
    /// assert!(f.contains(SimplificationOptions::KEEP_INPUT_ROOTS));
    /// ```
    => keep_input_roots, KEEP_INPUT_ROOTS);

    flag_builder_api!(
    /// Update to set [`KEEP_UNARY`](crate::SimplificationOptions::KEEP_UNARY).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().keep_unary();
    /// assert!(f.contains(SimplificationOptions::KEEP_UNARY));
    /// ```
    => keep_unary, KEEP_UNARY);

    flag_builder_api!(
    /// Update to set [`KEEP_UNARY_IN_INDIVIDUALS`](crate::SimplificationOptions::KEEP_UNARY_IN_INDIVIDUALS).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().keep_unary_in_individuals();
    /// assert!(f.contains(SimplificationOptions::KEEP_UNARY_IN_INDIVIDUALS));
    /// ```
    => keep_unary_in_individuals, KEEP_UNARY_IN_INDIVIDUALS);

    flag_builder_api!(
    /// Update to set [`FILTER_POPULATIONS`](crate::SimplificationOptions::FILTER_POPULATIONS).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().filter_populations();
    /// assert!(f.contains(SimplificationOptions::FILTER_POPULATIONS));
    /// ```
    => filter_populations, FILTER_POPULATIONS);

    flag_builder_api!(
    /// Update to set [`FILTER_SITES`](crate::SimplificationOptions::FILTER_SITES).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().filter_sites();
    /// assert!(f.contains(SimplificationOptions::FILTER_SITES));
    /// ```
    => filter_sites, FILTER_SITES);

    flag_builder_api!(
    /// Update to set [`REDUCE_TO_SITE_TOPOLOGY`](crate::SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().reduce_to_site_topology();
    /// assert!(f.contains(SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY));
    /// ```
    => reduce_to_site_topology, REDUCE_TO_SITE_TOPOLOGY);

    flag_builder_api!(
    /// Update to set [`FILTER_INDIVIDUALS`](crate::SimplificationOptions::FILTER_INDIVIDUALS).
    ///
    /// # Examples
    ///
    /// ```
    /// # use tskit::SimplificationOptions;
    /// let f = SimplificationOptions::default().filter_individuals();
    /// assert!(f.contains(SimplificationOptions::FILTER_INDIVIDUALS));
    /// ```
    => filter_individuals, FILTER_INDIVIDUALS);

    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::clear`].
///
/// # Examples
///
/// ## Set default (empty) flags
///
/// ```
/// # use tskit::TableClearOptions;
/// let f = TableClearOptions::default();
/// ```
///
/// ## Builder API
///
/// ```
/// # use tskit::TableClearOptions;
/// let f = TableClearOptions::default().clear_metadata_schema();
/// assert_eq!(f, TableClearOptions::CLEAR_METADATA_SCHEMAS);
/// ```
///
/// ```
/// # use tskit::TableClearOptions;
/// let f = TableClearOptions::default().clear_ts_metadata_and_schema();
/// assert_eq!(f, TableClearOptions::CLEAR_TS_METADATA_SCHEMA);
/// ```
///
/// ```
/// # use tskit::TableClearOptions;
/// let f = TableClearOptions::default().clear_provenance();
/// assert_eq!(f, TableClearOptions::CLEAR_PROVENANCE);
/// ```
///
/// ```
/// # use tskit::TableClearOptions;
/// let f = TableClearOptions::default().clear_metadata_schema().clear_ts_metadata_and_schema();
/// assert!(f.contains(TableClearOptions::CLEAR_METADATA_SCHEMAS));
/// assert!(f.contains(TableClearOptions::CLEAR_TS_METADATA_SCHEMA));
/// # let f = TableClearOptions::default();
/// # assert!(!f.contains(TableClearOptions::CLEAR_METADATA_SCHEMAS));
/// # assert!(!f.contains(TableClearOptions::CLEAR_TS_METADATA_SCHEMA));
/// # assert!(!f.contains(TableClearOptions::CLEAR_PROVENANCE));
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TableClearOptions(RawFlags);

impl TableClearOptions {
    make_constant_self!(=> CLEAR_METADATA_SCHEMAS,TSK_CLEAR_METADATA_SCHEMAS);
    make_constant_self!(=> CLEAR_TS_METADATA_SCHEMA, TSK_CLEAR_TS_METADATA_AND_SCHEMA);
    make_constant_self!(=> CLEAR_PROVENANCE,TSK_CLEAR_PROVENANCE);
    flag_builder_api!(
        /// Set [`CLEAR_METADATA_SCHEMAS`](crate::TableClearOptions::CLEAR_METADATA_SCHEMAS)
        => clear_metadata_schema, CLEAR_METADATA_SCHEMAS);
    flag_builder_api!(
        /// Set [`CLEAR_TS_METADATA_SCHEMA`](crate::TableClearOptions::CLEAR_TS_METADATA_SCHEMA)
        => clear_ts_metadata_and_schema, CLEAR_TS_METADATA_SCHEMA);
    flag_builder_api!(
        /// Set [`CLEAR_PROVENANCE`](crate::TableClearOptions::CLEAR_PROVENANCE)
        => clear_provenance, CLEAR_PROVENANCE);
    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::equals`].
///
/// # Examples
///
/// ## Set default (empty) flags
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default();
/// ```
///
/// ## Builder API
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default().ignore_metadata();
/// assert_eq!(f, TableEqualityOptions::IGNORE_METADATA);
/// ```
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default().ignore_ts_metadata();
/// assert_eq!(f, TableEqualityOptions::IGNORE_TS_METADATA);
/// ```
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default().ignore_timestamps();
/// assert_eq!(f, TableEqualityOptions::IGNORE_TIMESTAMPS);
/// ```
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default().ignore_provenance();
/// assert_eq!(f, TableEqualityOptions::IGNORE_PROVENANCE);
/// let f = f.ignore_metadata();
/// assert!(f.contains(TableEqualityOptions::IGNORE_PROVENANCE));
/// assert!(f.contains(TableEqualityOptions::IGNORE_METADATA));
/// ```
///
/// ### Method chaining
///
/// ```
/// # use tskit::TableEqualityOptions;
/// let f = TableEqualityOptions::default().ignore_provenance().ignore_metadata();
/// assert!(f.contains(TableEqualityOptions::IGNORE_PROVENANCE));
/// assert!(f.contains(TableEqualityOptions::IGNORE_METADATA));
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TableEqualityOptions(RawFlags);

impl TableEqualityOptions {
    make_constant_self!(=> IGNORE_METADATA,TSK_CMP_IGNORE_METADATA);
    make_constant_self!(=>IGNORE_TS_METADATA, TSK_CMP_IGNORE_TS_METADATA);
    make_constant_self!(=> IGNORE_PROVENANCE,TSK_CMP_IGNORE_PROVENANCE);
    make_constant_self!(=> IGNORE_TIMESTAMPS,TSK_CMP_IGNORE_TIMESTAMPS);
    flag_builder_api!(
        /// Set [`IGNORE_METADATA`](crate::TableEqualityOptions::IGNORE_METADATA)
        => ignore_metadata, IGNORE_METADATA);
    flag_builder_api!(
        /// Set [`IGNORE_TS_METADATA`](crate::TableEqualityOptions::IGNORE_TS_METADATA)
        => ignore_ts_metadata, IGNORE_TS_METADATA);
    flag_builder_api!(
        /// Set [:IGNORE_PROVENANCE`](crate::TableEqualityOptions::IGNORE_PROVENANCE)
        => ignore_provenance, IGNORE_PROVENANCE);
    flag_builder_api!(
        /// Set [`IGNORE_TIMESTAMPS`](crate::TableEqualityOptions::IGNORE_TIMESTAMPS)
        => ignore_timestamps, IGNORE_TIMESTAMPS);
    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::sort`].
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::TableSortOptions;
/// let f = TableSortOptions::default();
/// ```
///
/// ## Builder API
///
/// These methods can all be chained.
///
/// ```
/// # use tskit::TableSortOptions;
/// let f = TableSortOptions::default().no_check_integrity();
/// assert_eq!(f, TableSortOptions::NO_CHECK_INTEGRITY);
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TableSortOptions(RawFlags);

impl TableSortOptions {
    make_constant_self!(
    /// Do not validate contents of edge table.
    => NO_CHECK_INTEGRITY, TSK_NO_CHECK_INTEGRITY);
    flag_builder_api!(
        /// Set [`NO_CHECK_INTEGRITY`](crate::TableSortOptions::NO_CHECK_INTEGRITY)
        => no_check_integrity, NO_CHECK_INTEGRITY);
    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::topological_sort_individuals`].
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::IndividualTableSortOptions;
/// let f = IndividualTableSortOptions::default();
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IndividualTableSortOptions(RawFlags);

impl IndividualTableSortOptions {
    bits!();
    all!();
    contains!();
}

/// Specify the behavior of iterating over [`crate::Tree`] objects.
/// See [`crate::TreeSequence::tree_iterator`].
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::TreeFlags;
/// let f = TreeFlags::default();
/// ```
///
/// ## Builder API
///
/// These methods can be chained.
///
/// ```
/// # use tskit::TreeFlags;
/// let f = TreeFlags::default().sample_lists();
/// assert_eq!(f, TreeFlags::SAMPLE_LISTS);
/// ```
///
/// ```
/// # use tskit::TreeFlags;
/// let f = TreeFlags::default().no_sample_counts();
/// assert_eq!(f, TreeFlags::NO_SAMPLE_COUNTS);
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TreeFlags(RawFlags);

impl TreeFlags {
    make_constant_self!(
    /// Update sample lists, enabling [`crate::Tree::samples`].
    => SAMPLE_LISTS,TSK_SAMPLE_LISTS);
    make_constant_self!(
    /// Do *not* update the number of samples descending
    /// from each node. The default is to update these
    /// counts.
    => NO_SAMPLE_COUNTS,TSK_NO_SAMPLE_COUNTS);
    flag_builder_api!(
        /// Set [`SAMPLE_LISTS`](crate::TreeFlags::SAMPLE_LISTS)
        => sample_lists, SAMPLE_LISTS);
    flag_builder_api!(
        /// Set [`NO_SAMPLE_COUNTS`](crate::TreeFlags::NO_SAMPLE_COUNTS)
        => no_sample_counts, NO_SAMPLE_COUNTS);
    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::dump`].
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::TableOutputOptions;
/// let f = TableOutputOptions::default();
/// # assert_eq!(f, TableOutputOptions::default());
/// ```
///
/// # Note
///
/// We intentionally do *not* provide the TSK_NO_BUILD_INDEXES
/// flag.  Rather, we treat the various "dump" functions as
/// operations on immutable objects.  Thus, if indexes are desired
/// when outputting a [`crate::TableCollection`], then
/// call [`crate::TableCollection::build_index`] prior to calling
/// [`crate::TableCollection::dump`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TableOutputOptions(RawFlags);

impl TableOutputOptions {
    bits!();
    all!();
    contains!();
}

/// Modify behavior of [`crate::TableCollection::tree_sequence`]
/// and [`crate::TreeSequence::new`].
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::TreeSequenceFlags;
/// let f = TreeSequenceFlags::default();
/// ```
///
/// ## Builder API
///
/// These methods may be chained.
///
/// ```
/// # use tskit::TreeSequenceFlags;
/// let f = TreeSequenceFlags::default().build_indexes();
/// assert_eq!(f, TreeSequenceFlags::BUILD_INDEXES);
/// ```
///
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TreeSequenceFlags(RawFlags);

impl TreeSequenceFlags {
    make_constant_self!(
    /// If used, then build table indexes if they are not present.
    => BUILD_INDEXES, TSK_TS_INIT_BUILD_INDEXES);
    flag_builder_api!(
        /// Set [`BUILD_INDEXES`](crate::TreeSequenceFlags::BUILD_INDEXES)
        => build_indexes, BUILD_INDEXES);
    bits!();
    all!();
    contains!();
}

/// Flags to affect the behavior of
/// [`TableCollection::check_integrity`](crate::TableCollection::check_integrity).
///
/// # Examples
///
/// ## Default (empty) flags
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default();
/// ```
///
/// ## Builder API
///
/// These methods may be chained.
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_edge_ordering();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_EDGE_ORDERING);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_site_ordering();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_SITE_ORDERING);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_site_duplicates();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_SITE_DUPLICATES);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_mutation_ordering();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_MUTATION_ORDERING);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_individual_ordering();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_migration_ordering();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_MIGRATION_ORDERING);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_indexes();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_INDEXES);
/// ```
///
/// ```
/// # use tskit::TableIntegrityCheckFlags;
/// let f = TableIntegrityCheckFlags::default().check_trees();
/// assert_eq!(f, TableIntegrityCheckFlags::CHECK_TREES);
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TableIntegrityCheckFlags(RawFlags);

impl TableIntegrityCheckFlags {
    make_constant_self!(
    /// Check that edges are ordered
     => CHECK_EDGE_ORDERING, TSK_CHECK_EDGE_ORDERING);
    make_constant_self!(
    /// Check that sites are ordered
    => CHECK_SITE_ORDERING, TSK_CHECK_SITE_ORDERING);
    make_constant_self!(
    /// Check for duplicated sites
    => CHECK_SITE_DUPLICATES, TSK_CHECK_SITE_DUPLICATES);
    make_constant_self!(
    /// Check that mutations are ordered
    => CHECK_MUTATION_ORDERING, TSK_CHECK_MUTATION_ORDERING);
    make_constant_self!(
    /// Check that individuals are ordered
    => CHECK_INDIVIDUAL_ORDERING, TSK_CHECK_INDIVIDUAL_ORDERING);
    make_constant_self!(
    /// Check that migrations are ordered
    => CHECK_MIGRATION_ORDERING, TSK_CHECK_MIGRATION_ORDERING);
    make_constant_self!(
    /// Check that table indexes are valid
    => CHECK_INDEXES, TSK_CHECK_INDEXES);
    make_constant_self!(
    /// Check tree integrity.  Enables most or all of the preceding options.
    => CHECK_TREES, TSK_CHECK_TREES);
    flag_builder_api!(
        /// Set [`CHECK_EDGE_ORDERING`](crate::TableIntegrityCheckFlags::CHECK_EDGE_ORDERING)
        => check_edge_ordering, CHECK_EDGE_ORDERING);
    flag_builder_api!(
        /// Set [`CHECK_SITE_ORDERING`](crate::TableIntegrityCheckFlags::CHECK_SITE_ORDERING)
        => check_site_ordering, CHECK_SITE_ORDERING);
    flag_builder_api!(
        /// Set [`CHECK_INDIVIDUAL_ORDERING`](crate::TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING)
        => check_individual_ordering, CHECK_INDIVIDUAL_ORDERING);
    flag_builder_api!(
        /// Set [`CHECK_MUTATION_ORDERING`](crate::TableIntegrityCheckFlags::CHECK_MUTATION_ORDERING)
        => check_mutation_ordering, CHECK_MUTATION_ORDERING);
    flag_builder_api!(
        /// Set [`CHECK_MIGRATION_ORDERING`](crate::TableIntegrityCheckFlags::CHECK_MIGRATION_ORDERING)
        => check_migration_ordering, CHECK_MIGRATION_ORDERING);
    flag_builder_api!(
        /// Set [`CHECK_SITE_DUPLICATES`](crate::TableIntegrityCheckFlags::CHECK_SITE_DUPLICATES)
        => check_site_duplicates, CHECK_SITE_DUPLICATES);
    flag_builder_api!(
        /// Set [`CHECK_INDEXES`](crate::TableIntegrityCheckFlags::CHECK_INDEXES)
        => check_indexes, CHECK_INDEXES);
    flag_builder_api!(
        /// Set [`CHECK_TREES`](crate::TableIntegrityCheckFlags::CHECK_TREES)
        => check_trees, CHECK_TREES);
    bits!();
    all!();
    contains!();
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeFlags(tsk_flags_t);

impl NodeFlags {
    make_constant_self!(
        /// Node is a sample
        => IS_SAMPLE, TSK_NODE_IS_SAMPLE);

    /// Create a new flags instance with `IS_SAMPLE` set.
    pub fn new_sample() -> Self {
        Self::default().mark_sample()
    }

    /// Set [`IS_SAMPLE`](crate::NodeFlags::IS_SAMPLE)
    ///
    /// # Note
    ///
    /// This function is called `mark_sample` to not conflict
    /// with [`NodeFlags::is_sample`], which predates it.
    pub fn mark_sample(self) -> Self {
        Self(self.0 | Self::IS_SAMPLE.0)
    }

    /// Returns `true` if flags contains `IS_SAMPLE`,
    /// and `false` otherwise.
    pub fn is_sample(&self) -> bool {
        self.contains(Self::IS_SAMPLE)
    }

    bits!();
    all!();
    contains!();

    pub fn toggle<I: Into<Self>>(&mut self, bit: I) {
        self.bitxor_assign(bit.into());
    }

    pub fn remove<I>(&mut self, bit: I)
    where
        I: Into<Self>,
    {
        self.0 &= !bit.into().0
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
/// Individual flags
pub struct IndividualFlags(RawFlags);

impl IndividualFlags {
    bits!();
    all!();
    contains!();
}

flags_api!(SimplificationOptions);
flags_api!(TableClearOptions);
flags_api!(TableEqualityOptions);
flags_api!(TreeSequenceFlags);
flags_api!(TableSortOptions);
flags_api!(TreeFlags);
flags_api!(IndividualTableSortOptions);
flags_api!(TableIntegrityCheckFlags);
flags_api!(TableOutputOptions);
flags_api!(NodeFlags);

impl From<RawFlags> for IndividualFlags {
    fn from(flags: RawFlags) -> Self {
        Self(flags)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MutationParentsFlags(tsk_flags_t);

impl From<MutationParentsFlags> for tsk_flags_t {
    fn from(value: MutationParentsFlags) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_looks_like_zero() {
        let n = NodeFlags::default();
        assert_eq!(n.bits(), 0);
        let s = SimplificationOptions::default();
        assert_eq!(s.bits(), 0);
    }

    #[test]
    fn node_is_not_sample() {
        let n = NodeFlags::default();
        assert!(!n.is_sample());
    }

    #[test]
    fn node_is_sample() {
        let n = NodeFlags::new_sample();
        assert!(n.is_sample());
    }
}
