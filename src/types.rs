//! "Other" tskit types live here.

use crate::bindings as ll_bindings;

/// A "bookmark" is used to adjust
/// the ranges over which some table algorithms
/// function.
///
/// For example, when
/// [``sort``](crate::TableCollection::sort)ing
/// tables, a bookmark can be used to indicate
/// the first row from which to begin.
/// The names of the fields are the same
/// names as tables in a TableCollection.
pub struct Bookmark {
    pub offsets: ll_bindings::tsk_bookmark_t,
}

macro_rules! bookmark_getter {
    ($name: ident) => {
        /// Get the current value
        pub fn $name(&self) -> $crate::SizeType {
            self.offsets.$name.into()
        }
    };
}

macro_rules! bookmark_setter {
    ($name: ident, $field: ident) => {
        /// Set the current value
        pub fn $name<I: Into<$crate::bindings::tsk_size_t>>(&mut self, value: I) {
            self.offsets.$field = value.into();
        }
    };
}

impl Bookmark {
    pub const fn new() -> Self {
        Bookmark {
            offsets: ll_bindings::tsk_bookmark_t {
                individuals: 0,
                nodes: 0,
                edges: 0,
                migrations: 0,
                sites: 0,
                mutations: 0,
                populations: 0,
                provenances: 0,
            },
        }
    }

    bookmark_getter!(individuals);
    bookmark_getter!(nodes);
    bookmark_getter!(edges);
    bookmark_getter!(migrations);
    bookmark_getter!(sites);
    bookmark_getter!(mutations);
    bookmark_getter!(populations);
    bookmark_getter!(provenances);
    bookmark_setter!(set_individuals, individuals);
    bookmark_setter!(set_nodes, nodes);
    bookmark_setter!(set_edges, edges);
    bookmark_setter!(set_migrations, migrations);
    bookmark_setter!(set_sites, sites);
    bookmark_setter!(set_mutations, mutations);
    bookmark_setter!(set_populations, populations);
    bookmark_setter!(set_provenances, provenances);
}

#[cfg(test)]
mod test {

    use super::*;

    macro_rules! test_set {
        ($bmark: ident, $setter: ident, $getter: ident) => {
            $bmark.$setter($crate::SizeType::from(3));
            assert_eq!($bmark.$getter(), 3);
        };
    }

    #[test]
    fn test_bookmark_mutability() {
        let mut b = Bookmark::new();
        assert_eq!(b.offsets.nodes, 0);
        assert_eq!(b.offsets.edges, 0);
        assert_eq!(b.offsets.individuals, 0);
        assert_eq!(b.offsets.migrations, 0);
        assert_eq!(b.offsets.sites, 0);
        assert_eq!(b.offsets.mutations, 0);
        assert_eq!(b.offsets.populations, 0);
        assert_eq!(b.offsets.provenances, 0);
        assert_eq!(b.nodes(), 0);
        assert_eq!(b.edges(), 0);
        assert_eq!(b.individuals(), 0);
        assert_eq!(b.migrations(), 0);
        assert_eq!(b.sites(), 0);
        assert_eq!(b.mutations(), 0);
        assert_eq!(b.populations(), 0);
        assert_eq!(b.provenances(), 0);

        test_set!(b, set_nodes, nodes);
        test_set!(b, set_edges, edges);
        test_set!(b, set_migrations, migrations);
        test_set!(b, set_sites, sites);
        test_set!(b, set_mutations, mutations);
        test_set!(b, set_populations, populations);
        test_set!(b, set_provenances, provenances);
        test_set!(b, set_individuals, individuals);
    }
}
