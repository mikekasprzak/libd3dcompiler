//! Resource binding reflection

use d3dcompiler::{D3D11_SHADER_INPUT_BIND_DESC, ID3D11ShaderReflection, S_OK};
use std::ffi::{CStr, CString};

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ResourceType {
    /// Constant buffer (cbuffer)
    CBuffer = 0,
    /// Texture buffer (tbuffer)
    TBuffer = 1,
    /// Texture
    Texture = 2,
    /// Sampler
    Sampler = 3,
    /// UAV read/write typed
    UavRwTyped = 4,
    /// Structured buffer
    Structured = 5,
    /// UAV read/write structured
    UavRwStructured = 6,
    /// Byte address buffer
    ByteAddress = 7,
    /// UAV read/write byte address
    UavRwByteAddress = 8,
    /// UAV append structured
    UavAppendStructured = 9,
    /// UAV consume structured
    UavConsumeStructured = 10,
    /// UAV read/write structured with counter
    UavRwStructuredWithCounter = 11,
}

impl From<u32> for ResourceType {
    fn from(value: u32) -> Self {
        match value {
            0 => ResourceType::CBuffer,
            1 => ResourceType::TBuffer,
            2 => ResourceType::Texture,
            3 => ResourceType::Sampler,
            4 => ResourceType::UavRwTyped,
            5 => ResourceType::Structured,
            6 => ResourceType::UavRwStructured,
            7 => ResourceType::ByteAddress,
            8 => ResourceType::UavRwByteAddress,
            9 => ResourceType::UavAppendStructured,
            10 => ResourceType::UavConsumeStructured,
            11 => ResourceType::UavRwStructuredWithCounter,
            _ => ResourceType::CBuffer,
        }
    }
}

/// Resource return type (for textures)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ResourceReturnType {
    /// Unorm
    Unorm = 1,
    /// Snorm
    Snorm = 2,
    /// Signed integer
    Sint = 3,
    /// Unsigned integer
    Uint = 4,
    /// Float
    Float = 5,
    /// Mixed
    Mixed = 6,
    /// Double
    Double = 7,
    /// Continued
    Continued = 8,
}

impl From<u32> for ResourceReturnType {
    fn from(value: u32) -> Self {
        match value {
            1 => ResourceReturnType::Unorm,
            2 => ResourceReturnType::Snorm,
            3 => ResourceReturnType::Sint,
            4 => ResourceReturnType::Uint,
            5 => ResourceReturnType::Float,
            6 => ResourceReturnType::Mixed,
            7 => ResourceReturnType::Double,
            8 => ResourceReturnType::Continued,
            _ => ResourceReturnType::Float,
        }
    }
}

/// Resource dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ResourceDimension {
    /// Unknown
    Unknown = 0,
    /// Buffer
    Buffer = 1,
    /// Texture1D
    Texture1D = 2,
    /// Texture1D array
    Texture1DArray = 3,
    /// Texture2D
    Texture2D = 4,
    /// Texture2D array
    Texture2DArray = 5,
    /// Texture2D multisample
    Texture2DMs = 6,
    /// Texture2D multisample array
    Texture2DMsArray = 7,
    /// Texture3D
    Texture3D = 8,
    /// TextureCube
    TextureCube = 9,
    /// TextureCube array
    TextureCubeArray = 10,
    /// Bufex (extended buffer)
    BufferEx = 11,
}

impl From<u32> for ResourceDimension {
    fn from(value: u32) -> Self {
        match value {
            0 => ResourceDimension::Unknown,
            1 => ResourceDimension::Buffer,
            2 => ResourceDimension::Texture1D,
            3 => ResourceDimension::Texture1DArray,
            4 => ResourceDimension::Texture2D,
            5 => ResourceDimension::Texture2DArray,
            6 => ResourceDimension::Texture2DMs,
            7 => ResourceDimension::Texture2DMsArray,
            8 => ResourceDimension::Texture3D,
            9 => ResourceDimension::TextureCube,
            10 => ResourceDimension::TextureCubeArray,
            11 => ResourceDimension::BufferEx,
            _ => ResourceDimension::Unknown,
        }
    }
}

/// Resource binding information
#[derive(Debug, Clone)]
pub struct ResourceBinding {
    /// Resource name
    pub name: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Bind point (register number)
    pub bind_point: u32,
    /// Bind count (number of consecutive bindings)
    pub bind_count: u32,
    /// Flags
    pub flags: u32,
    /// Return type (for textures)
    pub return_type: ResourceReturnType,
    /// Dimension
    pub dimension: ResourceDimension,
    /// Number of samples (for multisampled textures)
    pub num_samples: u32,
}

impl ResourceBinding {
    fn from_raw(raw: &D3D11_SHADER_INPUT_BIND_DESC) -> Self {
        let name = if !raw.Name.is_null() {
            unsafe { CStr::from_ptr(raw.Name).to_string_lossy().into_owned() }
        } else {
            String::new()
        };

        ResourceBinding {
            name,
            resource_type: ResourceType::from(raw.Type),
            bind_point: raw.BindPoint,
            bind_count: raw.BindCount,
            flags: raw.uFlags,
            return_type: ResourceReturnType::from(raw.ReturnType),
            dimension: ResourceDimension::from(raw.Dimension),
            num_samples: raw.NumSamples,
        }
    }

    /// Returns true if this is a constant buffer.
    pub fn is_constant_buffer(&self) -> bool {
        self.resource_type == ResourceType::CBuffer
    }

    /// Returns true if this is a texture.
    pub fn is_texture(&self) -> bool {
        self.resource_type == ResourceType::Texture
    }

    /// Returns true if this is a sampler.
    pub fn is_sampler(&self) -> bool {
        self.resource_type == ResourceType::Sampler
    }

    /// Returns true if this is a UAV.
    pub fn is_uav(&self) -> bool {
        matches!(
            self.resource_type,
            ResourceType::UavRwTyped
                | ResourceType::UavRwStructured
                | ResourceType::UavRwByteAddress
                | ResourceType::UavAppendStructured
                | ResourceType::UavConsumeStructured
                | ResourceType::UavRwStructuredWithCounter
        )
    }
}

pub(crate) fn get_resource_binding(
    refl: *mut ID3D11ShaderReflection,
    index: u32,
) -> Option<ResourceBinding> {
    unsafe {
        let vtable = &*(*refl).vtable;
        let mut desc: D3D11_SHADER_INPUT_BIND_DESC = std::mem::zeroed();
        let result = (vtable.GetResourceBindingDesc)(refl, index, &mut desc);
        if result == S_OK {
            Some(ResourceBinding::from_raw(&desc))
        } else {
            None
        }
    }
}

pub(crate) fn get_resource_binding_by_name(
    refl: *mut ID3D11ShaderReflection,
    name: &str,
) -> Option<ResourceBinding> {
    let name_cstr = CString::new(name).ok()?;
    unsafe {
        let vtable = &*(*refl).vtable;
        let mut desc: D3D11_SHADER_INPUT_BIND_DESC = std::mem::zeroed();
        let result = (vtable.GetResourceBindingDescByName)(refl, name_cstr.as_ptr(), &mut desc);
        if result == S_OK {
            Some(ResourceBinding::from_raw(&desc))
        } else {
            None
        }
    }
}
