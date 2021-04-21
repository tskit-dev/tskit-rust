# Design

## Including the `tskit` `C` code.

We manually copy the source files from the current stable release.
We cannot use submodules because the `tskit` repository contains symbolic links, which causes problems with `bindgen`.

## Key principles

* Don't reinvent the wheel.
  If there is a `C` function in place, call it.
  Calling existing functions takes advantage of the high test coverage of `tskit`.
* Prefer rust idioms where possible.
  For example, provide iterator types instead of manual `next/advance` functions.
  See how `NodeIterator` works by looking in `src/traits.rs` and `src/trees.ts` for an example of a reusable iterator pattern.
