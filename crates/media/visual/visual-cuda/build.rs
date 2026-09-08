use std::{env, fs, path::PathBuf};

use serde_json::Value;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../gpu/slang/slang-build/compiler.cpp");
    println!("cargo:rerun-if-changed=../../../render/render-core/shaders");

    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let shared_shader_directory = manifest.join("../../../render/render-core/shaders");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let compiler = shrimply_slang_build::Compiler::new(&shared_shader_directory, &output);
    let source = shared_shader_directory.join("mesh_flow.slang");
    println!("cargo:rerun-if-changed={}", source.display());
    let mut bindings = String::from("// @generated from video Slang reflection.\n");
    let module = source
        .file_stem()
        .and_then(|name| name.to_str())
        .expect("video Slang module filename must be UTF-8");
    let artifacts = compiler.compile(&source, shrimply_slang_build::Target::Spirv, &[]);
    let reflected: Value = serde_json::from_slice(&artifacts.reflection)
        .unwrap_or_else(|error| panic!("parse video Slang reflection for {module}: {error}"));
    bindings.push_str(&shrimply_slang_build::generate_module(
        module,
        &artifacts.filename,
        &reflected,
        &artifacts.abi,
    ));
    fs::write(output.join("slang_bindings.rs"), bindings)
        .expect("write reflected video Slang Rust bindings");
}
