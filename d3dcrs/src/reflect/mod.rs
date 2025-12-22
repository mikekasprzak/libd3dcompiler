//! Shader reflection API
//!
//! This module provides safe wrappers for the D3D11 shader reflection interfaces,
//! allowing you to inspect compiled shaders for metadata like constant buffers,
//! resource bindings, input/output signatures, and instruction counts.
//!
//! # Example
//! ```no_run
//! use d3dcrs::{compile, ShaderTarget};
//! use d3dcrs::reflect::ShaderReflection;
//!
//! let bytecode = compile(
//!     r#"
//!     cbuffer Constants : register(b0) {
//!         float4x4 worldViewProj;
//!     };
//!     float4 main(float4 pos : SV_POSITION) : SV_TARGET {
//!         return mul(pos, worldViewProj);
//!     }
//!     "#,
//!     "main",
//!     ShaderTarget::PS_5_0
//! ).unwrap();
//!
//! let reflection = ShaderReflection::new(&bytecode).unwrap();
//! let desc = reflection.desc().unwrap();
//!
//! println!("Constant buffers: {}", desc.constant_buffers);
//! println!("Instructions: {}", desc.instruction_count);
//!
//! for cb in reflection.constant_buffers() {
//!     println!("CB: {} ({} bytes)", cb.name(), cb.size());
//! }
//! ```

mod bindings;
mod constant_buffer;
mod signature;
mod types;
mod variable;

pub use bindings::{ResourceBinding, ResourceDimension, ResourceReturnType, ResourceType};
pub use constant_buffer::ConstantBuffer;
pub use signature::{ComponentType, SignatureParameter, SystemValueType};
pub use types::{ShaderTypeClass, ShaderVariableType, TypeInfo};
pub use variable::Variable;

use crate::{Error, HResult, Result};
use d3dcompiler::{D3D11_SHADER_DESC, D3DReflect, ID3D11ShaderReflection, S_OK};
use std::ffi::CStr;
use std::ptr;

/// IID for ID3D11ShaderReflection: {8d536ca1-0cca-4956-a837-786963755584}
const IID_ID3D11SHADERREFLECTION: [u8; 16] = [
    0xa1, 0x6c, 0x53, 0x8d, 0xca, 0x0c, 0x56, 0x49, 0xa8, 0x37, 0x78, 0x69, 0x63, 0x75, 0x55, 0x84,
];

/// High-level shader description
#[derive(Debug, Clone)]
pub struct ShaderDesc {
    /// Shader version (encoded as type and model)
    pub version: u32,
    /// Creator string (compiler version)
    pub creator: String,
    /// Compile flags used
    pub flags: u32,
    /// Number of constant buffers
    pub constant_buffers: u32,
    /// Number of bound resources
    pub bound_resources: u32,
    /// Number of input parameters
    pub input_parameters: u32,
    /// Number of output parameters
    pub output_parameters: u32,
    /// Total instruction count
    pub instruction_count: u32,
    /// Number of temporary registers used
    pub temp_register_count: u32,
    /// Number of temporary arrays used
    pub temp_array_count: u32,
    /// Number of constant definitions
    pub def_count: u32,
    /// Number of declarations
    pub dcl_count: u32,
    /// Number of texture normal instructions
    pub texture_normal_instructions: u32,
    /// Number of texture load instructions
    pub texture_load_instructions: u32,
    /// Number of texture comparison instructions
    pub texture_comp_instructions: u32,
    /// Number of texture bias instructions
    pub texture_bias_instructions: u32,
    /// Number of texture gradient instructions
    pub texture_gradient_instructions: u32,
    /// Number of floating-point instructions
    pub float_instruction_count: u32,
    /// Number of integer instructions
    pub int_instruction_count: u32,
    /// Number of unsigned integer instructions
    pub uint_instruction_count: u32,
    /// Number of static flow control instructions
    pub static_flow_control_count: u32,
    /// Number of dynamic flow control instructions
    pub dynamic_flow_control_count: u32,
    /// Number of macro instructions
    pub macro_instruction_count: u32,
    /// Number of array instructions
    pub array_instruction_count: u32,
    /// Geometry shader output topology
    pub gs_output_topology: u32,
    /// Maximum output vertex count for geometry shader
    pub gs_max_output_vertex_count: u32,
    /// Input primitive type
    pub input_primitive: u32,
    /// Number of patch constant parameters
    pub patch_constant_parameters: u32,
    /// Number of geometry shader instances
    pub gs_instance_count: u32,
    /// Number of control points
    pub control_points: u32,
    /// Hull shader output primitive
    pub hs_output_primitive: u32,
    /// Hull shader partitioning mode
    pub hs_partitioning: u32,
    /// Tessellator domain
    pub tessellator_domain: u32,
    /// Number of barrier instructions
    pub barrier_instructions: u32,
    /// Number of interlocked instructions
    pub interlocked_instructions: u32,
    /// Number of texture store instructions
    pub texture_store_instructions: u32,
}

