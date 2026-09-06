#[cfg(target_os = "macos")]
fn main() {
    use std::{env, fs, path::PathBuf};

    println!("cargo:rerun-if-changed=build.rs");
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let shaders = manifest.join("../render-3d/shaders");
    let source = shaders.join("obj_compute.slang");
    println!("cargo:rerun-if-changed={}", source.display());
    println!(
        "cargo:rerun-if-changed={}",
        shaders.join("modules").display()
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"));
    let compiler = shrimply_slang_build::Compiler::new(&shaders, &output);
    let artifact = compiler.compile_as(
        &source,
        "obj_compute_metal",
        shrimply_slang_build::Target::Metal,
        &["obj_compute"],
    );
    fs::write(
        output.join("obj_metal.rs"),
        shrimply_slang_build::generate_metal_module(
            "obj_metal",
            &artifact.filename,
            &artifact.reflection,
        ),
    )
    .expect("write OBJ Metal source include");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
