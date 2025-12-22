//! Shader compilation API

use crate::{Blob, CompileFlags, Error, HResult, Result, ShaderTarget};
use d3dcompiler::{D3D_SHADER_MACRO, D3DCompile, ID3DBlob, ID3DInclude, S_OK};
use std::ffi::CString;
use std::ptr;

/// A preprocessor macro definition
#[derive(Debug, Clone)]
pub struct Define {
    pub(crate) name: CString,
    pub(crate) value: CString,
}

impl Define {
    /// Creates a new preprocessor define
    ///
    /// # Example
    /// ```
    /// use d3dcrs::Define;
    /// let define = Define::new("DEBUG", "1");
    /// ```
    pub fn new(name: &str, value: &str) -> Self {
        Define {
            name: CString::new(name).expect("Define name contains null byte"),
            value: CString::new(value).expect("Define value contains null byte"),
        }
    }

    /// Creates a define with an empty value
    pub fn flag(name: &str) -> Self {
        Self::new(name, "")
    }
}

/// Result of a successful shader compilation
#[derive(Debug)]
pub struct CompileResult {
    /// The compiled shader bytecode
    pub bytecode: Blob,
    /// Any warning messages from the compiler (if present)
    pub warnings: Option<String>,
}

/// Builder for shader compilation with fluent API
///
/// # Example
/// ```no_run
/// use d3dcrs::{CompileBuilder, CompileFlags, ShaderTarget};
///
/// let source = "float4 main() : SV_TARGET { return float4(1,0,0,1); }";
///
/// let result = CompileBuilder::new(source, "main", ShaderTarget::PS_5_0)
///     .source_name("my_shader.hlsl")
///     .define("DEBUG", "1")
///     .flags(CompileFlags::DEBUG | CompileFlags::WARNINGS_ARE_ERRORS)
///     .optimization_level(3)
///     .compile()
///     .unwrap();
/// ```
pub struct CompileBuilder<'a> {
    source: &'a [u8],
    source_name: Option<CString>,
    entry_point: CString,
    target: ShaderTarget,
    defines: Vec<Define>,
    include: Option<*mut ID3DInclude>,
    flags1: CompileFlags,
    flags2: u32,
}

impl<'a> CompileBuilder<'a> {
    /// Creates a new compile builder with the required parameters.
    ///
    /// # Arguments
    /// * `source` - The HLSL source code
    /// * `entry_point` - The name of the entry point function (e.g., "main")
    /// * `target` - The shader target (e.g., `ShaderTarget::PS_5_0`)
    pub fn new(source: &'a str, entry_point: &str, target: ShaderTarget) -> Self {
        CompileBuilder {
            source: source.as_bytes(),
            source_name: None,
            entry_point: CString::new(entry_point).expect("Entry point contains null byte"),
            target,
            defines: Vec::new(),
            include: None,
            flags1: CompileFlags::empty(),
            flags2: 0,
        }
    }

    /// Creates a compile builder from raw bytes.
    pub fn from_bytes(source: &'a [u8], entry_point: &str, target: ShaderTarget) -> Self {
        CompileBuilder {
            source,
            source_name: None,
            entry_point: CString::new(entry_point).expect("Entry point contains null byte"),
            target,
            defines: Vec::new(),
            include: None,
            flags1: CompileFlags::empty(),
            flags2: 0,
        }
    }

    /// Sets the source file name (used in error messages).
    pub fn source_name(mut self, name: &str) -> Self {
        self.source_name = Some(CString::new(name).expect("Source name contains null byte"));
        self
    }

    /// Adds a preprocessor define.
    pub fn define(mut self, name: &str, value: &str) -> Self {
        self.defines.push(Define::new(name, value));
        self
    }

    /// Adds a preprocessor define flag (empty value).
    pub fn define_flag(mut self, name: &str) -> Self {
        self.defines.push(Define::flag(name));
        self
    }

    /// Adds multiple preprocessor defines from an iterator.
    pub fn defines<I>(mut self, defines: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        for (name, value) in defines {
            self.defines.push(Define::new(name, value));
        }
        self
    }

    /// Adds a pre-built Define.
    pub fn with_define(mut self, define: Define) -> Self {
        self.defines.push(define);
        self
    }

    /// Sets compilation flags (replaces any existing flags).
    pub fn flags(mut self, flags: CompileFlags) -> Self {
        self.flags1 = flags;
        self
    }

    /// Adds compilation flags (bitwise OR with existing).
    pub fn with_flags(mut self, flags: CompileFlags) -> Self {
        self.flags1 |= flags;
        self
    }

    /// Enables debug mode (D3DCOMPILE_DEBUG).
    pub fn debug(self) -> Self {
        self.with_flags(CompileFlags::DEBUG)
    }

    /// Skips optimization (D3DCOMPILE_SKIP_OPTIMIZATION).
    pub fn skip_optimization(self) -> Self {
        self.with_flags(CompileFlags::SKIP_OPTIMIZATION)
    }

    /// Treats warnings as errors.
    pub fn warnings_are_errors(self) -> Self {
        self.with_flags(CompileFlags::WARNINGS_ARE_ERRORS)
    }