/// RAII wrapper for ID3D11ShaderReflection
///
/// Provides safe access to shader reflection data.
pub struct ShaderReflection {
    ptr: *mut ID3D11ShaderReflection,
}

impl ShaderReflection {
    /// Creates a shader reflection from compiled bytecode.
    pub fn new(bytecode: &[u8]) -> Result<Self> {
        unsafe {
            let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
            let result = D3DReflect(
                bytecode.as_ptr() as *const _,
                bytecode.len(),
                IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
                &mut reflector,
            );

            if result != S_OK || reflector.is_null() {
                return Err(Error::Reflection {
                    hresult: HResult(result),
                });
            }

            Ok(ShaderReflection {
                ptr: reflector as *mut ID3D11ShaderReflection,
            })
        }
    }

    /// Gets the shader description.
    pub fn desc(&self) -> Result<ShaderDesc> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
            let result = (vtable.GetDesc)(self.ptr, &mut desc);

            if result != S_OK {
                return Err(Error::Reflection {
                    hresult: HResult(result),
                });
            }

            let creator = if !desc.Creator.is_null() {
                CStr::from_ptr(desc.Creator).to_string_lossy().into_owned()
            } else {
                String::new()
            };

            Ok(ShaderDesc {
                version: desc.Version,
                creator,
                flags: desc.Flags,
                constant_buffers: desc.ConstantBuffers,
                bound_resources: desc.BoundResources,
                input_parameters: desc.InputParameters,
                output_parameters: desc.OutputParameters,
                instruction_count: desc.InstructionCount,
                temp_register_count: desc.TempRegisterCount,
                temp_array_count: desc.TempArrayCount,
                def_count: desc.DefCount,
                dcl_count: desc.DclCount,
                texture_normal_instructions: desc.TextureNormalInstructions,
                texture_load_instructions: desc.TextureLoadInstructions,
                texture_comp_instructions: desc.TextureCompInstructions,
                texture_bias_instructions: desc.TextureBiasInstructions,
                texture_gradient_instructions: desc.TextureGradientInstructions,
                float_instruction_count: desc.FloatInstructionCount,
                int_instruction_count: desc.IntInstructionCount,
                uint_instruction_count: desc.UintInstructionCount,
                static_flow_control_count: desc.StaticFlowControlCount,
                dynamic_flow_control_count: desc.DynamicFlowControlCount,
                macro_instruction_count: desc.MacroInstructionCount,
                array_instruction_count: desc.ArrayInstructionCount,
                gs_output_topology: desc.GSOutputTopology,
                gs_max_output_vertex_count: desc.GSMaxOutputVertexCount,
                input_primitive: desc.InputPrimitive,
                patch_constant_parameters: desc.PatchConstantParameters,
                gs_instance_count: desc.cGSInstanceCount,
                control_points: desc.cControlPoints,
                hs_output_primitive: desc.HSOutputPrimitive,
                hs_partitioning: desc.HSPartitioning,
                tessellator_domain: desc.TessellatorDomain,
                barrier_instructions: desc.cBarrierInstructions,
                interlocked_instructions: desc.cInterlockedInstructions,
                texture_store_instructions: desc.cTextureStoreInstructions,
            })
        }
    }

    /// Returns an iterator over constant buffers.
    pub fn constant_buffers(&self) -> ConstantBufferIter<'_> {
        let count = self.desc().map(|d| d.constant_buffers).unwrap_or(0);
        ConstantBufferIter {
            reflection: self,
            index: 0,
            count,
        }
    }

    /// Gets a constant buffer by index.
    pub fn constant_buffer(&self, index: u32) -> Option<ConstantBuffer<'_>> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let cb = (vtable.GetConstantBufferByIndex)(self.ptr, index);
            if cb.is_null() {
                None
            } else {
                Some(ConstantBuffer::new(cb))
            }
        }
    }

    /// Gets a constant buffer by name.
    pub fn constant_buffer_by_name(&self, name: &str) -> Option<ConstantBuffer<'_>> {
        let name_cstr = std::ffi::CString::new(name).ok()?;
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let cb = (vtable.GetConstantBufferByName)(self.ptr, name_cstr.as_ptr());
            if cb.is_null() {
                None
            } else {
                // Check if it's a valid CB by trying to get its desc
                let wrapper = ConstantBuffer::new(cb);
                if wrapper.desc_raw().is_ok() {
                    Some(wrapper)
                } else {
                    None
                }
            }
        }
    }

    /// Returns an iterator over input parameters.
    pub fn input_parameters(&self) -> InputParameterIter<'_> {
        let count = self.desc().map(|d| d.input_parameters).unwrap_or(0);
        InputParameterIter {
            reflection: self,
            index: 0,
            count,
        }
    }

    /// Gets an input parameter by index.
    pub fn input_parameter(&self, index: u32) -> Option<SignatureParameter> {
        signature::get_input_parameter(self.ptr, index)
    }

    /// Returns an iterator over output parameters.
    pub fn output_parameters(&self) -> OutputParameterIter<'_> {
        let count = self.desc().map(|d| d.output_parameters).unwrap_or(0);
        OutputParameterIter {
            reflection: self,
            index: 0,
            count,
        }
    }

    /// Gets an output parameter by index.
    pub fn output_parameter(&self, index: u32) -> Option<SignatureParameter> {
        signature::get_output_parameter(self.ptr, index)
    }

    /// Returns an iterator over resource bindings.
    pub fn resource_bindings(&self) -> ResourceBindingIter<'_> {
        let count = self.desc().map(|d| d.bound_resources).unwrap_or(0);
        ResourceBindingIter {
            reflection: self,
            index: 0,
            count,
        }
    }

    /// Gets a resource binding by index.
    pub fn resource_binding(&self, index: u32) -> Option<ResourceBinding> {
        bindings::get_resource_binding(self.ptr, index)
    }

    /// Gets a resource binding by name.
    pub fn resource_binding_by_name(&self, name: &str) -> Option<ResourceBinding> {
        bindings::get_resource_binding_by_name(self.ptr, name)
    }

    /// Gets compute shader thread group size (for compute shaders only).
    ///
    /// Returns (x, y, z) thread counts.
    pub fn thread_group_size(&self) -> (u32, u32, u32) {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut x = 0u32;
            let mut y = 0u32;
            let mut z = 0u32;
            (vtable.GetThreadGroupSize)(self.ptr, &mut x, &mut y, &mut z);
            (x, y, z)
        }
    }

    /// Gets the number of MOV instructions.
    pub fn mov_instruction_count(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetMovInstructionCount)(self.ptr)
        }
    }

    /// Gets the number of MOVC instructions.
    pub fn movc_instruction_count(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetMovcInstructionCount)(self.ptr)
        }
    }

    /// Gets the number of conversion instructions.
    pub fn conversion_instruction_count(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetConversionInstructionCount)(self.ptr)
        }
    }

    /// Gets the number of bitwise instructions.
    pub fn bitwise_instruction_count(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetBitwiseInstructionCount)(self.ptr)
        }
    }

    /// Gets the geometry shader input primitive type.
    pub fn gs_input_primitive(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetGSInputPrimitive)(self.ptr)
        }
    }

    /// Returns whether this is a sample frequency shader.
    pub fn is_sample_frequency_shader(&self) -> bool {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.IsSampleFrequencyShader)(self.ptr) != 0
        }
    }

    /// Gets the number of interface slots.
    pub fn num_interface_slots(&self) -> u32 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetNumInterfaceSlots)(self.ptr)
        }
    }

    /// Gets the minimum feature level required.
    pub fn min_feature_level(&self) -> Result<u32> {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            let mut level = 0u32;
            let result = (vtable.GetMinFeatureLevel)(self.ptr, &mut level);
            if result == S_OK {
                Ok(level)
            } else {
                Err(Error::Reflection {
                    hresult: HResult(result),
                })
            }
        }
    }

    /// Gets required feature flags.
    pub fn requires_flags(&self) -> u64 {
        unsafe {
            let vtable = &*(*self.ptr).vtable;
            (vtable.GetRequiresFlags)(self.ptr)
        }
    }

    /// Returns the raw pointer (for advanced use).
    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *mut ID3D11ShaderReflection {
        self.ptr
    }
}

