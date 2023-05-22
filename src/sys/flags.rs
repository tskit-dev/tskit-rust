use crate::sys::bindings as ll_bindings;
use crate::RawFlags;
use bitflags::bitflags;

macro_rules! impl_from_for_flag_types {
    ($flagstype: ty) => {
        impl From<$crate::RawFlags> for $flagstype {
            fn from(value: $crate::RawFlags) -> Self {
                <$flagstype>::from_bits_truncate(value)
            }
        }
    };
}

macro_rules! impl_flags {
    ($flagstype: ty) => {
        impl $flagstype {
            /// We do not enforce valid flags in the library.
            /// This function will return `true` if any bits
            /// are set that do not correspond to allowed flags.
            pub fn is_valid(&self) -> bool {
                Self::from_bits(self.bits()).is_some()
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

bitflags! {
    /// Control the behavior of [`crate::TableCollection::simplify`]
    /// and [`crate::TreeSequence::simplify`]
    ///
    /// Inclusion of values sets an option to `true`.
    /// The default behavior (`NONE`) is to perform the algorithm
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
    /// let flags = tskit::SimplificationOptions::default();
    /// assert_eq!(flags, tskit::SimplificationOptions::NONE);
    /// ```
    ///
    /// ### Using a "builder" API
    ///
    /// ```
    /// let flags =
    /// tskit::SimplificationOptions::default().keep_unary().filter_populations().filter_sites();
    /// assert!(flags.contains(tskit::SimplificationOptions::KEEP_UNARY));
    /// assert!(flags.contains(tskit::SimplificationOptions::FILTER_POPULATIONS));
    /// assert!(flags.contains(tskit::SimplificationOptions::FILTER_SITES));
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct SimplificationOptions: RawFlags {
        /// Default behavior
        const NONE = 0;
        const FILTER_SITES = ll_bindings::TSK_SIMPLIFY_FILTER_SITES;
        /// If True, remove any populations that are not referenced by
        /// nodes after simplification; new population IDs are allocated
        /// sequentially from zero.
        /// If False, the population table will not be altered in any way.
        const FILTER_POPULATIONS = ll_bindings::TSK_SIMPLIFY_FILTER_POPULATIONS;
        /// If True, remove any individuals that are not referenced by nodes
        /// after simplification; new individual IDs are allocated sequentially
        /// from zero. If False, the individual table will not be altered in any way.
        const FILTER_INDIVIDUALS = ll_bindings::TSK_SIMPLIFY_FILTER_INDIVIDUALS;
        /// Whether to reduce the topology down to the trees that are present at sites.
        const REDUCE_TO_SITE_TOPOLOGY = ll_bindings::TSK_SIMPLIFY_REDUCE_TO_SITE_TOPOLOGY;
        /// If True, preserve unary nodes (i.e. nodes with exactly one child)
        /// that exist on the path from samples to root.
        const KEEP_UNARY  = ll_bindings::TSK_SIMPLIFY_KEEP_UNARY;
        /// Whether to retain history ancestral to the MRCA of the samples.
        const KEEP_INPUT_ROOTS = ll_bindings::TSK_SIMPLIFY_KEEP_INPUT_ROOTS;
        ///  If True, preserve unary nodes that exist on the path from samples
        ///  to root, but only if they are associated with an individual
        ///  in the individuals table.
        ///  Cannot be specified at the same time as `KEEP_UNARY`.
        const KEEP_UNARY_IN_INDIVIDUALS  = ll_bindings::TSK_SIMPLIFY_KEEP_UNARY_IN_INDIVIDUALS;
    }
}

impl SimplificationOptions {
    flag_builder_api!(
    /// Update to set [`KEEP_INPUT_ROOTS`](crate::SimplificationOptions::KEEP_INPUT_ROOTS).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().keep_input_roots();
    /// assert!(f.contains(tskit::SimplificationOptions::KEEP_INPUT_ROOTS));
    /// ```
    => keep_input_roots, KEEP_INPUT_ROOTS);

    flag_builder_api!(
    /// Update to set [`KEEP_UNARY`](crate::SimplificationOptions::KEEP_UNARY).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().keep_unary();
    /// assert!(f.contains(tskit::SimplificationOptions::KEEP_UNARY));
    /// ```
    => keep_unary, KEEP_UNARY);

    flag_builder_api!(
    /// Update to set [`KEEP_UNARY_IN_INDIVIDUALS`](crate::SimplificationOptions::KEEP_UNARY_IN_INDIVIDUALS).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().keep_unary_in_individuals();
    /// assert!(f.contains(tskit::SimplificationOptions::KEEP_UNARY_IN_INDIVIDUALS));
    /// ```
    => keep_unary_in_individuals, KEEP_UNARY_IN_INDIVIDUALS);

    flag_builder_api!(
    /// Update to set [`FILTER_POPULATIONS`](crate::SimplificationOptions::FILTER_POPULATIONS).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().filter_populations();
    /// assert!(f.contains(tskit::SimplificationOptions::FILTER_POPULATIONS));
    /// ```
    => filter_populations, FILTER_POPULATIONS);

    flag_builder_api!(
    /// Update to set [`FILTER_SITES`](crate::SimplificationOptions::FILTER_SITES).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().filter_sites();
    /// assert!(f.contains(tskit::SimplificationOptions::FILTER_SITES));
    /// ```
    => filter_sites, FILTER_SITES);

    flag_builder_api!(
    /// Update to set [`REDUCE_TO_SITE_TOPOLOGY`](crate::SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().reduce_to_site_topology();
    /// assert!(f.contains(tskit::SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY));
    /// ```
    => reduce_to_site_topology, REDUCE_TO_SITE_TOPOLOGY);

    flag_builder_api!(
    /// Update to set [`FILTER_INDIVIDUALS`](crate::SimplificationOptions::FILTER_INDIVIDUALS).
    ///
    /// # Examples
    ///
    /// ```
    /// let f = tskit::SimplificationOptions::default().filter_individuals();
    /// assert!(f.contains(tskit::SimplificationOptions::FILTER_INDIVIDUALS));
    /// ```
    => filter_individuals, FILTER_INDIVIDUALS);
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::clear`].
    ///
    /// # Examples
    ///
    /// ## Set default (empty) flags
    ///
    /// ```
    /// let f = tskit::TableClearOptions::default();
    /// assert_eq!(f, tskit::TableClearOptions::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// ```
    /// let f = tskit::TableClearOptions::default().clear_metadata_schema();
    /// assert_eq!(f, tskit::TableClearOptions::CLEAR_METADATA_SCHEMAS);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableClearOptions::default().clear_ts_metadata_and_schema();
    /// assert_eq!(f, tskit::TableClearOptions::CLEAR_TS_METADATA_SCHEMA);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableClearOptions::default().clear_provenance();
    /// assert_eq!(f, tskit::TableClearOptions::CLEAR_PROVENANCE);
    ///
    /// ```
    /// let f = tskit::TableClearOptions::default().clear_metadata_schema().clear_ts_metadata_and_schema();
    /// assert!(f.contains(tskit::TableClearOptions::CLEAR_METADATA_SCHEMAS));
    /// assert!(f.contains(tskit::TableClearOptions::CLEAR_TS_METADATA_SCHEMA));
    /// let f = f.clear();
    /// assert!(f.contains(tskit::TableClearOptions::CLEAR_METADATA_SCHEMAS));
    /// assert!(f.contains(tskit::TableClearOptions::CLEAR_TS_METADATA_SCHEMA));
    /// assert!(f.contains(tskit::TableClearOptions::CLEAR_PROVENANCE);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TableClearOptions : RawFlags {
        /// Default behavior.
        const NONE = 0;
        const CLEAR_METADATA_SCHEMAS = ll_bindings::TSK_CLEAR_METADATA_SCHEMAS;
        const CLEAR_TS_METADATA_SCHEMA = ll_bindings::TSK_CLEAR_TS_METADATA_AND_SCHEMA;
        const CLEAR_PROVENANCE = ll_bindings::TSK_CLEAR_PROVENANCE;
    }
}

impl TableClearOptions {
    flag_builder_api!(
        /// Set [`CLEAR_METADATA_SCHEMAS`](crate::TableClearOptions::CLEAR_METADATA_SCHEMAS)
        => clear_metadata_schema, CLEAR_METADATA_SCHEMAS);
    flag_builder_api!(
        /// Set [`CLEAR_TS_METADATA_SCHEMA`](crate::TableClearOptions::CLEAR_TS_METADATA_SCHEMA)
        => clear_ts_metadata_and_schema, CLEAR_TS_METADATA_SCHEMA);
    flag_builder_api!(
        /// Set [`CLEAR_PROVENANCE`](crate::TableClearOptions::CLEAR_PROVENANCE)
        => clear_provenance, CLEAR_PROVENANCE);
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::equals`].
    ///
    /// # Examples
    ///
    /// ## Set default (empty) flags
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default();
    /// assert_eq!(f, tskit::TableEqualityOptions::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default().ignore_metadata();
    /// assert_eq!(f, tskit::TableEqualityOptions::IGNORE_METADATA);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default().ignore_ts_metadata();
    /// assert_eq!(f, tskit::TableEqualityOptions::IGNORE_TS_METADATA);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default().ignore_timestamps();
    /// assert_eq!(f, tskit::TableEqualityOptions::IGNORE_TIMESTAMPS);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default().ignore_provenance();
    /// assert_eq!(f, tskit::TableEqualityOptions::IGNORE_PROVENANCE);
    /// let f = f.ignore_metadata();
    /// assert!(f.contains(tskit::TableEqualityOptions::IGNORE_PROVENANCE));
    /// assert!(f.contains(tskit::TableEqualityOptions::IGNORE_METADATA));
    /// ```
    ///
    /// ### Method chaining
    ///
    /// ```
    /// let f = tskit::TableEqualityOptions::default().ignore_provenance().ignore_metadata();
    /// assert!(f.contains(tskit::TableEqualityOptions::IGNORE_PROVENANCE));
    /// assert!(f.contains(tskit::TableEqualityOptions::IGNORE_METADATA));
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TableEqualityOptions : RawFlags {
        /// Default behavior.
        const NONE = 0;
        const IGNORE_METADATA = ll_bindings::TSK_CMP_IGNORE_METADATA;
        const IGNORE_TS_METADATA = ll_bindings::TSK_CMP_IGNORE_TS_METADATA;
        const IGNORE_PROVENANCE = ll_bindings::TSK_CMP_IGNORE_PROVENANCE;
        const IGNORE_TIMESTAMPS = ll_bindings::TSK_CMP_IGNORE_TIMESTAMPS;
    }
}

impl TableEqualityOptions {
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
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::sort`].
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::TableSortOptions::default();
    /// assert_eq!(f, tskit::TableSortOptions::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// These methods can all be chained.
    ///
    /// ```
    /// let f = tskit::TableSortOptions::default().no_check_integrity();
    /// assert_eq!(f, tskit::TableSortOptions::NO_CHECK_INTEGRITY);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TableSortOptions : RawFlags {
        /// Default behavior.
        const NONE = 0;
        /// Do not validate contents of edge table.
        const NO_CHECK_INTEGRITY = ll_bindings::TSK_NO_CHECK_INTEGRITY;
    }
}

impl TableSortOptions {
    flag_builder_api!(
        /// Set [`NO_CHECK_INTEGRITY`](crate::TableSortOptions::NO_CHECK_INTEGRITY)
        => no_check_integrity, NO_CHECK_INTEGRITY);
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::sort_individuals`].
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::IndividualTableSortOptions::default();
    /// assert_eq!(f, tskit::IndividualTableSortOptions::NONE);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct IndividualTableSortOptions : RawFlags {
        /// Default behavior.
        const NONE = 0;
    }
}

bitflags! {
    /// Specify the behavior of iterating over [`Tree`] objects.
    /// See [`TreeSequence::tree_iterator`].
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::TreeFlags::default();
    /// assert_eq!(f, tskit::TreeFlags::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// These methods can be chained.
    ///
    /// ```
    /// let f = tskit::TreeFlags::default().sample_lists();
    /// assert_eq!(f, tskit::TreeFlags::SAMPLE_LISTS);
    /// ```
    ///
    /// ```
    /// let f = tskit::TreeFlags::default().no_sample_counts();
    /// assert_eq!(f, tskit::TreeFlags::NO_SAMPLE_COUNTS);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TreeFlags: RawFlags {
        /// Default behavior.
        const NONE = 0;
        /// Update sample lists, enabling [`Tree::samples`].
        const SAMPLE_LISTS = ll_bindings::TSK_SAMPLE_LISTS;
        /// Do *not* update the number of samples descending
        /// from each node. The default is to update these
        /// counts.
        const NO_SAMPLE_COUNTS = ll_bindings::TSK_NO_SAMPLE_COUNTS;
    }
}

impl TreeFlags {
    flag_builder_api!(
        /// Set [`SAMPLE_LISTS`](crate::TreeFlags::SAMPLE_LISTS)
        => sample_lists, SAMPLE_LISTS);
    flag_builder_api!(
        /// Set [`NO_SAMPLE_COUNTS`](crate::TreeFlags::NO_SAMPLE_COUNTS)
        => no_sample_counts, NO_SAMPLE_COUNTS);
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::dump`].
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::TableOutputOptions::default();
    /// assert_eq!(f, tskit::TableOutputOptions::NONE);
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
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TableOutputOptions : RawFlags {
        const NONE = 0;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::tree_sequence`]
    /// and [`crate::TreeSequence::new`].
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::TreeSequenceFlags::default();
    /// assert_eq!(f, tskit::TreeSequenceFlags::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// These methods may be chained.
    ///
    /// ```
    /// let f = tskit::TreeSequenceFlags::default().build_indexes();
    /// assert_eq!(f, tskit::TreeSequenceFlags::BUILD_INDEXES);
    /// ```
    ///
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TreeSequenceFlags: RawFlags {
        /// Default behavior
        const NONE = 0;
        /// If used, then build table indexes if they are not present.
        const BUILD_INDEXES = ll_bindings::TSK_TS_INIT_BUILD_INDEXES;
    }
}

