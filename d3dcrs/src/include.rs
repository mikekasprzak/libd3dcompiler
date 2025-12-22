//! Include handler trait for custom #include resolution

use crate::{Error, Result};
use std::path::PathBuf;

/// Include type (local or system)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IncludeType {
    /// Local include (#include "file.h")
    Local = 0,
    /// System include (#include <file.h>)
    System = 1,
}

impl From<u32> for IncludeType {
    fn from(value: u32) -> Self {
        match value {
            0 => IncludeType::Local,
            _ => IncludeType::System,
        }
    }
}

/// Trait for custom include file resolution
///
/// Implement this trait to provide custom handling for #include directives
/// in your HLSL shaders.
///
/// # Example
/// ```no_run
/// use d3dcrs::{IncludeHandler, IncludeType, Result};
///
/// struct MyIncludeHandler {
///     base_path: std::path::PathBuf,
/// }
///
/// impl IncludeHandler for MyIncludeHandler {
///     fn open(&mut self, include_type: IncludeType, filename: &str) -> Result<Vec<u8>> {
///         let path = self.base_path.join(filename);
///         std::fs::read(&path).map_err(|e| d3dcrs::Error::IncludeNotFound(filename.to_string()))
///     }
/// }
/// ```
pub trait IncludeHandler {
    /// Opens an include file and returns its contents.
    ///
    /// # Arguments
    /// * `include_type` - Whether this is a local (#include "...") or system (#include <...>) include
    /// * `filename` - The filename from the #include directive
    ///
    /// # Returns
    /// The file contents as a byte vector, or an error if the file cannot be found/read.
    fn open(&mut self, include_type: IncludeType, filename: &str) -> Result<Vec<u8>>;
}

/// File system include handler that resolves includes from specified directories.
///
/// # Example
/// ```no_run
/// use d3dcrs::FileSystemInclude;
///
/// let include = FileSystemInclude::new()
///     .with_path("shaders/include")
///     .with_path("/usr/local/share/hlsl");
/// ```
#[derive(Debug, Clone, Default)]
pub struct FileSystemInclude {
    search_paths: Vec<PathBuf>,
}

impl FileSystemInclude {
    /// Creates a new file system include handler with no search paths.
    pub fn new() -> Self {
        FileSystemInclude {
            search_paths: Vec::new(),
        }
    }

    /// Creates a new handler with the current directory as the first search path.
    pub fn with_current_dir() -> Self {
        let mut handler = Self::new();
        if let Ok(cwd) = std::env::current_dir() {
            handler.search_paths.push(cwd);
        }
        handler
    }

    /// Adds a search path (builder pattern).
    pub fn with_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Adds a search path.
    pub fn add_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.search_paths.push(path.into());
    }

    /// Returns the search paths.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}

impl IncludeHandler for FileSystemInclude {
    fn open(&mut self, _include_type: IncludeType, filename: &str) -> Result<Vec<u8>> {
        // Try each search path
        for search_path in &self.search_paths {
            let path = search_path.join(filename);
            if path.exists() {
                return std::fs::read(&path).map_err(Into::into);
            }
        }

        // If no search paths, try the filename directly
        if self.search_paths.is_empty() {
            let path = PathBuf::from(filename);
            if path.exists() {
                return std::fs::read(&path).map_err(Into::into);
            }
        }

        Err(Error::IncludeNotFound(filename.to_string()))
    }
}

/// In-memory include handler for testing or embedded includes.
///
/// # Example
/// ```
/// use d3dcrs::MemoryInclude;
///
/// let mut handler = MemoryInclude::new();
/// handler.add("common.hlsl", b"float4 white = float4(1,1,1,1);");
/// ```
pub struct MemoryInclude {
    files: std::collections::HashMap<String, Vec<u8>>,
}

impl MemoryInclude {
    /// Creates a new empty memory include handler.
    pub fn new() -> Self {
        MemoryInclude {
            files: std::collections::HashMap::new(),
        }
    }

    /// Adds a file to the handler.
    pub fn add(&mut self, filename: &str, contents: &[u8]) {
        self.files.insert(filename.to_string(), contents.to_vec());
    }

    /// Adds a file (builder pattern).
    pub fn with_file(mut self, filename: &str, contents: &[u8]) -> Self {
        self.add(filename, contents);
        self
    }
}

impl Default for MemoryInclude {
    fn default() -> Self {
        Self::new()
    }
}

impl IncludeHandler for MemoryInclude {
    fn open(&mut self, _include_type: IncludeType, filename: &str) -> Result<Vec<u8>> {
        self.files
            .get(filename)
            .cloned()
            .ok_or_else(|| Error::IncludeNotFound(filename.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_include() {
        let mut handler = MemoryInclude::new().with_file("test.hlsl", b"float x = 1.0;");

        let result = handler.open(IncludeType::Local, "test.hlsl");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"float x = 1.0;");

        let missing = handler.open(IncludeType::Local, "missing.hlsl");
        assert!(missing.is_err());
    }

    #[test]
    fn test_include_type() {
        assert_eq!(IncludeType::from(0), IncludeType::Local);
        assert_eq!(IncludeType::from(1), IncludeType::System);
        assert_eq!(IncludeType::from(99), IncludeType::System);
    }
}
