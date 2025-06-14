// build.rs
use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("labrador-c");

    // 1) Gather all .c files under labrador-c
    let c_files = vec![
        "greyhound.c",
        "dachshund.c",
        "pack.c",
        "chihuahua.c",
        "labrador.c",
        "data.c",
        "jlproj.c",
        "polx.c",
        "poly.c",
        "polz.c",
        "ntt.S",
        "invntt.S",
        "aesctr.c",
        "fips202.c",
        "randombytes.c",
        "cpucycles.c",
        "sparsemat.c",
    ]
    .into_iter()
    .map(|f| src.join(f))
    .collect::<Vec<_>>();

    // 2) Invoke gcc to build a shared lib
    let mut cmd = Command::new("gcc");
    cmd.args(&[
        "-std=gnu18",
        "-Wall",
        "-Wextra",
        "-Wmissing-prototypes",
        "-Wredundant-decls",
    ]);
    cmd.args(&[
        "-Wshadow",
        "-Wpointer-arith",
        "-Wno-unused-function",
        "-fmax-errors=1",
        "-flto=auto",
        "-fwrapv",
    ]);
    cmd.args(&[
        "-march=native",
        "-mtune=native",
        "-O3",
        "-fvisibility=hidden",
    ]);
    cmd.args(&["-fPIC", "-shared", "-DLOGQ=24", "-I", src.to_str().unwrap()]);
    for file in &c_files {
        cmd.arg(file);
    }
    cmd.arg("-o").arg(src.join("liblabrador24.so"));
    let status = cmd.status().expect("failed to build liblabrador24.so");
    assert!(status.success());

    // 3) Tell Cargo to link
    println!("cargo:rustc-link-search=native={}", src.display());
    println!("cargo:rustc-link-lib=dylib=labrador24");

    // 4) Generate bindings as before...
    let bindings = bindgen::Builder::default()
        .header(src.join("labrador.h").to_string_lossy())
        .clang_arg(format!("-I{}", src.display()))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings.rs");
}
