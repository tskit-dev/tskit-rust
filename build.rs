use std::path::Path;

fn main() {
    pkg_config::Config::new()
        .atleast_version("1.2");

    let src = [
        "subprojects/tskit/c/tskit/convert.c",
        "subprojects/tskit/c/tskit/core.c",
        "subprojects/tskit/c/tskit/genotypes.c",
        "subprojects/tskit/c/tskit/haplotype_matching.c",
        "subprojects/tskit/c/tskit/stats.c",
        "subprojects/tskit/c/tskit/tables.c",
        "subprojects/tskit/c/tskit/trees.c",
        "subprojects/tskit/c/subprojects/kastore/kastore.c",
    ];

    let tskit_path = Path::new("subprojects/tskit/c");
    let kastore_path = Path::new("subprojects/tskit/c/subprojects/kastore");
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include(tskit_path)
        .include(kastore_path)
        .flag("-Wno-unused-parameter");
    build.compile("tskit");
}
