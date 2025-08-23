# Metadata processing with Python

## `JSON` metadata

If your metadata are generated in `JSON` format via `serde` (see [here](metadata_derive.md)), then the metadata are simple to access from Python.
The code repository for `tskit-rust` contains examples in the `python/` subdirectory.

You may work with `JSON` metadata with or without a metadata schema (see [here](https://tskit.dev/tskit/docs/stable/metadata.html)).
A schema is useful for data validation but there is an unfortunate inefficiency if your input to Python is a tree sequence rather than a table collection.
You will have to copy the tables, add the metadata schema, and regenerate a tree sequence.
See the examples mentioned above.

## Other formats

The `tskit-python` API only supports `JSON` and Python's `struct` data formats.
It is useful to use a format other than `JSON` in order to minimize storage requirements.
However, doing so will require that you provide a method to covert the data into a valid Python object.

An easy way to provide conversion methods is to use [pyo3](https://pyo3.rs) to create a small Python module to deserialize your metadata into Python objects.
The `tskit-rust` code repository contains an example of this in the `python/` subdirectory.
The module is shown in its entirety below:

```rust, noplaygound, ignore
{{#include ../../python/tskit_glue/src/lib.rs}}
```

Using it in Python is just a matter of importing the module:

```python
{{#include ../../python/test_bincode_metadata.py}}
```