    /// Sets the optimization level (0-3).
    ///
    /// * Level 0: No optimization
    /// * Level 1: Default optimization (default)
    /// * Level 2: More optimization
    /// * Level 3: Full optimization
    pub fn optimization_level(mut self, level: u32) -> Self {
        self.flags1 = self.flags1.with_optimization_level(level);
        self
    }

    /// Sets the matrix packing order to row-major.
    pub fn row_major_matrices(self) -> Self {
        self.with_flags(CompileFlags::PACK_MATRIX_ROW_MAJOR)
    }

    /// Sets the matrix packing order to column-major.
    pub fn column_major_matrices(self) -> Self {
        self.with_flags(CompileFlags::PACK_MATRIX_COLUMN_MAJOR)
    }

    /// Sets a custom include handler.
    ///
    /// # Safety
    /// The include handler must remain valid for the duration of compilation.
    /// Use `IncludeWrapper` from the include module for a safe interface.
    pub unsafe fn include_handler(mut self, include: *mut ID3DInclude) -> Self {
        self.include = Some(include);
        self
    }

    /// Sets secondary flags (Flags2 parameter).
    pub fn flags2(mut self, flags: u32) -> Self {
        self.flags2 = flags;
        self
    }

    /// Compiles the shader.
    ///
    /// Returns the compiled bytecode and any warning messages.
    pub fn compile(self) -> Result<CompileResult> {
        // Build defines array (null-terminated)
        let mut defines_raw: Vec<D3D_SHADER_MACRO> = self
            .defines
            .iter()
            .map(|d| D3D_SHADER_MACRO {
                Name: d.name.as_ptr(),
                Definition: d.value.as_ptr(),
            })
            .collect();
        defines_raw.push(D3D_SHADER_MACRO {
            Name: ptr::null(),
            Definition: ptr::null(),
        });

        let target_cstr = self.target.as_cstring();

        unsafe {
            let mut code: *mut ID3DBlob = ptr::null_mut();
            let mut errors: *mut ID3DBlob = ptr::null_mut();

            let result = D3DCompile(
                self.source.as_ptr() as *const _,
                self.source.len(),
                self.source_name
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                defines_raw.as_ptr(),
                self.include.unwrap_or(ptr::null_mut()),
                self.entry_point.as_ptr(),
                target_cstr.as_ptr(),
                self.flags1.bits(),
                self.flags2,
                &mut code,
                &mut errors,
            );

            let error_blob = Blob::from_raw(errors);

            if result != S_OK {
                let message = error_blob
                    .as_ref()
                    .map(|b| b.to_string_lossy())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("Unknown error (HRESULT: 0x{:08x})", result as u32));

                return Err(Error::Compilation {
                    hresult: HResult(result),
                    message,
                });
            }

            let bytecode = Blob::from_raw(code).ok_or_else(|| Error::Compilation {
                hresult: HResult(result),
                message: "No bytecode returned from compiler".to_string(),
            })?;

            let warnings = error_blob
                .as_ref()
                .map(|b| b.to_string_lossy())
                .filter(|s| !s.is_empty());

            Ok(CompileResult { bytecode, warnings })
        }
    }
}

/// Convenience function for simple shader compilation.
///
/// # Example
/// ```no_run
/// use d3dcrs::{compile, ShaderTarget};
///
/// let source = "float4 main() : SV_TARGET { return float4(1,0,0,1); }";
/// let bytecode = compile(source, "main", ShaderTarget::PS_5_0).unwrap();
/// ```
pub fn compile(source: &str, entry_point: &str, target: ShaderTarget) -> Result<Blob> {
    CompileBuilder::new(source, entry_point, target)
        .compile()
        .map(|r| r.bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_PS: &str = "float4 main() : SV_TARGET { return float4(1,0,0,1); }";

    #[test]
    fn test_compile_simple_shader() {
        let result = compile(SIMPLE_PS, "main", ShaderTarget::PS_5_0);
        assert!(result.is_ok(), "Compilation should succeed");

        let bytecode = result.unwrap();
        assert!(!bytecode.is_empty(), "Bytecode should not be empty");
        assert_eq!(
            &bytecode[0..4],
            b"DXBC",
            "Bytecode should start with DXBC magic"
        );
    }

    #[test]
    fn test_compile_with_defines() {
        let source = r#"
            #ifdef USE_RED
            float4 main() : SV_TARGET { return float4(1,0,0,1); }
            #else
            float4 main() : SV_TARGET { return float4(0,1,0,1); }
            #endif
        "#;

        let result = CompileBuilder::new(source, "main", ShaderTarget::PS_5_0)
            .define("USE_RED", "1")
            .compile();

        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_error() {
        let bad_source = "float4 main() : SV_TARGET { return undefined_variable; }";
        let result = compile(bad_source, "main", ShaderTarget::PS_5_0);

        assert!(result.is_err());
        if let Err(Error::Compilation { message, .. }) = result {
            assert!(
                message.contains("undefined") || message.contains("undeclared"),
                "Error should mention undefined: {}",
                message
            );
        }
    }
}
