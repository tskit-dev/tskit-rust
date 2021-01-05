use tskit::*;

#[cfg(test)]
mod test_bookmark {

    use super::*;

    #[test]
    fn test_bookmark_mutability() {
        let mut b = types::Bookmark::new();
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