impl TreeSequenceFlags {
    flag_builder_api!(
        /// Set [`BUILD_INDEXES`](crate::TreeSequenceFlags::BUILD_INDEXES)
        => build_indexes, BUILD_INDEXES);
}

bitflags! {
    /// Flags to affect the behavior of
    /// [`TableCollection::check_integrity`](crate::TableCollection::check_integrity).
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::NONE);
    /// ```
    ///
    /// ## Builder API
    ///
    /// These methods may be chained.
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_edge_ordering();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_EDGE_ORDERING);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_site_ordering();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_SITE_ORDERING);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_site_duplicates();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_SITE_DUPLICATES);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_mutation_ordering();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_MUTATION_ORDERING);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_individual_ordering();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_INDIVIDUAL_ORDERING);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_migration_ordering();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_MIGRATION_ORDERING);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_indexes();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_INDEXES);
    /// ```
    ///
    /// ```
    /// let f = tskit::TableIntegrityCheckFlags::default().check_trees();
    /// assert_eq!(f, tskit::TableIntegrityCheckFlags::CHECK_TREES);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct TableIntegrityCheckFlags: RawFlags {
        /// Default behavior is a set of basic checks
        const NONE = 0;
        /// Check that edges are ordered
        const CHECK_EDGE_ORDERING =ll_bindings::TSK_CHECK_EDGE_ORDERING;
        /// Check that sites are ordered
        const CHECK_SITE_ORDERING =ll_bindings::TSK_CHECK_SITE_ORDERING;
        /// Check for duplicated sites
        const CHECK_SITE_DUPLICATES=ll_bindings::TSK_CHECK_SITE_DUPLICATES;
        /// Check that mutations are ordered
        const CHECK_MUTATION_ORDERING =ll_bindings::TSK_CHECK_MUTATION_ORDERING;
        /// Check that individuals are ordered
        const CHECK_INDIVIDUAL_ORDERING=ll_bindings::TSK_CHECK_INDIVIDUAL_ORDERING;
        /// Check that migrations are ordered
        const CHECK_MIGRATION_ORDERING= ll_bindings::TSK_CHECK_MIGRATION_ORDERING;
        /// Check that table indexes are valid
        const CHECK_INDEXES=ll_bindings::TSK_CHECK_INDEXES;
        /// Check tree integrity.  Enables most or all of the preceding options.
        const CHECK_TREES=ll_bindings::TSK_CHECK_TREES;
    }
}

