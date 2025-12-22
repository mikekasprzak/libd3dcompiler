//! Constant buffer reflection

use super::variable::Variable;
use crate::{Error, HResult, Result};
use d3dcompiler::{D3D11_SHADER_BUFFER_DESC, ID3D11ShaderReflectionConstantBuffer, S_OK};
use std::ffi::CStr;
use std::marker::PhantomData;

/// Constant buffer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ConstantBufferType {
    /// cbuffer
    ConstantBuffer = 0,
    /// tbuffer
    TextureBuffer = 1,
    /// Interface pointers
    InterfacePointers = 2,
    /// Resource bind info
    ResourceBindInfo = 3,
}

impl From<u32> for ConstantBufferType {
    fn from(value: u32) -> Self {
        match value {
            0 => ConstantBufferType::ConstantBuffer,
            1 => ConstantBufferType::TextureBuffer,
            2 => ConstantBufferType::InterfacePointers,
            3 => ConstantBufferType::ResourceBindInfo,
            _ => ConstantBufferType::ConstantBuffer,
        }
    }
}

/// Constant buffer description
#[derive(Debug, Clone)]
pub struct ConstantBufferDesc {
    /// Buffer name
    pub name: String,
    /// Buffer type
    pub buffer_type: ConstantBufferType,
    /// Number of variables
    pub variables: u32,
    /// Size in bytes
    pub size: u32,
    /// Flags
    pub flags: u32,
}

/// Wrapper for ID3D11ShaderReflectionConstantBuffer
pub struct ConstantBuffer<'a> {
    ptr: *mut ID3D11ShaderReflectionConstantBuffer,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ConstantBuffer<'a> {
    pub(crate) fn new(ptr: *mut ID3D11ShaderReflectionConstantBuffer) -> Self {
        ConstantBuffer {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Gets the raw description (internal use).
    pub(crate) fn desc_raw(&self) -> Result<D3D11_SHADER_BUFFER_DESC> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
            let result = (vtable.GetDesc)(self.ptr, &mut desc);
            if result == S_OK {
                Ok(desc)
            } else {
                Err(Error::Reflection {
                    hresult: HResult(result),
                })
            }
        }
    }

    /// Gets the constant buffer description.
    pub fn desc(&self) -> Result<ConstantBufferDesc> {
        let raw = self.desc_raw()?;

        let name = if !raw.Name.is_null() {
            unsafe { CStr::from_ptr(raw.Name).to_string_lossy().into_owned() }
        } else {
            String::new()
        };

        Ok(ConstantBufferDesc {
            name,
            buffer_type: ConstantBufferType::from(raw.Type),
            variables: raw.Variables,
            size: raw.Size,
            flags: raw.uFlags,
        })
    }

    /// Gets the buffer name.
    pub fn name(&self) -> String {
        self.desc().map(|d| d.name).unwrap_or_default()
    }

    /// Gets the buffer size in bytes.
    pub fn size(&self) -> u32 {
        self.desc().map(|d| d.size).unwrap_or(0)
    }

    /// Gets the number of variables.
    pub fn variable_count(&self) -> u32 {
        self.desc().map(|d| d.variables).unwrap_or(0)
    }

    /// Gets the buffer type.
    pub fn buffer_type(&self) -> ConstantBufferType {
        self.desc()
            .map(|d| d.buffer_type)
            .unwrap_or(ConstantBufferType::ConstantBuffer)
    }

    /// Gets a variable by index.
    pub fn variable(&self, index: u32) -> Option<Variable<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let var = (vtable.GetVariableByIndex)(self.ptr, index);
            if var.is_null() {
                None
            } else {
                Some(Variable::new(var))
            }
        }
    }

    /// Gets a variable by name.
    pub fn variable_by_name(&self, name: &str) -> Option<Variable<'a>> {
        let name_cstr = std::ffi::CString::new(name).ok()?;
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let var = (vtable.GetVariableByName)(self.ptr, name_cstr.as_ptr());
            if var.is_null() {
                None
            } else {
                // Check if it's a valid variable by trying to get its desc
                let wrapper = Variable::new(var);
                if wrapper.desc_raw().is_ok() {
                    Some(wrapper)
                } else {
                    None
                }
            }
        }
    }

    /// Returns an iterator over variables.
    pub fn variables(&self) -> VariableIter<'a> {
        let count = self.variable_count();
        VariableIter {
            cb: self.ptr,
            index: 0,
            count,
            _marker: PhantomData,
        }
    }
}

/// Iterator over constant buffer variables
pub struct VariableIter<'a> {
    cb: *mut ID3D11ShaderReflectionConstantBuffer,
    index: u32,
    count: u32,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for VariableIter<'a> {
    type Item = Variable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        unsafe {
            let vtable = &*(*self.cb).vtable;
            let var = (vtable.GetVariableByIndex)(self.cb, self.index);
            self.index += 1;
            if var.is_null() {
                None
            } else {
                Some(Variable::new(var))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for VariableIter<'_> {}
