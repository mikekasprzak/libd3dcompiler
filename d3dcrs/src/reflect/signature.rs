//! Input/output signature parameter reflection

use d3dcompiler::{D3D11_SIGNATURE_PARAMETER_DESC, ID3D11ShaderReflection, S_OK};
use std::ffi::CStr;

/// System value semantic type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SystemValueType {
    /// Undefined (user-defined semantic)
    Undefined = 0,
    /// SV_Position
    Position = 1,
    /// SV_ClipDistance
    ClipDistance = 2,
    /// SV_CullDistance
    CullDistance = 3,
    /// SV_RenderTargetArrayIndex
    RenderTargetArrayIndex = 4,
    /// SV_ViewportArrayIndex
    ViewportArrayIndex = 5,
    /// SV_VertexID
    VertexId = 6,
    /// SV_PrimitiveID
    PrimitiveId = 7,
    /// SV_InstanceID
    InstanceId = 8,
    /// SV_IsFrontFace
    IsFrontFace = 9,
    /// SV_SampleIndex
    SampleIndex = 10,
    /// Final quad edge tessellation factor
    FinalQuadEdgeTessFactor = 11,
    /// Final quad inside tessellation factor
    FinalQuadInsideTessFactor = 12,
    /// Final triangle edge tessellation factor
    FinalTriEdgeTessFactor = 13,
    /// Final triangle inside tessellation factor
    FinalTriInsideTessFactor = 14,
    /// Final line detail tessellation factor
    FinalLineDetailTessFactor = 15,
    /// Final line density tessellation factor
    FinalLineDensityTessFactor = 16,
    /// SV_Target
    Target = 64,
    /// SV_Depth
    Depth = 65,
    /// SV_Coverage
    Coverage = 66,
    /// SV_DepthGreaterEqual
    DepthGreaterEqual = 67,
    /// SV_DepthLessEqual
    DepthLessEqual = 68,
}

impl From<u32> for SystemValueType {
    fn from(value: u32) -> Self {
        match value {
            0 => SystemValueType::Undefined,
            1 => SystemValueType::Position,
            2 => SystemValueType::ClipDistance,
            3 => SystemValueType::CullDistance,
            4 => SystemValueType::RenderTargetArrayIndex,
            5 => SystemValueType::ViewportArrayIndex,
            6 => SystemValueType::VertexId,
            7 => SystemValueType::PrimitiveId,
            8 => SystemValueType::InstanceId,
            9 => SystemValueType::IsFrontFace,
            10 => SystemValueType::SampleIndex,
            11 => SystemValueType::FinalQuadEdgeTessFactor,
            12 => SystemValueType::FinalQuadInsideTessFactor,
            13 => SystemValueType::FinalTriEdgeTessFactor,
            14 => SystemValueType::FinalTriInsideTessFactor,
            15 => SystemValueType::FinalLineDetailTessFactor,
            16 => SystemValueType::FinalLineDensityTessFactor,
            64 => SystemValueType::Target,
            65 => SystemValueType::Depth,
            66 => SystemValueType::Coverage,
            67 => SystemValueType::DepthGreaterEqual,
            68 => SystemValueType::DepthLessEqual,
            _ => SystemValueType::Undefined,
        }
    }
}

/// Component type for shader parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ComponentType {
    /// Unknown type
    Unknown = 0,
    /// Unsigned 32-bit integer
    Uint32 = 1,
    /// Signed 32-bit integer
    Sint32 = 2,
    /// 32-bit float
    Float32 = 3,
}

impl From<u32> for ComponentType {
    fn from(value: u32) -> Self {
        match value {
            1 => ComponentType::Uint32,
            2 => ComponentType::Sint32,
            3 => ComponentType::Float32,
            _ => ComponentType::Unknown,
        }
    }
}

/// Shader input/output signature parameter
#[derive(Debug, Clone)]
pub struct SignatureParameter {
    /// Semantic name (e.g., "POSITION", "TEXCOORD")
    pub semantic_name: String,
    /// Semantic index (e.g., 0 for TEXCOORD0)
    pub semantic_index: u32,
    /// Register number
    pub register: u32,
    /// System value type
    pub system_value_type: SystemValueType,
    /// Component type
    pub component_type: ComponentType,
    /// Mask of used components (x=1, y=2, z=4, w=8)
    pub mask: u8,
    /// Read/write mask
    pub read_write_mask: u8,
    /// Stream index (for geometry shaders)
    pub stream: u32,
}

impl SignatureParameter {
    fn from_raw(raw: &D3D11_SIGNATURE_PARAMETER_DESC) -> Self {
        let semantic_name = if !raw.SemanticName.is_null() {
            unsafe {
                CStr::from_ptr(raw.SemanticName)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        SignatureParameter {
            semantic_name,
            semantic_index: raw.SemanticIndex,
            register: raw.Register,
            system_value_type: SystemValueType::from(raw.SystemValueType),
            component_type: ComponentType::from(raw.ComponentType),
            mask: raw.Mask,
            read_write_mask: raw.ReadWriteMask,
            stream: raw.Stream,
        }
    }

    /// Returns the number of components used (1-4).
    pub fn component_count(&self) -> u32 {
        self.mask.count_ones()
    }

    /// Returns true if this is a system value (SV_*).
    pub fn is_system_value(&self) -> bool {
        self.system_value_type != SystemValueType::Undefined
    }
}

pub(crate) fn get_input_parameter(
    refl: *mut ID3D11ShaderReflection,
    index: u32,
) -> Option<SignatureParameter> {
    unsafe {
        let vtable = &*(*refl).vtable;
        let mut desc: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
        let result = (vtable.GetInputParameterDesc)(refl, index, &mut desc);
        if result == S_OK {
            Some(SignatureParameter::from_raw(&desc))
        } else {
            None
        }
    }
}

pub(crate) fn get_output_parameter(
    refl: *mut ID3D11ShaderReflection,
    index: u32,
) -> Option<SignatureParameter> {
    unsafe {
        let vtable = &*(*refl).vtable;
        let mut desc: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
        let result = (vtable.GetOutputParameterDesc)(refl, index, &mut desc);
        if result == S_OK {
            Some(SignatureParameter::from_raw(&desc))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub(crate) fn get_patch_constant_parameter(
    refl: *mut ID3D11ShaderReflection,
    index: u32,
) -> Option<SignatureParameter> {
    unsafe {
        let vtable = &*(*refl).vtable;
        let mut desc: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
        let result = (vtable.GetPatchConstantParameterDesc)(refl, index, &mut desc);
        if result == S_OK {
            Some(SignatureParameter::from_raw(&desc))
        } else {
            None
        }
    }
}
