//! RAII wrapper for ID3DBlob

use crate::{Error, HResult, Result};
use d3dcompiler::{D3DCreateBlob, ID3DBlob, S_OK};
use std::ops::Deref;
use std::slice;

/// RAII wrapper for ID3DBlob
///
/// Provides safe access to blob data and automatic cleanup via Drop.
/// When dropped, the blob's reference count is decremented.
pub struct Blob {
    ptr: *mut ID3DBlob,
}

impl Blob {
    /// Creates a new Blob from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid ID3DBlob pointer or null.
    /// Takes ownership of the reference count (does not AddRef).
    pub(crate) unsafe fn from_raw(ptr: *mut ID3DBlob) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Blob { ptr })
        }
    }

    /// Creates a new Blob from a raw pointer, returning an error if null.
    ///
    /// # Safety
    /// The pointer must be a valid ID3DBlob pointer or null.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw_or_err(ptr: *mut ID3DBlob, err_msg: &str) -> Result<Self> {
        unsafe { Self::from_raw(ptr) }.ok_or_else(|| Error::InvalidParameter(err_msg.to_string()))
    }

    /// Creates a new empty blob with the specified size.
    pub fn new(size: usize) -> Result<Self> {
        let mut blob: *mut ID3DBlob = std::ptr::null_mut();
        unsafe {
            let result = D3DCreateBlob(size, &mut blob);
            if result != S_OK {
                return Err(Error::CreateBlob {
                    hresult: HResult(result),
                });
            }
            Ok(Blob { ptr: blob })
        }
    }

    /// Returns the blob data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let ptr = (vtable.GetBufferPointer)(self.ptr);
            let size = (vtable.GetBufferSize)(self.ptr);
            slice::from_raw_parts(ptr as *const u8, size)
        }
    }

    /// Returns the blob data as a mutable byte slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let ptr = (vtable.GetBufferPointer)(self.ptr);
            let size = (vtable.GetBufferSize)(self.ptr);
            slice::from_raw_parts_mut(ptr as *mut u8, size)
        }
    }

    /// Returns the size of the blob in bytes.
    pub fn len(&self) -> usize {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetBufferSize)(self.ptr)
        }
    }

    /// Returns true if the blob is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Interprets the blob as a UTF-8 string.
    ///
    /// Useful for error messages and disassembly output.
    /// Trailing null bytes are trimmed.
    pub fn as_str(&self) -> Result<&str> {
        let bytes = self.as_bytes();
        // Trim trailing null bytes
        let trimmed = bytes
            .iter()
            .rposition(|&b| b != 0)
            .map(|i| &bytes[..=i])
            .unwrap_or(&[]);
        std::str::from_utf8(trimmed).map_err(Into::into)
    }

    /// Converts the blob to a String, trimming trailing nulls.
    ///
    /// Returns an error if the blob contains invalid UTF-8.
    pub fn to_string_lossy(&self) -> String {
        let bytes = self.as_bytes();
        let trimmed = bytes
            .iter()
            .rposition(|&b| b != 0)
            .map(|i| &bytes[..=i])
            .unwrap_or(&[]);
        String::from_utf8_lossy(trimmed).into_owned()
    }

    /// Returns the raw pointer (for internal use with FFI).
    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *mut ID3DBlob {
        self.ptr
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let vtable = &*(*self.ptr).vtable;
                (vtable.Release)(self.ptr);
            }
        }
    }
}

impl Deref for Blob {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl std::fmt::Debug for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blob")
            .field("len", &self.len())
            .field("ptr", &self.ptr)
            .finish()
    }
}

// Blob is Send + Sync since ID3DBlob is thread-safe
// The underlying COM object uses atomic reference counting
unsafe impl Send for Blob {}
unsafe impl Sync for Blob {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_create() {
        let blob = Blob::new(256).unwrap();
        assert_eq!(blob.len(), 256);
        assert!(!blob.is_empty());
    }

    #[test]
    fn test_blob_write_read() {
        let mut blob = Blob::new(16).unwrap();
        let data = blob.as_bytes_mut();
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = i as u8;
        }

        let read = blob.as_bytes();
        for (i, &byte) in read.iter().enumerate() {
            assert_eq!(byte, i as u8);
        }
    }
}