impl TableIntegrityCheckFlags {
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
}

bitflags! {
    /// Node flags
    ///
    /// # Examples
    ///
    /// ## Default (empty) flags
    ///
    /// ```
    /// let f = tskit::NodeFlags::default();
    /// assert_eq!(f, tskit::NodeFlags::NONE);
    /// ```
    ///
    /// ## Create a sample node
    ///
    /// Creating a sample node is such a common task that it is supported
    /// via a constructor:
    ///
    /// ```
    /// let f = tskit::NodeFlags::new_sample();
    /// assert_eq!(f, tskit::NodeFlags::IS_SAMPLE);
    /// ```
    ///
    /// ## Buider API
    ///
    /// These methods can be chained.
    ///
    /// ```
    /// let f = tskit::NodeFlags::default().mark_sample();
    /// assert_eq!(f, tskit::NodeFlags::IS_SAMPLE);
    /// ```
    #[derive(Default)]
    #[repr(transparent)]
    pub struct NodeFlags : RawFlags {
        /// Default (empty)
        const NONE = 0;
        /// Node is a sample
        const IS_SAMPLE = ll_bindings::TSK_NODE_IS_SAMPLE;
    }
}

impl NodeFlags {
    /// Create a new flags instance with `IS_SAMPLE` set.
    pub fn new_sample() -> Self {
        Self::default().mark_sample()
    }

