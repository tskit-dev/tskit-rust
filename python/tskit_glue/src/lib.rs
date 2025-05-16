// NOTES:
// This code shows how to decode metadata generated in rust
// using a format that tskit-python does NOT support.
//
// This example works by creating rust structs that exactly mimic
// our metadata types and are exposed to Python.
// For production code, it would be wiser to reuse the rust types
// that first generated the metadata.
// We cannot do that here, else we'd have to publish the crates
// defining those types to crates.io/PyPi, which is just
// ecosystem pollution.
//
// Importantly, deserialization does not require that our
// input/output types be identical!
// Rather, they simply have to have the same fields.
// We exploit this fact here, which allows us to make
// new types with the same fields as our metadata.

use pyo3::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[pyclass]
struct MutationMetadata {
    effect_size: f64,
    dominance: f64,
}

#[pymethods]
impl MutationMetadata {
    fn effect_size(&self) -> f64 {
        self.effect_size
    }
    fn dominance(&self) -> f64 {
        self.dominance
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[pyclass]
struct IndividualMetadata {
    name: String,
    phenotypes: Vec<i32>,
}

#[pymethods]
impl IndividualMetadata {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn phenotypes(&self) -> Vec<i32> {
        self.phenotypes.clone()
    }
}

/// Decode mutation metadata generated in rust via the `bincode` crate.
#[pyfunction]
fn decode_bincode_mutation_metadata(md: Vec<u8>) -> MutationMetadata {
    // NOTE: the unwrap here is not correct for production code
    // and a failure will crash the Python interpreter!
    let md = bincode::deserialize_from(md.as_slice()).unwrap();
    md
}

/// Decode individual metadata generated in rust via the `bincode` crate.
#[pyfunction]
fn decode_bincode_individual_metadata(md: Vec<u8>) -> IndividualMetadata {
    // NOTE: the unwrap here is not correct for production code
    // and a failure will crash the Python interpreter!
    let md = bincode::deserialize_from(md.as_slice()).unwrap();
    md
}

/// A Python module implemented in Rust.
#[pymodule]
fn tskit_glue(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_bincode_mutation_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bincode_individual_metadata, m)?)?;
    Ok(())
}
