mod reflection;
mod slang;

pub use reflection::{generate_cuda_abi, generate_module};
pub use slang::{Compiler, CudaReflection, shader_sources};
