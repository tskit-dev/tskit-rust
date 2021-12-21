# Error handling

```rust, should_panic, noplayground
# extern crate tskit;
let mut tables = tskit::TableCollection::new(0.0).unwrap();
```

```rust, should_panic, noplayground
# extern crate tskit;
let mut tables = tskit::TableCollection::new(-1.0).unwrap();
```