impl Drop for ShaderReflection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let vtable = &*(*self.ptr).vtable;
                (vtable.Release)(self.ptr);
            }
        }
    }
}

unsafe impl Send for ShaderReflection {}
unsafe impl Sync for ShaderReflection {}

/// Iterator over constant buffers
pub struct ConstantBufferIter<'a> {
    reflection: &'a ShaderReflection,
    index: u32,
    count: u32,
}

impl<'a> Iterator for ConstantBufferIter<'a> {
    type Item = ConstantBuffer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let cb = self.reflection.constant_buffer(self.index)?;
        self.index += 1;
        Some(cb)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ConstantBufferIter<'_> {}

/// Iterator over input parameters
pub struct InputParameterIter<'a> {
    reflection: &'a ShaderReflection,
    index: u32,
    count: u32,
}

impl<'a> Iterator for InputParameterIter<'a> {
    type Item = SignatureParameter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let param = self.reflection.input_parameter(self.index)?;
        self.index += 1;
        Some(param)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for InputParameterIter<'_> {}

/// Iterator over output parameters
pub struct OutputParameterIter<'a> {
    reflection: &'a ShaderReflection,
    index: u32,
    count: u32,
}

impl<'a> Iterator for OutputParameterIter<'a> {
    type Item = SignatureParameter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let param = self.reflection.output_parameter(self.index)?;
        self.index += 1;
        Some(param)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for OutputParameterIter<'_> {}

/// Iterator over resource bindings
pub struct ResourceBindingIter<'a> {
    reflection: &'a ShaderReflection,
    index: u32,
    count: u32,
}

