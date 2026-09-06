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
    let reflection: serde_json::Value =
        serde_json::from_slice(&artifact.reflection).expect("parse OBJ Metal reflection");
    let parameters = reflection["parameters"]
        .as_array()
        .expect("OBJ Metal reflection parameters");
    let binding = |name: &str| {
        let parameter = parameters
            .iter()
            .find(|parameter| parameter["name"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("OBJ Metal reflection omitted {name}"));
        assert_eq!(
            parameter["binding"]["kind"].as_str(),
            Some("constantBuffer"),
            "OBJ Metal resource {name} has an unexpected binding kind"
        );
        parameter["binding"]["index"]
            .as_u64()
            .unwrap_or_else(|| panic!("OBJ Metal reflection omitted {name} binding index"))
    };
    let entry = reflection["entryPoints"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["name"].as_str() == Some("obj_compute"))
        })
        .expect("OBJ Metal reflection omitted obj_compute");
    assert_eq!(
        entry["stage"].as_str(),
        Some("compute"),
        "OBJ Metal entry must be compute"
    );
    let threads = entry["threadGroupSize"]
        .as_array()
        .expect("OBJ Metal reflection omitted obj_compute thread group");
    let threads = threads
        .iter()
        .map(|value| value.as_u64().expect("OBJ Metal thread count") as usize)
        .collect::<Vec<_>>();
    assert_eq!(threads.len(), 3, "OBJ Metal thread group must be 3D");
    fs::write(
        output.join("obj_metal.rs"),
        format!(
            concat!(
                "pub const OBJ_METAL_SOURCE: &str = include_str!({:?});\n",
                "pub const OBJ_SCENE_BUFFER: usize = {};\n",
                "pub const OBJ_OUTPUT_SIZE_BUFFER: usize = {};\n",
                "pub const OBJ_ACCELERATION_BUFFER: usize = {};\n",
                "pub const OBJ_POSITIONS_BUFFER: usize = {};\n",
                "pub const OBJ_NORMALS_BUFFER: usize = {};\n",
                "pub const OBJ_MATERIALS_BUFFER: usize = {};\n",
                "pub const OBJ_INSTANCES_BUFFER: usize = {};\n",
                "pub const OBJ_OUTPUT_BUFFER: usize = {};\n",
                "pub const OBJ_THREADS: [usize; 3] = [{}, {}, {}];\n",
            ),
            output.join(artifact.filename).display().to_string(),
            binding("scene"),
            binding("output_size"),
            binding("scene_acceleration"),
            binding("positions"),
            binding("normals"),
            binding("materials"),
            binding("mesh_instances"),
            binding("output_pixels"),
            threads[0],
            threads[1],
            threads[2],
        ),
    )
    .expect("write OBJ Metal source include");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
