## Iterating over trees

To iterate over trees left-to-right:

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_trees}}
```

This API depends on traits most easily brought into scope via the crate prelude:

```rust, noplayground, ignore
use tskit::prelude::*;
```

A `next_back()` function allows iteration to the next tree left of the current tree.
We currently do not have an API expressing "build me an iterator starting from the rightmost tree".
Such an thing is certainly doable.
