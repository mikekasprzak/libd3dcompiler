//! HLSL preprocessing API

use crate::compile::Define;
use crate::{Blob, Error, HResult, Result};
use d3dcompiler::{D3D_SHADER_MACRO, D3DPreprocess, ID3DBlob, ID3DInclude, S_OK};
use std::ffi::CString;
use std::ptr;

/// Result of successful preprocessing
#[derive(Debug)]
pub struct PreprocessResult {
    /// The preprocessed source code
    pub source: Blob,
    /// Any warning messages
    pub warnings: Option<String>,
}

/// Builder for HLSL preprocessing
///
/// # Example
/// ```no_run
/// use d3dcrs::PreprocessBuilder;
///
/// let source = r#"
///     #define PI 3.14159
///     float4 main() : SV_TARGET { return float4(PI, 0, 0, 1); }
/// "#;
///
/// let result = PreprocessBuilder::new(source)
///     .source_name("my_shader.hlsl")
///     .define("EXTRA", "1")
///     .preprocess()
///     .unwrap();
///
/// println!("Preprocessed: {}", result.source.to_string_lossy());
/// ```
pub struct PreprocessBuilder<'a> {
    source: &'a [u8],
    source_name: Option<CString>,
    defines: Vec<Define>,
    include: Option<*mut ID3DInclude>,
}

impl<'a> PreprocessBuilder<'a> {
    /// Creates a new preprocess builder.
    pub fn new(source: &'a str) -> Self {
        PreprocessBuilder {
            source: source.as_bytes(),
            source_name: None,
            defines: Vec::new(),
            include: None,
        }
    }

    /// Creates a preprocess builder from raw bytes.
    pub fn from_bytes(source: &'a [u8]) -> Self {
        PreprocessBuilder {
            source,
            source_name: None,
            defines: Vec::new(),
            include: None,
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
        self.defines.push(Define::new(name, ""));
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

    /// Sets a custom include handler.
    ///
    /// # Safety
    /// The include handler must remain valid for the duration of preprocessing.
    pub unsafe fn include_handler(mut self, include: *mut ID3DInclude) -> Self {
        self.include = Some(include);
        self
    }

    /// Preprocesses the source.
    pub fn preprocess(self) -> Result<PreprocessResult> {
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

        unsafe {
            let mut code: *mut ID3DBlob = ptr::null_mut();
            let mut errors: *mut ID3DBlob = ptr::null_mut();

            let result = D3DPreprocess(
                self.source.as_ptr() as *const _,
                self.source.len(),
                self.source_name
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                defines_raw.as_ptr(),
                self.include.unwrap_or(ptr::null_mut()),
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

                return Err(Error::Preprocessing {
                    hresult: HResult(result),
                    message,
                });
            }

            let source = Blob::from_raw(code).ok_or_else(|| Error::Preprocessing {
                hresult: HResult(result),
                message: "No output from preprocessor".to_string(),
            })?;

            let warnings = error_blob
                .as_ref()
                .map(|b| b.to_string_lossy())
                .filter(|s| !s.is_empty());

            Ok(PreprocessResult { source, warnings })
        }
    }
}

/// Convenience function for simple preprocessing.
///
/// # Example
/// ```no_run
/// use d3dcrs::preprocess;
///
/// let source = "#define X 1\nfloat4 main() : SV_TARGET { return X; }";
/// let preprocessed = preprocess(source).unwrap();
/// ```
pub fn preprocess(source: &str) -> Result<Blob> {
    PreprocessBuilder::new(source)
        .preprocess()
        .map(|r| r.source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_simple() {
        let source = r#"
            #define PI 3.14159
            float4 main() : SV_TARGET { return float4(PI, 0, 0, 1); }
        "#;

        let result = preprocess(source);
        assert!(result.is_ok());

        let preprocessed = result.unwrap();
        let text = preprocessed.to_string_lossy();
        assert!(text.contains("3.14159"), "Should contain expanded PI value");
        assert!(!text.contains("#define PI"), "Defines should be expanded");
    }
}
