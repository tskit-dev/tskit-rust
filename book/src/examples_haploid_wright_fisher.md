# Haploid Wright-Fisher simulation

The following code simulates a haploid Wright-Fisher model.
The code is a reimplementation of an example program distributed with `tskit-c`.

In addition to `tskit`, the example uses:

* [rand](https://crates.io/crates/rand) for random number generation.
* [anyhow](https://crates.io/crates/anyhow) for error propagation.

```rust, noplayground, ignore
{{#include ../../examples/haploid_wright_fisher.rs:haploid_wright_fisher}}
```
