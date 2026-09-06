use std::{env, fmt::Write, fs, path::PathBuf};

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
    let compute_reflection: serde_json::Value =
        serde_json::from_slice(&compute.reflection).expect("Gaussian compute Metal reflection");
    let raster_reflection: serde_json::Value =
        serde_json::from_slice(&raster.reflection).expect("Gaussian raster Metal reflection");
    let raster_source = fs::read_to_string(output.join(&raster.filename))
        .expect("read Gaussian raster Metal source");
    assert!(
        !raster_source.contains("threadgroup"),
        "Gaussian raster Metal source contains compute threadgroup state"
    );

    let mut generated = format!(
        "pub const GAUSSIAN_COMPUTE_METAL_SOURCE: &str = include_str!({:?});\n\
         pub const GAUSSIAN_RASTER_METAL_SOURCE: &str = include_str!({:?});\n",
        output.join(compute.filename).display().to_string(),
        output.join(raster.filename).display().to_string(),
    );
    for (name, constant) in [
        ("gaussians", "COMPUTE_GAUSSIANS_BUFFER"),
        ("uniforms", "COMPUTE_UNIFORMS_BUFFER"),
        ("sort_keys", "COMPUTE_SORT_KEYS_BUFFER"),
        ("sorted_indices", "COMPUTE_SORTED_INDICES_BUFFER"),
        ("scratch_keys", "COMPUTE_SCRATCH_KEYS_BUFFER"),
        ("scratch_indices", "COMPUTE_SCRATCH_INDICES_BUFFER"),
        ("sort_group_offsets", "COMPUTE_SORT_GROUP_OFFSETS_BUFFER"),
        ("draw_indirect", "COMPUTE_DRAW_INDIRECT_BUFFER"),
        ("sort_constants", "COMPUTE_SORT_CONSTANTS_BUFFER"),
    ] {
        binding(
            &mut generated,
            &compute_reflection,
            name,
            constant,
            "constantBuffer",
        );
    }
    for (name, constant, kind) in [
        ("gaussians", "RASTER_GAUSSIANS_BUFFER", "constantBuffer"),
        (
            "higher_order_sh",
            "RASTER_HIGHER_ORDER_BUFFER",
            "constantBuffer",
        ),
        ("uniforms", "RASTER_UNIFORMS_BUFFER", "constantBuffer"),
        (
            "sorted_indices",
            "RASTER_SORTED_INDICES_BUFFER",
            "constantBuffer",
        ),
        (
            "accumulation_texture",
            "RASTER_ACCUMULATION_TEXTURE",
            "shaderResource",
        ),
    ] {
        binding(&mut generated, &raster_reflection, name, constant, kind);
    }
    for (name, constant) in [
        ("prepare_depth_sort", "PREPARE_SORT_THREADS"),
        ("radix_histogram_pass", "RADIX_HISTOGRAM_THREADS"),
        ("radix_prefix_pass", "RADIX_PREFIX_THREADS"),
        ("radix_scatter_pass", "RADIX_SCATTER_THREADS"),
    ] {
        entry(
            &compute_reflection,
            name,
            "compute",
            Some((&mut generated, constant)),
        );
    }
    for (name, stage) in [
        ("gaussian_vertex_metal", "vertex"),
        ("gaussian_fragment", "fragment"),
        ("resolve_vertex", "vertex"),
        ("resolve_fragment", "fragment"),
    ] {
        entry(&raster_reflection, name, stage, None);
    }
    fs::write(output.join("gaussian_metal.rs"), generated)
        .expect("write Gaussian Metal source include");
}

fn binding(
    generated: &mut String,
    reflection: &serde_json::Value,
    name: &str,
    constant: &str,
    kind: &str,
) {
    let parameter = reflection["parameters"]
        .as_array()
        .expect("Gaussian Metal parameters")
        .iter()
        .find(|parameter| parameter["name"] == name)
        .unwrap_or_else(|| panic!("Gaussian Metal reflection omitted {name}"));
    assert_eq!(
        parameter["binding"]["kind"], kind,
        "Gaussian Metal binding kind changed for {name}"
    );
    let index = parameter["binding"]["index"]
        .as_u64()
        .unwrap_or_else(|| panic!("Gaussian Metal binding index omitted for {name}"));
    writeln!(generated, "const {constant}: usize = {index};").unwrap();
}

fn entry(
    reflection: &serde_json::Value,
    name: &str,
    stage: &str,
    threads: Option<(&mut String, &str)>,
) {
    let entry = reflection["entryPoints"]
        .as_array()
        .expect("Gaussian Metal entry points")
        .iter()
        .find(|entry| entry["name"] == name)
        .unwrap_or_else(|| panic!("Gaussian Metal reflection omitted {name}"));
    assert_eq!(
        entry["stage"], stage,
        "Gaussian Metal stage changed for {name}"
    );
    if let Some((generated, constant)) = threads {
        let size = entry["threadGroupSize"]
            .as_array()
            .unwrap_or_else(|| panic!("Gaussian Metal thread group omitted for {name}"));
        assert_eq!(size.len(), 3, "Gaussian Metal thread group rank changed");
        writeln!(
            generated,
            "const {constant}: [usize; 3] = [{}, {}, {}];",
            size[0].as_u64().expect("thread group x"),
            size[1].as_u64().expect("thread group y"),
            size[2].as_u64().expect("thread group z")
        )
        .unwrap();
    }
}
