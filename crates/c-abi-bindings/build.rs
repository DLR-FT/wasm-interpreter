extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let common_builder = cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_no_includes()
        .with_sys_include("stdint.h");

    // build C bindings
    common_builder
        .clone()
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");

    // build C++ bindings
    common_builder
        .clone()
        .with_language(cbindgen::Language::Cxx)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.hpp");
}
