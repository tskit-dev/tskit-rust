## Creation

We initialize a `TableCollection` with a sequence length.
In `tskit-c`, the genome length is a C `double`.
Here it is a [newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) called `tskit::Position`:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:create_table_collection_with_newtype}}
```

The newtype pattern gives type safety by disallowing you to send a position to a function where a time is required, etc..
However, it can be inconvenient to type out the full type names every time.
Thus, the API defines most functions taking arguments `Into<T>` where `T` is one of our newtypes.
This design means that the following is equivalent to what we wrote above:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:create_table_collection}}
```
