//! Error types for d3dcrs operations

use std::fmt;
use thiserror::Error;

/// HRESULT error codes from Windows/D3D APIs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HResult(pub i32);

impl HResult {
    /// Success
    pub const S_OK: HResult = HResult(0);
    /// Generic failure
    pub const E_FAIL: HResult = HResult(0x80004005u32 as i32);
    /// Invalid argument
    pub const E_INVALIDARG: HResult = HResult(0x80070057u32 as i32);

    /// Returns true if the result indicates success
    #[inline]
    pub fn is_success(&self) -> bool {
        self.0 >= 0
    }

    /// Returns true if the result indicates an error
    #[inline]
    pub fn is_error(&self) -> bool {
        self.0 < 0
    }

    /// Returns the raw HRESULT value
    #[inline]
    pub fn code(&self) -> i32 {
        self.0
    }
}

impl fmt::Debug for HResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HResult(0x{:08x})", self.0 as u32)
    }
}

impl fmt::Display for HResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0 as u32)
    }
}

impl From<i32> for HResult {
    fn from(hr: i32) -> Self {
        HResult(hr)
    }
}

/// Error type for d3dcrs operations
#[derive(Error, Debug)]
pub enum Error {
    /// Shader compilation failed
    #[error("Compilation failed: {message}")]
    Compilation {
        /// The HRESULT error code
        hresult: HResult,
        /// Error message from the compiler
        message: String,
    },

    /// Preprocessing failed
    #[error("Preprocessing failed: {message}")]
    Preprocessing {
        /// The HRESULT error code
        hresult: HResult,
        /// Error message from the preprocessor
        message: String,
    },

    /// Disassembly failed
    #[error("Disassembly failed (HRESULT: {hresult})")]
    Disassembly {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Reflection failed
    #[error("Reflection failed (HRESULT: {hresult})")]
    Reflection {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Strip shader failed
    #[error("Strip shader failed (HRESULT: {hresult})")]
    StripShader {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Get blob part failed
    #[error("Get blob part failed (HRESULT: {hresult})")]
    GetBlobPart {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Set blob part failed
    #[error("Set blob part failed (HRESULT: {hresult})")]
    SetBlobPart {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Create blob failed
    #[error("Create blob failed (HRESULT: {hresult})")]
    CreateBlob {
        /// The HRESULT error code
        hresult: HResult,
    },

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// UTF-8 encoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// Include file not found
    #[error("Include file not found: {0}")]
    IncludeNotFound(String),

    /// IO error during include resolution
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for d3dcrs operations
pub type Result<T> = std::result::Result<T, Error>;