    flag_builder_api!(
        /// Set [`IS_SAMPLE`](crate::NodeFlags::IS_SAMPLE)
        ///
        /// # Note
        ///
        /// This function is called `mark_sample` to not conflict
        /// with [`NodeFlags::is_sample`], which predates it.
        => mark_sample, IS_SAMPLE);

    /// We do not enforce valid flags in the library.
    /// This function will return `true` if any bits
    /// are set that do not correspond to allowed flags.
    pub fn is_valid(&self) -> bool {
        true
    }

    /// Returns `true` if flags contains `IS_SAMPLE`,
    /// and `false` otherwise.
    pub fn is_sample(&self) -> bool {
        self.contains(NodeFlags::IS_SAMPLE)
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(transparent)]
    /// Individual flags
    pub struct IndividualFlags : RawFlags {
        /// Default (empty)
        const NONE = 0;
    }
}

impl IndividualFlags {
    /// We do not enforce valid flags in the library.
    /// This function will return `true` if any bits
    /// are set that do not correspond to allowed flags.
    pub fn is_valid(&self) -> bool {
        true
    }
}

impl_flags!(SimplificationOptions);
impl_flags!(TableClearOptions);
impl_flags!(TableEqualityOptions);
impl_flags!(TreeSequenceFlags);
impl_flags!(TableSortOptions);
impl_flags!(TreeFlags);
impl_flags!(IndividualTableSortOptions);
impl_flags!(TableIntegrityCheckFlags);
impl_flags!(TableOutputOptions);

impl_from_for_flag_types!(SimplificationOptions);
impl_from_for_flag_types!(TableClearOptions);
impl_from_for_flag_types!(TableEqualityOptions);
impl_from_for_flag_types!(TreeSequenceFlags);
impl_from_for_flag_types!(TableSortOptions);
impl_from_for_flag_types!(TreeFlags);
impl_from_for_flag_types!(IndividualTableSortOptions);
impl_from_for_flag_types!(TableIntegrityCheckFlags);
impl_from_for_flag_types!(TableOutputOptions);

impl From<RawFlags> for NodeFlags {
    fn from(flags: RawFlags) -> Self {
        // Safety: node flags can contain user-defined values.
        // It is an error on the user's part to define flags
        // in the first 16 bits, as per the C API docs.
        unsafe { Self::from_bits_unchecked(flags) }
    }
}

impl From<RawFlags> for IndividualFlags {
    fn from(flags: RawFlags) -> Self {
        // Safety: node flags can contain user-defined values.
        // It is an error on the user's part to define flags
        // in the first 16 bits, as per the C API docs.
        unsafe { Self::from_bits_unchecked(flags) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
