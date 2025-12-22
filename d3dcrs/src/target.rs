//! Shader target types (shader type + shader model)

use std::ffi::CString;
use std::fmt;

/// Shader type (vertex, pixel, compute, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    /// Vertex shader
    Vertex,
    /// Pixel (fragment) shader
    Pixel,
    /// Geometry shader
    Geometry,
    /// Hull (tessellation control) shader
    Hull,
    /// Domain (tessellation evaluation) shader
    Domain,
    /// Compute shader
    Compute,
}

impl ShaderType {
    /// Returns the shader type prefix (vs, ps, gs, hs, ds, cs)
    pub fn prefix(&self) -> &'static str {
        match self {
            ShaderType::Vertex => "vs",
            ShaderType::Pixel => "ps",
            ShaderType::Geometry => "gs",
            ShaderType::Hull => "hs",
            ShaderType::Domain => "ds",
            ShaderType::Compute => "cs",
        }
    }
}

impl fmt::Display for ShaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.prefix())
    }
}

/// Shader model version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderModel {
    /// Shader Model 4.0
    SM4_0,
    /// Shader Model 4.1
    SM4_1,
    /// Shader Model 5.0
    SM5_0,
    /// Shader Model 5.1
    SM5_1,
    /// Shader Model 6.0
    SM6_0,
    /// Shader Model 6.1
    SM6_1,
    /// Shader Model 6.2
    SM6_2,
    /// Shader Model 6.3
    SM6_3,
    /// Shader Model 6.4
    SM6_4,
    /// Shader Model 6.5
    SM6_5,
    /// Shader Model 6.6
    SM6_6,
    /// Shader Model 6.7
    SM6_7,
}

impl ShaderModel {
    /// Returns the shader model suffix (4_0, 4_1, 5_0, etc.)
    pub fn suffix(&self) -> &'static str {
        match self {
            ShaderModel::SM4_0 => "4_0",
            ShaderModel::SM4_1 => "4_1",
            ShaderModel::SM5_0 => "5_0",
            ShaderModel::SM5_1 => "5_1",
            ShaderModel::SM6_0 => "6_0",
            ShaderModel::SM6_1 => "6_1",
            ShaderModel::SM6_2 => "6_2",
            ShaderModel::SM6_3 => "6_3",
            ShaderModel::SM6_4 => "6_4",
            ShaderModel::SM6_5 => "6_5",
            ShaderModel::SM6_6 => "6_6",
            ShaderModel::SM6_7 => "6_7",
        }
    }

    /// Returns the major version number
    pub fn major(&self) -> u32 {
        match self {
            ShaderModel::SM4_0 | ShaderModel::SM4_1 => 4,
            ShaderModel::SM5_0 | ShaderModel::SM5_1 => 5,
            _ => 6,
        }
    }

    /// Returns the minor version number
    pub fn minor(&self) -> u32 {
        match self {
            ShaderModel::SM4_0 | ShaderModel::SM5_0 | ShaderModel::SM6_0 => 0,
            ShaderModel::SM4_1 | ShaderModel::SM5_1 | ShaderModel::SM6_1 => 1,
            ShaderModel::SM6_2 => 2,
            ShaderModel::SM6_3 => 3,
            ShaderModel::SM6_4 => 4,
            ShaderModel::SM6_5 => 5,
            ShaderModel::SM6_6 => 6,
            ShaderModel::SM6_7 => 7,
        }
    }
}

impl fmt::Display for ShaderModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.suffix())
    }
}

/// Complete shader target specification (type + model)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderTarget {
    /// The shader type
    pub shader_type: ShaderType,
    /// The shader model version
    pub model: ShaderModel,
}

impl ShaderTarget {
    // Vertex shader targets
    pub const VS_4_0: ShaderTarget = ShaderTarget::new(ShaderType::Vertex, ShaderModel::SM4_0);
    pub const VS_4_1: ShaderTarget = ShaderTarget::new(ShaderType::Vertex, ShaderModel::SM4_1);
    pub const VS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Vertex, ShaderModel::SM5_0);
    pub const VS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Vertex, ShaderModel::SM5_1);

    // Pixel shader targets
    pub const PS_4_0: ShaderTarget = ShaderTarget::new(ShaderType::Pixel, ShaderModel::SM4_0);
    pub const PS_4_1: ShaderTarget = ShaderTarget::new(ShaderType::Pixel, ShaderModel::SM4_1);
    pub const PS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Pixel, ShaderModel::SM5_0);
    pub const PS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Pixel, ShaderModel::SM5_1);

    // Geometry shader targets
    pub const GS_4_0: ShaderTarget = ShaderTarget::new(ShaderType::Geometry, ShaderModel::SM4_0);
    pub const GS_4_1: ShaderTarget = ShaderTarget::new(ShaderType::Geometry, ShaderModel::SM4_1);
    pub const GS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Geometry, ShaderModel::SM5_0);
    pub const GS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Geometry, ShaderModel::SM5_1);

    // Hull shader targets (SM5+)
    pub const HS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Hull, ShaderModel::SM5_0);
    pub const HS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Hull, ShaderModel::SM5_1);

    // Domain shader targets (SM5+)
    pub const DS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Domain, ShaderModel::SM5_0);
    pub const DS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Domain, ShaderModel::SM5_1);

    // Compute shader targets (SM5+)
    pub const CS_4_0: ShaderTarget = ShaderTarget::new(ShaderType::Compute, ShaderModel::SM4_0);
    pub const CS_4_1: ShaderTarget = ShaderTarget::new(ShaderType::Compute, ShaderModel::SM4_1);
    pub const CS_5_0: ShaderTarget = ShaderTarget::new(ShaderType::Compute, ShaderModel::SM5_0);
    pub const CS_5_1: ShaderTarget = ShaderTarget::new(ShaderType::Compute, ShaderModel::SM5_1);

    /// Creates a new shader target
    pub const fn new(shader_type: ShaderType, model: ShaderModel) -> Self {
        ShaderTarget { shader_type, model }
    }

    /// Returns the target string (e.g., "vs_5_0")
    pub fn as_str(&self) -> String {
        format!("{}_{}", self.shader_type.prefix(), self.model.suffix())
    }

    /// Returns the target as a null-terminated C string
    pub(crate) fn as_cstring(&self) -> CString {
        CString::new(self.as_str()).unwrap()
    }
}

impl fmt::Display for ShaderTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.shader_type.prefix(), self.model.suffix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_strings() {
        assert_eq!(ShaderTarget::VS_5_0.as_str(), "vs_5_0");
        assert_eq!(ShaderTarget::PS_5_1.as_str(), "ps_5_1");
        assert_eq!(ShaderTarget::CS_5_0.as_str(), "cs_5_0");
        assert_eq!(ShaderTarget::GS_4_0.as_str(), "gs_4_0");
    }

    #[test]
    fn test_shader_model_versions() {
        assert_eq!(ShaderModel::SM5_0.major(), 5);
        assert_eq!(ShaderModel::SM5_0.minor(), 0);
        assert_eq!(ShaderModel::SM5_1.major(), 5);
        assert_eq!(ShaderModel::SM5_1.minor(), 1);
    }
}
