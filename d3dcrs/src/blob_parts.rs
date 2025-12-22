//! Blob part extraction and modification API

use crate::{Blob, Error, HResult, Result};
use d3dcompiler::{D3DGetBlobPart, D3DSetBlobPart, ID3DBlob, S_OK};
use std::ptr;

/// Blob part types for D3DGetBlobPart/D3DSetBlobPart
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BlobPart {
    /// Input signature
    InputSignature = 0,
    /// Output signature
    OutputSignature = 1,
    /// Input and output signature combined
    InputAndOutputSignature = 2,
    /// Patch constant signature (for tessellation shaders)
    PatchConstantSignature = 3,
    /// All signatures combined
    AllSignatures = 4,
    /// Debug information
    DebugInfo = 5,
    /// Legacy shader (SM 1.x-3.x)
    LegacyShader = 6,
    /// XNA prepass shader
    XnaPrepassShader = 7,
    /// XNA shader
    XnaShader = 8,
    /// Program database (PDB) data
    Pdb = 9,
    /// Private data section
    PrivateData = 10,
    /// Root signature
    RootSignature = 11,
    /// Debug name
    DebugName = 12,
}

/// Extracts a specific part from compiled shader bytecode.
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, get_blob_part, BlobPart, ShaderTarget};
///
/// let bytecode = compile(
///     "float4 main(float4 pos : SV_POSITION) : SV_TARGET { return pos; }",
///     "main",
///     ShaderTarget::PS_5_0
/// ).unwrap();
///
/// // Extract the input signature
/// if let Ok(input_sig) = get_blob_part(&bytecode, BlobPart::InputSignature) {
///     println!("Input signature: {} bytes", input_sig.len());
/// }
/// ```
pub fn get_blob_part(bytecode: &[u8], part: BlobPart) -> Result<Blob> {
    get_blob_part_with_flags(bytecode, part, 0)
}

/// Extracts a specific part from compiled shader bytecode with flags.
pub fn get_blob_part_with_flags(bytecode: &[u8], part: BlobPart, flags: u32) -> Result<Blob> {
    unsafe {
        let mut blob: *mut ID3DBlob = ptr::null_mut();

        let result = D3DGetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            part as u32,
            flags,
            &mut blob,
        );

        if result != S_OK {
            return Err(Error::GetBlobPart {
                hresult: HResult(result),
            });
        }

        Blob::from_raw(blob).ok_or(Error::GetBlobPart {
            hresult: HResult(result),
        })
    }
}

/// Replaces or inserts a part in compiled shader bytecode.
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, set_blob_part, BlobPart, ShaderTarget};
///
/// let bytecode = compile(
///     "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
///     "main",
///     ShaderTarget::PS_5_0
/// ).unwrap();
///
/// // Add private data to the shader
/// let private_data = b"My custom metadata";
/// let modified = set_blob_part(&bytecode, BlobPart::PrivateData, private_data).unwrap();
///
/// println!("Original: {} bytes, Modified: {} bytes",
///     bytecode.len(), modified.len());
/// ```
pub fn set_blob_part(bytecode: &[u8], part: BlobPart, data: &[u8]) -> Result<Blob> {
    set_blob_part_with_flags(bytecode, part, 0, data)
}

/// Replaces or inserts a part in compiled shader bytecode with flags.
pub fn set_blob_part_with_flags(
    bytecode: &[u8],
    part: BlobPart,
    flags: u32,
    data: &[u8],
) -> Result<Blob> {
    unsafe {
        let mut blob: *mut ID3DBlob = ptr::null_mut();

        let result = D3DSetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            part as u32,
            flags,
            data.as_ptr() as *const _,
            data.len(),
            &mut blob,
        );

        if result != S_OK {
            return Err(Error::SetBlobPart {
                hresult: HResult(result),
            });
        }

        Blob::from_raw(blob).ok_or(Error::SetBlobPart {
            hresult: HResult(result),
        })
    }
}

/// Extracts the input signature from compiled bytecode.
pub fn get_input_signature(bytecode: &[u8]) -> Result<Blob> {
    get_blob_part(bytecode, BlobPart::InputSignature)
}

/// Extracts the output signature from compiled bytecode.
pub fn get_output_signature(bytecode: &[u8]) -> Result<Blob> {
    get_blob_part(bytecode, BlobPart::OutputSignature)
}

/// Extracts debug info from compiled bytecode (if present).
pub fn get_debug_info(bytecode: &[u8]) -> Result<Blob> {
    get_blob_part(bytecode, BlobPart::DebugInfo)
}

/// Extracts private data from compiled bytecode (if present).
pub fn get_private_data(bytecode: &[u8]) -> Result<Blob> {
    get_blob_part(bytecode, BlobPart::PrivateData)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ShaderTarget, compile};

    const VERTEX_SHADER: &str = r#"
        struct VS_INPUT {
            float3 pos : POSITION;
            float2 uv : TEXCOORD0;
        };

        struct VS_OUTPUT {
            float4 pos : SV_POSITION;
            float2 uv : TEXCOORD0;
        };

        VS_OUTPUT main(VS_INPUT input) {
            VS_OUTPUT output;
            output.pos = float4(input.pos, 1.0);
            output.uv = input.uv;
            return output;
        }
    "#;

    #[test]
    fn test_get_input_signature() {
        let bytecode = compile(VERTEX_SHADER, "main", ShaderTarget::VS_5_0).unwrap();

        let result = get_input_signature(&bytecode);
        if let Ok(sig) = result {
            assert!(!sig.is_empty());
            println!("Input signature: {} bytes", sig.len());
        }
    }

    #[test]
    fn test_get_output_signature() {
        let bytecode = compile(VERTEX_SHADER, "main", ShaderTarget::VS_5_0).unwrap();

        let result = get_output_signature(&bytecode);
        if let Ok(sig) = result {
            assert!(!sig.is_empty());
            println!("Output signature: {} bytes", sig.len());
        }
    }

    #[test]
    fn test_set_private_data() {
        let bytecode = compile(
            "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
            "main",
            ShaderTarget::PS_5_0,
        )
        .unwrap();

        let private_data = b"Test private data!";
        let result = set_blob_part(&bytecode, BlobPart::PrivateData, private_data);

        if let Ok(modified) = result {
            assert!(modified.len() >= bytecode.len());
            assert_eq!(&modified[0..4], b"DXBC", "Should still be valid DXBC");
        }
    }
}
