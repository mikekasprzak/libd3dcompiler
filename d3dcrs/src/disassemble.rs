//! Shader disassembly API

use crate::{Blob, DisassembleFlags, Error, HResult, Result};
use d3dcompiler::{D3DDisassemble, ID3DBlob, S_OK};
use std::ffi::CString;
use std::ptr;

/// Builder for shader disassembly
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, DisassembleBuilder, DisassembleFlags, ShaderTarget};
///
/// let bytecode = compile(
///     "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
///     "main",
///     ShaderTarget::PS_5_0
/// ).unwrap();
///
/// let disasm = DisassembleBuilder::new(&bytecode)
///     .flags(DisassembleFlags::ENABLE_INSTRUCTION_NUMBERING)
///     .comment("My shader")
///     .disassemble()
///     .unwrap();
///
/// println!("{}", disasm.to_string_lossy());
/// ```
pub struct DisassembleBuilder<'a> {
    bytecode: &'a [u8],
    flags: DisassembleFlags,
    comment: Option<CString>,
}

impl<'a> DisassembleBuilder<'a> {
    /// Creates a new disassemble builder from compiled bytecode.
    pub fn new(bytecode: &'a [u8]) -> Self {
        DisassembleBuilder {
            bytecode,
            flags: DisassembleFlags::empty(),
            comment: None,
        }
    }

    /// Creates a new disassemble builder from a Blob.
    pub fn from_blob(blob: &'a Blob) -> Self {
        Self::new(blob.as_bytes())
    }

    /// Sets disassembly flags.
    pub fn flags(mut self, flags: DisassembleFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Adds disassembly flags.
    pub fn with_flags(mut self, flags: DisassembleFlags) -> Self {
        self.flags |= flags;
        self
    }

    /// Enables instruction numbering.
    pub fn with_instruction_numbering(self) -> Self {
        self.with_flags(DisassembleFlags::ENABLE_INSTRUCTION_NUMBERING)
    }

    /// Enables instruction offsets.
    pub fn with_instruction_offsets(self) -> Self {
        self.with_flags(DisassembleFlags::ENABLE_INSTRUCTION_OFFSET)
    }

    /// Enables color-coded output.
    pub fn with_color(self) -> Self {
        self.with_flags(DisassembleFlags::ENABLE_COLOR_CODE)
    }

    /// Sets a comment to include in the disassembly output.
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = Some(CString::new(comment).expect("Comment contains null byte"));
        self
    }

    /// Disassembles the bytecode.
    pub fn disassemble(self) -> Result<Blob> {
        unsafe {
            let mut disasm: *mut ID3DBlob = ptr::null_mut();

            let result = D3DDisassemble(
                self.bytecode.as_ptr() as *const _,
                self.bytecode.len(),
                self.flags.bits(),
                self.comment
                    .as_ref()
                    .map(|c| c.as_ptr())
                    .unwrap_or(ptr::null()),
                &mut disasm,
            );

            if result != S_OK {
                return Err(Error::Disassembly {
                    hresult: HResult(result),
                });
            }

            Blob::from_raw(disasm).ok_or(Error::Disassembly {
                hresult: HResult(result),
            })
        }
    }
}

/// Convenience function for simple disassembly.
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, disassemble, ShaderTarget};
///
/// let bytecode = compile(
///     "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
///     "main",
///     ShaderTarget::PS_5_0
/// ).unwrap();
///
/// let asm = disassemble(&bytecode).unwrap();
/// println!("{}", asm.to_string_lossy());
/// ```
pub fn disassemble(bytecode: &[u8]) -> Result<Blob> {
    DisassembleBuilder::new(bytecode).disassemble()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ShaderTarget, compile};

    #[test]
    fn test_disassemble() {
        let bytecode = compile(
            "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
            "main",
            ShaderTarget::PS_5_0,
        )
        .unwrap();

        let result = disassemble(&bytecode);
        assert!(result.is_ok());

        let asm = result.unwrap();
        let text = asm.to_string_lossy();
        assert!(text.contains("ps_5_0"), "Should contain shader model");
    }

    #[test]
    fn test_disassemble_with_options() {
        let bytecode = compile(
            "float4 main() : SV_TARGET { return float4(1,0,0,1); }",
            "main",
            ShaderTarget::PS_5_0,
        )
        .unwrap();

        let asm = DisassembleBuilder::new(&bytecode)
            .with_instruction_numbering()
            .comment("Test shader")
            .disassemble()
            .unwrap();

        let text = asm.to_string_lossy();
        assert!(!text.is_empty());
    }
}
