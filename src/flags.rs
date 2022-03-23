use crate::bindings as ll_bindings;
use crate::tsk_flags_t;
use bitflags::bitflags;

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
    /// ```
    /// // Initial flags set to 0:
    /// let mut flags = tskit::SimplificationOptions::default();
    ///
    /// // Add some options:
    /// flags.insert(tskit::SimplificationOptions::KEEP_UNARY);
    /// flags.insert(tskit::SimplificationOptions::FILTER_POPULATIONS);
    ///
    /// assert!(flags.contains(tskit::SimplificationOptions::KEEP_UNARY));
    /// assert!(flags.contains(tskit::SimplificationOptions::FILTER_POPULATIONS));
    /// ```
    ///
    /// ## All-in-one initialization
    ///
    /// ```
    /// use tskit::SimplificationOptions as SO;
    /// let flags = SO::FILTER_SITES | SO::KEEP_UNARY;
    /// assert!(flags.contains(SO::FILTER_SITES));
    /// assert!(flags.contains(SO::KEEP_UNARY));
    /// assert!(!flags.contains(SO::FILTER_POPULATIONS));
    /// ```
    #[derive(Default)]
    pub struct SimplificationOptions: tsk_flags_t {
        /// Default behavior
        const NONE = 0;
        const FILTER_SITES = ll_bindings::TSK_FILTER_SITES;
        /// If True, remove any populations that are not referenced by
        /// nodes after simplification; new population IDs are allocated
        /// sequentially from zero.
        /// If False, the population table will not be altered in any way.
        const FILTER_POPULATIONS = ll_bindings::TSK_FILTER_POPULATIONS;
        /// If True, remove any individuals that are not referenced by nodes
        /// after simplification; new individual IDs are allocated sequentially
        /// from zero. If False, the individual table will not be altered in any way.
        const FILTER_INDIVIDUALS = ll_bindings::TSK_FILTER_INDIVIDUALS;
        /// Whether to reduce the topology down to the trees that are present at sites.
        const REDUCE_TO_SITE_TOPOLOGY = ll_bindings::TSK_REDUCE_TO_SITE_TOPOLOGY;
        /// If True, preserve unary nodes (i.e. nodes with exactly one child)
        /// that exist on the path from samples to root.
        const KEEP_UNARY  = ll_bindings::TSK_KEEP_UNARY;
        /// Whether to retain history ancestral to the MRCA of the samples.
        const KEEP_INPUT_ROOTS = ll_bindings::TSK_KEEP_INPUT_ROOTS;
        ///  If True, preserve unary nodes that exist on the path from samples
        ///  to root, but only if they are associated with an individual
        ///  in the individuals table.
        ///  Cannot be specified at the same time as `KEEP_UNARY`.
        const KEEP_UNARY_IN_INDIVIDUALS  = ll_bindings::TSK_KEEP_UNARY_IN_INDIVIDUALS;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::clear`].
    #[derive(Default)]
    pub struct TableClearOptions : tsk_flags_t {
        /// Default behavior.
        const NONE = 0;
        const CLEAR_METADATA_SCHEMAS = ll_bindings::TSK_CLEAR_METADATA_SCHEMAS;
        const CLEAR_TS_METADATA_SCHEMAS = ll_bindings::TSK_CLEAR_TS_METADATA_AND_SCHEMA;
        const CLEAR_PROVENANCE = ll_bindings::TSK_CLEAR_PROVENANCE;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::equals`].
    #[derive(Default)]
    pub struct TableEqualityOptions : tsk_flags_t {
        /// Default behavior.
        const NONE = 0;
        const IGNORE_METADATA = ll_bindings::TSK_CMP_IGNORE_METADATA;
        const IGNORE_TS_METADATA = ll_bindings::TSK_CMP_IGNORE_TS_METADATA;
        const IGNORE_PROVENANCE = ll_bindings::TSK_CMP_IGNORE_PROVENANCE;
        const IGNORE_TIMESTAMPS = ll_bindings::TSK_CMP_IGNORE_TIMESTAMPS;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::sort`].
    #[derive(Default)]
    pub struct TableSortOptions : tsk_flags_t {
        /// Default behavior.
        const NONE = 0;
        /// Do not validate contents of edge table.
        const NO_CHECK_INTEGRITY = ll_bindings::TSK_NO_CHECK_INTEGRITY;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::sort_individuals`].
    #[derive(Default)]
    pub struct IndividualTableSortOptions : tsk_flags_t {
        /// Default behavior.
        const NONE = 0;
    }
}

bitflags! {
    /// Specify the behavior of iterating over [`Tree`] objects.
    /// See [`TreeSequence::tree_iterator`].
    #[derive(Default)]
    pub struct TreeFlags: tsk_flags_t {
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

bitflags! {
    /// Modify behavior of [`crate::TableCollection::dump`].
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
    pub struct TableOutputOptions : tsk_flags_t {
        const NONE = 0;
    }
}

bitflags! {
    /// Modify behavior of [`crate::TableCollection::tree_sequence`]
    /// and [`crate::TreeSequence::new`].
    #[derive(Default)]
    pub struct TreeSequenceFlags: tsk_flags_t {
        /// Default behavior
        const NONE = 0;
        /// If used, then build table indexes if they are not present.
        const BUILD_INDEXES = ll_bindings::TSK_BUILD_INDEXES;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct TableIntegrityCheckFlags: tsk_flags_t {
        /// Default behavior is a set of basic checks
        const DEFAULT = 0;
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
