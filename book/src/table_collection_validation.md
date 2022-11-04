## Checking table integrity

The data model involves lazy checking of inputs.
In other words, we can add invalid row data that is not caught by the "add row" functions.
We inherit this behavior from the C API.

We can check that the tables contain valid data by:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:integrity_check}}
```
