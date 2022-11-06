# Metadata schema

For useful data interchange with `tskit-python`, we need to define [metadata schema](https://tskit.dev/tskit/docs/stable/metadata.html).

There are currently several points slowing down a rust API for schema:

* It is not clear which `serde` formats are compatible with metadata on the Python side.
* Experiments have shown that `serde_json` works with `tskit-python`.
  * Ideally, we would also like a binary format compatible with the Python `struct`
    module.
* However, we have not found a solution eliminating the need to manually write the
  schema as a string and add it to the tables.
  Various crates to generate JSON schema from rust structs return schema that are over-specified
  and fail to validate in `tskit-python`.
* We also have the problem that we will need to add some Python to our CI to prove to ourselves
  that some reasonable tests can pass.

