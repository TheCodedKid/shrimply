#[cfg(target_os = "linux")]
use std::{env, fs, path::PathBuf, process::Command};

#[cfg(target_os = "linux")]
use shrimply_slang_build::{Compiler, Target};

#[cfg(target_os = "linux")]
const DEFAULT_CUBIN_TARGET: &str = "sm_86";
#[cfg(target_os = "linux")]
const DEFAULT_PTX_TARGET: &str = "compute_50";
#[cfg(target_os = "linux")]
const MODULES: &str = include_str!("../render-core/shaders/kernels.txt");

#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    for variable in [
        "CUDA_IMAGE_FORMAT",
        "CUDA_TARGET",
        "CUDA_PTX_TARGET",
        "CUDA_HOST_CXX",
        "CUDA_ALLOW_UNSUPPORTED_COMPILER",
        "CUDA_HOME",
        "CUDA_TOOLKIT_PATH",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    let format = env::var("CUDA_IMAGE_FORMAT").unwrap_or_else(|_| "cubin".to_owned());
    let (target, extension, nvcc_output) = match format.as_str() {
        "cubin" => {
            let target =
                env::var("CUDA_TARGET").unwrap_or_else(|_| DEFAULT_CUBIN_TARGET.to_owned());
            assert!(
                target.starts_with("sm_"),
                "CUDA_TARGET must be a physical SM architecture"
            );
            (target, "cubin", "--cubin")
        }
        "ptx" => {
            let target =
                env::var("CUDA_PTX_TARGET").unwrap_or_else(|_| DEFAULT_PTX_TARGET.to_owned());
            assert!(
                target.starts_with("compute_"),
                "CUDA_PTX_TARGET must be a virtual compute architecture"
            );
            (target, "ptx", "--ptx")
        }
        _ => panic!("unsupported CUDA_IMAGE_FORMAT {format:?}; expected cubin or ptx"),
    };
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shaders = manifest.join("../render-core/shaders");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("CUDA build output"));
    let output = out.join("cuda").join(&target);
    fs::create_dir_all(&output).expect("create CUDA artifact directory");
    let compiler = Compiler::new(&shaders, &output);
    let toolkit = env::var_os("CUDA_TOOLKIT_PATH")
        .or_else(|| env::var_os("CUDA_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/cuda"));
    let host = env::var("CUDA_HOST_CXX").unwrap_or_else(|_| "g++-15".to_owned());
    let mut bindings = String::new();
    for module in MODULES.lines() {
        let source = shaders.join(format!("{module}.slang"));
        let artifact = compiler.compile(&source, Target::Cuda, &[]);
        let image = output.join(format!("{module}.{extension}"));
        let mut command = Command::new(toolkit.join("bin/nvcc"));
        command
            .arg(format!("--compiler-bindir={host}"))
            .args([nvcc_output, "-O2", "-w"])
            .arg(format!("--gpu-architecture={target}"));
        if env::var_os("CUDA_ALLOW_UNSUPPORTED_COMPILER").is_some_and(|value| !value.is_empty()) {
            command.arg("--allow-unsupported-compiler");
        }
        let status = command
            .arg(output.join(artifact.filename))
            .arg("-o")
            .arg(&image)
            .status()
            .expect("compile generated CUDA source with NVCC");
        assert!(
            status.success(),
            "compile CUDA kernel module {module}: {status}"
        );
        bindings.push_str(&format!(
            "pub const {}: &[u8] = include_bytes!({:?});\n",
            module.to_uppercase(),
            image
        ));
    }
    bindings.push_str(&format!("pub const IMAGE_FORMAT: &str = {format:?};\n"));
    bindings.push_str(&format!("pub const IMAGE_TARGET: &str = {target:?};\n"));
    fs::write(out.join("kernels.rs"), bindings).expect("write CUDA kernel bindings");
}
