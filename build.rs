// build.rs
use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("labrador-c");

    // 1) Build the shared library via Makefile
    //    (this will produce `libdogs.so` in labrador-c)
    let status = Command::new("make")
        .arg("libdogs.so")
        .current_dir(&src)
        .status()
        .expect("failed to run make in labrador-c");
    assert!(status.success(), "make failed");

    // 2) Tell Cargo to link against libdogs.so
    println!("cargo:rustc-link-search=native={}", src.display());
    println!("cargo:rustc-link-lib=dylib=dogs");

    // 3) Generate bindings for the public header
    let bindings = bindgen::Builder::default()
        .header(src.join("labrador.h").to_string_lossy())
        .clang_arg(format!("-I{}", src.display()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings.rs");
}
