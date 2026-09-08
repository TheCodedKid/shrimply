#[cfg(target_os = "macos")]
fn main() {
    use std::{env, fs, path::PathBuf};

    println!("cargo:rerun-if-changed=build.rs");
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let shaders = manifest.join("../render-3d-core/shaders");
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
        &[
            "obj_compute",
            "obj_composite_upload_compute",
            "obj_denoise_composite_compute",
            "obj_outline_distance_compute",
            "obj_outline_compute",
        ],
    );
    fs::write(
        output.join("obj_metal.rs"),
        shrimply_slang_build::generate_metal_module(
            "obj_metal",
            &artifact.filename,
            &artifact.reflection,
            &[
                (
                    "SceneUniforms",
                    "shrimply_render_3d_core::obj::SceneUniforms",
                ),
                ("PointLight", "shrimply_render_3d_core::obj::PointLight"),
                ("SunLight", "shrimply_render_3d_core::obj::SunLight"),
                ("Ground", "shrimply_render_3d_core::obj::Ground"),
                (
                    "EnvironmentSettings",
                    "shrimply_render_3d_core::obj::EnvironmentSettings",
                ),
                ("PbrSettings", "shrimply_render_3d_core::obj::PbrSettings"),
                ("ToonSettings", "shrimply_render_3d_core::obj::ToonSettings"),
                ("OutputSize", "super::OutputSize"),
                ("ComputeMaterial", "super::ComputeMaterial"),
                ("ComputePbr", "super::ComputePbr"),
                (
                    "TextureMapping",
                    "shrimply_render_3d_core::obj::TextureMapping",
                ),
                ("MeshInstance", "shrimply_render_3d_core::obj::MeshInstance"),
            ],
        ),
    )
    .expect("write OBJ Metal source include");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
