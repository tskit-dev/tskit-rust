// ANCHOR: use_prelude
// The prelude exports traits, etc., that are tedious to live without
use tskit::prelude::*;
// But, not all types are exported.
// Here, we specifically import TableCollection 
// to avoid constantly having to refer to the namespace.
use tskit::TableCollection;
// ANCHOR_END: use_prelude

fn initialize() -> tskit::TableCollection {
    // ANCHOR: initialization
    let tables = match TableCollection::new(1000.0) {
        Ok(t) => t,
        Err(e) => panic!("{e}"),
    };
    // ANCHOR_END: initialization

    tables
}

#[cfg(test)]
mod test_example_tables_without_metadata {

    use super::*;

    #[test]
    fn test_initialize() {
        let _ = initialize();
    }
}
