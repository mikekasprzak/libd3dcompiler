//! Shader type reflection

use crate::{Error, HResult, Result};
use d3dcompiler::{D3D11_SHADER_TYPE_DESC, ID3D11ShaderReflectionType, S_OK};
use std::ffi::CStr;
use std::marker::PhantomData;

/// Shader variable type class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShaderTypeClass {
    /// Scalar (float, int, etc.)
    Scalar = 0,
    /// Vector (float2, float3, float4)
    Vector = 1,
    /// Matrix (float4x4, etc.)
    MatrixRows = 2,
    /// Column-major matrix
    MatrixColumns = 3,
    /// Object (texture, sampler, etc.)
    Object = 4,
    /// Struct
    Struct = 5,
    /// Interface class
    InterfaceClass = 6,
    /// Interface pointer
    InterfacePointer = 7,
}

impl From<u32> for ShaderTypeClass {
    fn from(value: u32) -> Self {
        match value {
            0 => ShaderTypeClass::Scalar,
            1 => ShaderTypeClass::Vector,
            2 => ShaderTypeClass::MatrixRows,
            3 => ShaderTypeClass::MatrixColumns,
            4 => ShaderTypeClass::Object,
            5 => ShaderTypeClass::Struct,
            6 => ShaderTypeClass::InterfaceClass,
            7 => ShaderTypeClass::InterfacePointer,
            _ => ShaderTypeClass::Scalar,
        }
    }
}

/// Shader variable type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShaderVariableType {
    /// Void type
    Void = 0,
    /// Boolean
    Bool = 1,
    /// 32-bit integer
    Int = 2,
    /// 32-bit float
    Float = 3,
    /// String
    String = 4,
    /// Texture
    Texture = 5,
    /// Texture1D
    Texture1D = 6,
    /// Texture2D
    Texture2D = 7,
    /// Texture3D
    Texture3D = 8,
    /// TextureCube
    TextureCube = 9,
    /// Sampler
    Sampler = 10,
    /// Sampler1D
    Sampler1D = 11,
    /// Sampler2D
    Sampler2D = 12,
    /// Sampler3D
    Sampler3D = 13,
    /// SamplerCube
    SamplerCube = 14,
    /// Pixel shader
    PixelShader = 15,
    /// Vertex shader
    VertexShader = 16,
    /// Pixel fragment
    PixelFragment = 17,
    /// Vertex fragment
    VertexFragment = 18,
    /// Unsigned integer
    Uint = 19,
    /// 8-bit unsigned integer
    Uint8 = 20,
    /// Geometry shader
    GeometryShader = 21,
    /// Rasterizer
    Rasterizer = 22,
    /// Depth stencil
    DepthStencil = 23,
    /// Blend
    Blend = 24,
    /// Buffer
    Buffer = 25,
    /// Constant buffer
    CBuffer = 26,
    /// Texture buffer
    TBuffer = 27,
    /// Texture1D array
    Texture1DArray = 28,
    /// Texture2D array
    Texture2DArray = 29,
    /// Render target view
    RenderTargetView = 30,
    /// Depth stencil view
    DepthStencilView = 31,
    /// Texture2D multisample
    Texture2DMs = 32,
    /// Texture2D multisample array
    Texture2DMsArray = 33,
    /// TextureCube array
    TextureCubeArray = 34,
    /// Hull shader
    HullShader = 35,
    /// Domain shader
    DomainShader = 36,
    /// Interface pointer
    InterfacePointer = 37,
    /// Compute shader
    ComputeShader = 38,
    /// Double
    Double = 39,
    /// RW Texture1D
    RwTexture1D = 40,
    /// RW Texture1D array
    RwTexture1DArray = 41,
    /// RW Texture2D
    RwTexture2D = 42,
    /// RW Texture2D array
    RwTexture2DArray = 43,
    /// RW Texture3D
    RwTexture3D = 44,
    /// RW Buffer
    RwBuffer = 45,
    /// Byte address buffer
    ByteAddressBuffer = 46,
    /// RW Byte address buffer
    RwByteAddressBuffer = 47,
    /// Structured buffer
    StructuredBuffer = 48,
    /// RW Structured buffer
    RwStructuredBuffer = 49,
    /// Append structured buffer
    AppendStructuredBuffer = 50,
    /// Consume structured buffer
    ConsumeStructuredBuffer = 51,
    /// Minimum precision 8-bit float
    Min8Float = 52,
    /// Minimum precision 10-bit float
    Min10Float = 53,
    /// Minimum precision 16-bit float
    Min16Float = 54,
    /// Minimum precision 12-bit integer
    Min12Int = 55,
    /// Minimum precision 16-bit integer
    Min16Int = 56,
    /// Minimum precision 16-bit unsigned integer
    Min16Uint = 57,
}

impl From<u32> for ShaderVariableType {
    fn from(value: u32) -> Self {
        // Direct cast - if out of range, return Void
        if value <= 57 {
            unsafe { std::mem::transmute::<u32, ShaderVariableType>(value) }
        } else {
            ShaderVariableType::Void
        }
    }
}

/// Type description
#[derive(Debug, Clone)]
pub struct TypeDesc {
    /// Type class (scalar, vector, matrix, etc.)
    pub class: ShaderTypeClass,
    /// Variable type (float, int, etc.)
    pub var_type: ShaderVariableType,
    /// Number of rows (for matrices)
    pub rows: u32,
    /// Number of columns (for vectors/matrices)
    pub columns: u32,
    /// Number of elements (for arrays)
    pub elements: u32,
    /// Number of members (for structs)
    pub members: u32,
    /// Offset in parent structure
    pub offset: u32,
    /// Type name
    pub name: String,
}

