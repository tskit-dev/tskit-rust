# Row ID types

In the C and Python tskit libraries, adding a row to a table returns a "row" id.
In C/Python, this id is an integer.
On the C side, the integer type is `tsk_id_t`, which is currently 32 bits, equivalent to rust's `i32`.

The rust library defines a *different* id type for each table!
The node table has "node id", the edge table has an "edge id", etc..
This "strong typing" pattern prevents you from confusing your row id types.
For example, you can never mistakenly use a vector of individual ids in functions expecting vectors of node ids.
Trying to do so will fail to compile.

## Node IDs

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:create_node_id}}
```

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:create_null_node_id}}
```

## Vectors of IDs

The row ID types have some features that will appear magical when viewed through the lens of C.
A vector of IDs can be treated as a raw C array of the underlying integer type with no performance overhead!

First, let's create a vector of node IDs to play with:

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:create_vec_node_id}}
```

A `Vec<NodeId>` can be passed to rust functions in the usual fashion:

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:mock_rust_fn}}
```

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:call_mock_rust_fn}}
```

Consider the following two C functions that model common patterns in the C API:

```c
/* A function that does something with an array of "samples", which means
   nodes in the C API */
void tsk_foo(const tsk_id_t * samples, tsk_size_t num_samples) {}
/* A function that may modify the input */
void tsk_foo2(tsk_id_t * samples, tsk_size_t num_samples) {}
```

We can call these functions directly, but we have to cast to the correct pointer and size types:

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:call_mock_tsk_fn}}
```

```rust, noplayground, ignore
{{#include ../../../examples/row_id_types.rs:call_mock_tsk_fn_mut}}
```

In summary:

* The row ids are a "zero cost abstraction".
  We get improved type safety with no runtime cost.
* The example C functions shown above show where the type safety is gained.
  For the C functions, you only know if "sample" refers to "nodes" or
  "individuals" from the documentation.
  For rust functions accessing the C API, they only accept vectors/slices
  of the correct ID type.
  Code trying to pass in the wrong ID type will fail to compile.
* The examples here only mention `NodeId`, but the same principles apply to `EdgeId`, `SiteId`, `MutationId`, etc..

