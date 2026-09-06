use std::{env, fs, path::PathBuf};

const COMPUTE_ENTRIES: &[&str] = &[
    "prepare_depth_sort",
    "radix_histogram_pass",
    "radix_prefix_pass",
    "radix_scatter_pass",
];
const RASTER_ENTRIES: &[&str] = &[
    "gaussian_vertex_metal",
    "gaussian_fragment",
    "resolve_vertex",
    "resolve_fragment",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let shaders = manifest.join("../3dgs/shaders");
    let source = shaders.join("gaussian.slang");
    println!("cargo:rerun-if-changed={}", source.display());
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"));
    let compiler = shrimply_slang_build::Compiler::new(&shaders, &output);
    let compute = compiler.compile_as(
        &source,
        "gaussian_compute",
        shrimply_slang_build::Target::Metal,
        COMPUTE_ENTRIES,
    );
    let raster = compiler.compile_as(
        &source,
        "gaussian_raster",
        shrimply_slang_build::Target::Metal,
        RASTER_ENTRIES,
    );
    let raster_source = fs::read_to_string(output.join(&raster.filename))
        .expect("read Gaussian raster Metal source");
    assert!(
        !raster_source.contains("threadgroup"),
        "Gaussian raster Metal source contains compute threadgroup state"
    );
    let generated = shrimply_slang_build::generate_metal_module(
        "gaussian_compute",
        &compute.filename,
        &compute.reflection,
        &[],
    ) + &shrimply_slang_build::generate_metal_module(
        "gaussian_raster",
        &raster.filename,
        &raster.reflection,
        &[],
    );
    fs::write(output.join("gaussian_metal.rs"), generated)
        .expect("write Gaussian Metal reflection module");
}