/// Wrapper for ID3D11ShaderReflectionType
pub struct TypeInfo<'a> {
    ptr: *mut ID3D11ShaderReflectionType,
    _marker: PhantomData<&'a ()>,
}

impl std::fmt::Debug for TypeInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeInfo").field("ptr", &self.ptr).finish()
    }
}

impl<'a> TypeInfo<'a> {
    pub(crate) fn new(ptr: *mut ID3D11ShaderReflectionType) -> Self {
        TypeInfo {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Gets the type description.
    pub fn desc(&self) -> Result<TypeDesc> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut desc: D3D11_SHADER_TYPE_DESC = std::mem::zeroed();
            let result = (vtable.GetDesc)(self.ptr, &mut desc);

            if result != S_OK {
                return Err(Error::Reflection {
                    hresult: HResult(result),
                });
            }

            let name = if !desc.Name.is_null() {
                CStr::from_ptr(desc.Name).to_string_lossy().into_owned()
            } else {
                String::new()
            };

            Ok(TypeDesc {
                class: ShaderTypeClass::from(desc.Class),
                var_type: ShaderVariableType::from(desc.Type),
                rows: desc.Rows,
                columns: desc.Columns,
                elements: desc.Elements,
                members: desc.Members,
                offset: desc.Offset,
                name,
            })
        }
    }

    /// Gets the type class.
    pub fn class(&self) -> ShaderTypeClass {
        self.desc()
            .map(|d| d.class)
            .unwrap_or(ShaderTypeClass::Scalar)
    }

    /// Gets the variable type.
    pub fn var_type(&self) -> ShaderVariableType {
        self.desc()
            .map(|d| d.var_type)
            .unwrap_or(ShaderVariableType::Void)
    }

    /// Gets the number of rows.
    pub fn rows(&self) -> u32 {
        self.desc().map(|d| d.rows).unwrap_or(0)
    }

    /// Gets the number of columns.
    pub fn columns(&self) -> u32 {
        self.desc().map(|d| d.columns).unwrap_or(0)
    }

    /// Gets the number of elements (for arrays).
    pub fn elements(&self) -> u32 {
        self.desc().map(|d| d.elements).unwrap_or(0)
    }

    /// Gets the number of members (for structs).
    pub fn member_count(&self) -> u32 {
        self.desc().map(|d| d.members).unwrap_or(0)
    }

    /// Gets a member type by index.
    pub fn member_type(&self, index: u32) -> Option<TypeInfo<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let member = (vtable.GetMemberTypeByIndex)(self.ptr, index);
            if member.is_null() {
                None
            } else {
                Some(TypeInfo::new(member))
            }
        }
    }

    /// Gets a member type by name.
    pub fn member_type_by_name(&self, name: &str) -> Option<TypeInfo<'a>> {
        let name_cstr = std::ffi::CString::new(name).ok()?;
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let member = (vtable.GetMemberTypeByName)(self.ptr, name_cstr.as_ptr());
            if member.is_null() {
                None
            } else {
                Some(TypeInfo::new(member))
            }
        }
    }

    /// Gets a member's name by index.
    pub fn member_name(&self, index: u32) -> Option<String> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let name = (vtable.GetMemberTypeName)(self.ptr, index);
            if name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(name).to_string_lossy().into_owned())
            }
        }
    }

    /// Returns an iterator over struct members.
    pub fn members(&self) -> MemberIter<'a> {
        let count = self.member_count();
        MemberIter {
            type_ptr: self.ptr,
            index: 0,
            count,
            _marker: PhantomData,
        }
    }

    /// Gets the sub-type (for arrays).
    pub fn sub_type(&self) -> Option<TypeInfo<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let sub = (vtable.GetSubType)(self.ptr);
            if sub.is_null() {
                None
            } else {
                Some(TypeInfo::new(sub))
            }
        }
    }

    /// Gets the base class (for classes).
    pub fn base_class(&self) -> Option<TypeInfo<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let base = (vtable.GetBaseClass)(self.ptr);
            if base.is_null() {
                None
            } else {
                Some(TypeInfo::new(base))
            }
        }
    }

    /// Gets the number of interfaces.
    pub fn num_interfaces(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetNumInterfaces)(self.ptr)
        }
    }

    /// Gets an interface by index.
    pub fn interface(&self, index: u32) -> Option<TypeInfo<'a>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let iface = (vtable.GetInterfaceByIndex)(self.ptr, index);
            if iface.is_null() {
                None
            } else {
                Some(TypeInfo::new(iface))
            }
        }
    }
}

/// Struct member with name and type
#[derive(Debug)]
pub struct Member<'a> {
    /// Member name
    pub name: String,
    /// Member type
    pub type_info: TypeInfo<'a>,
}

/// Iterator over struct members
pub struct MemberIter<'a> {
    type_ptr: *mut ID3D11ShaderReflectionType,
    index: u32,
    count: u32,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for MemberIter<'a> {
    type Item = Member<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        unsafe {
            let vtable = &*(*self.type_ptr).vtable;

            let name_ptr = (vtable.GetMemberTypeName)(self.type_ptr, self.index);
            let name = if name_ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
            };

            let member_type = (vtable.GetMemberTypeByIndex)(self.type_ptr, self.index);
            self.index += 1;

            if member_type.is_null() {
                None
            } else {
                Some(Member {
                    name,
                    type_info: TypeInfo::new(member_type),
                })
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for MemberIter<'_> {}
