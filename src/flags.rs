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
