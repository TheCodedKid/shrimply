use std::{env, fs, path::PathBuf};

use serde_json::Value;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shaders");
    println!("cargo:rerun-if-changed=../slang-build/reflect.cpp");
    println!("cargo:rerun-if-env-changed=SLANG_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=SLANG_BUILD_DIR");

    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let shader_directory = manifest.join("shaders");
    let source = shader_directory.join("reflection.slang");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let compiler = shrimply_slang_build::Compiler::new(&manifest, &output);
    let artifacts = compiler.reflect_cuda(&shader_directory, &source, &output, "reflect_abi");
    let reflection: Value = serde_json::from_slice(&artifacts.reflection)
        .unwrap_or_else(|error| panic!("parse compositor CUDA reflection: {error}"));
    fs::write(
        output.join("cuda_abi.rs"),
        shrimply_slang_build::generate_cuda_abi(&reflection, &artifacts.abi),
    )
    .expect("write reflected compositor CUDA ABI");
}