impl<'a> Iterator for ResourceBindingIter<'a> {
    type Item = ResourceBinding;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let binding = self.reflection.resource_binding(self.index)?;
        self.index += 1;
        Some(binding)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ResourceBindingIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ShaderTarget, compile};

    const VERTEX_SHADER: &str = r#"
        struct VS_INPUT {
            float3 pos : POSITION;
            float2 uv : TEXCOORD0;
        };

        struct VS_OUTPUT {
            float4 pos : SV_POSITION;
            float2 uv : TEXCOORD0;
        };

        cbuffer Constants : register(b0) {
            float4x4 worldViewProj;
        };

        VS_OUTPUT main(VS_INPUT input) {
            VS_OUTPUT output;
            output.pos = mul(float4(input.pos, 1.0), worldViewProj);
            output.uv = input.uv;
            return output;
        }
    "#;

    #[test]
    fn test_reflection_basic() {
        let bytecode = compile(VERTEX_SHADER, "main", ShaderTarget::VS_5_0).unwrap();
        let reflection = ShaderReflection::new(&bytecode).unwrap();

        let desc = reflection.desc().unwrap();
        assert!(desc.constant_buffers >= 1);
        assert!(desc.input_parameters >= 2);
        assert!(desc.output_parameters >= 1);
    }

    #[test]
    fn test_constant_buffer_iteration() {
        let bytecode = compile(VERTEX_SHADER, "main", ShaderTarget::VS_5_0).unwrap();
        let reflection = ShaderReflection::new(&bytecode).unwrap();

        let mut found_constants = false;
        for cb in reflection.constant_buffers() {
            if cb.name().contains("Constants") || cb.name().contains("$Globals") {
                found_constants = true;
            }
        }
        assert!(found_constants, "Should find the Constants buffer");
    }

    #[test]
    fn test_input_parameters() {
        let bytecode = compile(VERTEX_SHADER, "main", ShaderTarget::VS_5_0).unwrap();
        let reflection = ShaderReflection::new(&bytecode).unwrap();

        let params: Vec<_> = reflection.input_parameters().collect();
        assert!(params.len() >= 2);

        let semantics: Vec<_> = params.iter().map(|p| p.semantic_name.as_str()).collect();
        assert!(semantics.contains(&"POSITION") || semantics.contains(&"SV_POSITION"));
    }
}
