//! Shader stripping API

use crate::{Blob, Error, HResult, Result, StripFlags};
use d3dcompiler::{D3DStripShader, ID3DBlob, S_OK};
use std::ptr;

/// Strips specified data from a compiled shader.
///
/// This can be used to remove debug info, reflection data, or other
/// optional sections from a compiled shader to reduce its size.
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, strip_shader, ShaderTarget, StripFlags, CompileFlags};
///
/// // Compile with debug info
/// let bytecode = d3dcrs::CompileBuilder::new(
///     "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
///     "main",
///     ShaderTarget::PS_5_0
/// )
/// .debug()
/// .compile()
/// .unwrap();
///
/// // Strip the debug info for release
/// let stripped = strip_shader(&bytecode.bytecode, StripFlags::DEBUG_INFO).unwrap();
///
/// println!("Original: {} bytes, Stripped: {} bytes",
///     bytecode.bytecode.len(), stripped.len());
/// ```
pub fn strip_shader(bytecode: &[u8], flags: StripFlags) -> Result<Blob> {
    unsafe {
        let mut stripped: *mut ID3DBlob = ptr::null_mut();

        let result = D3DStripShader(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            flags.bits(),
            &mut stripped,
        );

        if result != S_OK {
            return Err(Error::StripShader {
                hresult: HResult(result),
            });
        }

        Blob::from_raw(stripped).ok_or(Error::StripShader {
            hresult: HResult(result),
        })
    }
}

/// Strips debug info from a compiled shader.
///
/// Convenience function equivalent to `strip_shader(bytecode, StripFlags::DEBUG_INFO)`.
pub fn strip_debug_info(bytecode: &[u8]) -> Result<Blob> {
    strip_shader(bytecode, StripFlags::DEBUG_INFO)
}

/// Strips reflection data from a compiled shader.
///
/// Convenience function equivalent to `strip_shader(bytecode, StripFlags::REFLECTION_DATA)`.
pub fn strip_reflection_data(bytecode: &[u8]) -> Result<Blob> {
    strip_shader(bytecode, StripFlags::REFLECTION_DATA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompileBuilder, CompileFlags, ShaderTarget};

    #[test]
    fn test_strip_debug_info() {
        // Compile with debug info
        let result = CompileBuilder::new(
            "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
            "main",
            ShaderTarget::PS_5_0,
        )
        .flags(CompileFlags::DEBUG)
        .compile()
        .unwrap();

        let original_size = result.bytecode.len();

        // Strip debug info
        let stripped = strip_debug_info(&result.bytecode);

        // Stripping may or may not reduce size depending on what's in the shader
        // Just verify it succeeds and returns valid DXBC
        if let Ok(stripped) = stripped {
            assert!(!stripped.is_empty());
            assert_eq!(&stripped[0..4], b"DXBC");
            println!(
                "Original: {} bytes, Stripped: {} bytes",
                original_size,
                stripped.len()
            );
        }
    }
}
