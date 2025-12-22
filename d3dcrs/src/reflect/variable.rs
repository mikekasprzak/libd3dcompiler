//! Shader variable reflection

use super::types::TypeInfo;
use crate::{Error, HResult, Result};
use d3dcompiler::{D3D11_SHADER_VARIABLE_DESC, ID3D11ShaderReflectionVariable, S_OK};
use std::ffi::CStr;
use std::marker::PhantomData;

/// Shader variable description
#[derive(Debug, Clone)]
pub struct VariableDesc {
    /// Variable name
    pub name: String,
    /// Start offset in constant buffer
    pub start_offset: u32,
    /// Size in bytes
    pub size: u32,
    /// Flags
    pub flags: u32,
    /// Default value (if any)
    pub has_default_value: bool,
    /// Start texture slot (-1 if not used)
    pub start_texture: u32,
    /// Texture size
    pub texture_size: u32,
    /// Start sampler slot (-1 if not used)
    pub start_sampler: u32,
    /// Sampler size
    pub sampler_size: u32,
}

/// Wrapper for ID3D11ShaderReflectionVariable
pub struct Variable<'a> {
    ptr: *mut ID3D11ShaderReflectionVariable,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Variable<'a> {
    pub(crate) fn new(ptr: *mut ID3D11ShaderReflectionVariable) -> Self {
        Variable {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Gets the raw description (internal use).
    pub(crate) fn desc_raw(&self) -> Result<D3D11_SHADER_VARIABLE_DESC> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut desc: D3D11_SHADER_VARIABLE_DESC = std::mem::zeroed();
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

    /// Gets the variable description.
    pub fn desc(&self) -> Result<VariableDesc> {
        let raw = self.desc_raw()?;

        let name = if !raw.Name.is_null() {
            unsafe { CStr::from_ptr(raw.Name).to_string_lossy().into_owned() }
        } else {
            String::new()
        };

        Ok(VariableDesc {
            name,
            start_offset: raw.StartOffset,
            size: raw.Size,
            flags: raw.uFlags,
            has_default_value: !raw.DefaultValue.is_null(),
            start_texture: raw.StartTexture,
            texture_size: raw.TextureSize,
            start_sampler: raw.StartSampler,
            sampler_size: raw.SamplerSize,
        })
    }

    /// Gets the variable name.
    pub fn name(&self) -> String {
        self.desc().map(|d| d.name).unwrap_or_default()
    }

    /// Gets the start offset in the constant buffer.
    pub fn offset(&self) -> u32 {
        self.desc().map(|d| d.start_offset).unwrap_or(0)
    }

    /// Gets the size in bytes.
    pub fn size(&self) -> u32 {
        self.desc().map(|d| d.size).unwrap_or(0)
    }

    /// Gets the variable's type information.
    pub fn get_type(&self) -> Option<TypeInfo<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let type_ptr = (vtable.GetType)(self.ptr);
            if type_ptr.is_null() {
                None
            } else {
                Some(TypeInfo::new(type_ptr))
            }
        }
    }

    /// Gets the interface slot for this variable.
    pub fn interface_slot(&self, index: u32) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetInterfaceSlot)(self.ptr, index)
        }
    }
}
