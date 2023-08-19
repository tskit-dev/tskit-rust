## Advanced topics

### Bulk decoding of metadata

To populate a `Vec` with decoded metadata and accounting
for some rows not having metadata:

* Realize that the decode methods of the `MetadataRoundtrip` trait are associated functions.
* We can use a lending iterator over rows to avoid unnecessary copying.

Therefore:

```rust, noplayground, ignore
{{#include ../../tests/book_metadata.rs:metadata_bulk_decode_lending_iter}}
```
  
To filter out rows without metadata:

```rust, noplayground, ignore
{{#include ../../tests/book_metadata.rs:metadata_bulk_decode_lending_iter_with_filter}}
```

The first method gives `Vec<Option<MutationMetadata>>`.
The second gives `Vec<(MutationId, MutationMetadata)>`.

