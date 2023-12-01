extern crate bindgen;
use std::path::Path;

fn main() {
    pkg_config::Config::new().atleast_version("1.2");

    let src = [
        "subprojects/tskit/tskit/convert.c",
        "subprojects/tskit/tskit/core.c",
        "subprojects/tskit/tskit/genotypes.c",
        "subprojects/tskit/tskit/haplotype_matching.c",
        "subprojects/tskit/tskit/stats.c",
        "subprojects/tskit/tskit/tables.c",
        "subprojects/tskit/tskit/trees.c",
        "subprojects/kastore/kastore.c",
    ];

    let tskit_path = Path::new("subprojects/tskit/");
    let kastore_path = Path::new("subprojects/kastore/");
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include(tskit_path)
        .include(kastore_path)
        .flag("-Wno-unused-parameter");
    build.compile("tskit");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg("-Isubprojects/tskit")
        .clang_arg("-Isubprojects/kastore")
        .allowlist_type("tsk.*")
        .allowlist_function("tsk.*")
        .allowlist_type("TSK_.*")
        .allowlist_var("TSK_.*")
        .allowlist_type("KAS_.*")
        .allowlist_var("KAS_.*")
        .allowlist_type("kastore.*")
        .allowlist_function("kastore.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("auto_bindings.rs"))
        .expect("Couldn't write bindings!");
}
