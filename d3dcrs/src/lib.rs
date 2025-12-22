//! Safe, ergonomic Rust API for D3DCompiler
//!
//! This crate provides a safe wrapper around the D3DCompiler library,
//! offering Rust idioms like Result types, RAII wrappers, iterators,
//! and builder patterns.
//!
//! # Example
//!
//! ```no_run
//! use d3dcrs::{compile, ShaderTarget, ShaderReflection};
//!
//! let source = r#"
//!     float4 main(float4 pos : SV_POSITION) : SV_TARGET {
//!         return pos;
//!     }
//! "#;
//!
//! // Compile a pixel shader
//! let bytecode = compile(source, "main", ShaderTarget::PS_5_0).unwrap();
//!
//! // Reflect on the compiled shader
//! let reflection = ShaderReflection::new(&bytecode).unwrap();
//! let desc = reflection.desc().unwrap();
//! println!("Instructions: {}", desc.instruction_count);
//! ```

mod blob;
mod blob_parts;
mod compile;
mod disassemble;
mod error;
mod flags;
mod include;
mod preprocess;
pub mod reflect;
mod strip;
mod target;

pub use blob::Blob;
pub use blob_parts::{
    BlobPart, get_blob_part, get_debug_info, get_input_signature, get_output_signature,
    get_private_data, set_blob_part,
};
pub use compile::{CompileBuilder, CompileResult, Define, compile};
pub use disassemble::{DisassembleBuilder, disassemble};
pub use error::{Error, HResult, Result};
pub use flags::{CompileFlags, DisassembleFlags, StripFlags};
pub use include::{FileSystemInclude, IncludeHandler, IncludeType, MemoryInclude};
pub use preprocess::{PreprocessBuilder, PreprocessResult, preprocess};
pub use reflect::ShaderReflection;
pub use strip::{strip_debug_info, strip_reflection_data, strip_shader};
pub use target::{ShaderModel, ShaderTarget, ShaderType};
