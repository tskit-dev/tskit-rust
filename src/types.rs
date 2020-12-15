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

impl Bookmark {
    pub const fn new() -> Self {
        return Bookmark {
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
        };
    }
}

#[cfg(test)]
mod test {

    use super::*;

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
        b.offsets.nodes = 3;
        assert_eq!(b.offsets.nodes, 3);
    }
}
