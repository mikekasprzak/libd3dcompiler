//! Compile, strip, and disassemble flags

use bitflags::bitflags;

bitflags! {
    /// D3DCOMPILE flags for shader compilation
    ///
    /// These flags control various aspects of shader compilation including
    /// optimization levels, debug info, and strictness settings.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CompileFlags: u32 {
        /// Insert debug information into the output
        const DEBUG = 1 << 0;

        /// Skip validation step (use with caution)
        const SKIP_VALIDATION = 1 << 1;

        /// Skip optimization passes
        const SKIP_OPTIMIZATION = 1 << 2;

        /// Pack matrices in row-major order
        const PACK_MATRIX_ROW_MAJOR = 1 << 3;

        /// Pack matrices in column-major order (default)
        const PACK_MATRIX_COLUMN_MAJOR = 1 << 4;

        /// Use partial precision (lower quality, potentially faster)
        const PARTIAL_PRECISION = 1 << 5;

        /// Force vertex shader to run on software vertex processing
        const FORCE_VS_SOFTWARE_NO_OPT = 1 << 6;

        /// Force pixel shader to run on software
        const FORCE_PS_SOFTWARE_NO_OPT = 1 << 7;

        /// Disable preshaders
        const NO_PRESHADER = 1 << 8;

        /// Avoid flow control constructs where possible
        const AVOID_FLOW_CONTROL = 1 << 9;

        /// Prefer flow control constructs
        const PREFER_FLOW_CONTROL = 1 << 10;

        /// Enable strict mode (fail on undefined behavior)
        const ENABLE_STRICTNESS = 1 << 11;

        /// Enable backwards compatibility mode
        const ENABLE_BACKWARDS_COMPATIBILITY = 1 << 12;

        /// Require IEEE strict floating-point behavior
        const IEEE_STRICTNESS = 1 << 13;

        /// Optimization level 0 (no optimization)
        const OPTIMIZATION_LEVEL0 = 1 << 14;

        /// Optimization level 1 (default)
        const OPTIMIZATION_LEVEL1 = 0;

        /// Optimization level 2
        const OPTIMIZATION_LEVEL2 = (1 << 14) | (1 << 15);

        /// Optimization level 3 (full optimization)
        const OPTIMIZATION_LEVEL3 = 1 << 15;

        /// Treat warnings as errors
        const WARNINGS_ARE_ERRORS = 1 << 18;

        /// Enable unbounded descriptor tables
        const RESOURCES_MAY_ALIAS = 1 << 19;

        /// Enable unbounded descriptor tables
        const ENABLE_UNBOUNDED_DESCRIPTOR_TABLES = 1 << 20;

        /// All resources are bound
        const ALL_RESOURCES_BOUND = 1 << 21;

        /// Generate debug name for source
        const DEBUG_NAME_FOR_SOURCE = 1 << 22;

        /// Generate debug name for binary
        const DEBUG_NAME_FOR_BINARY = 1 << 23;
    }
}

impl CompileFlags {
    /// Returns flags for a given optimization level (0-3)
    pub fn optimization_level(level: u32) -> Self {
        match level {
            0 => CompileFlags::OPTIMIZATION_LEVEL0,
            1 => CompileFlags::OPTIMIZATION_LEVEL1,
            2 => CompileFlags::OPTIMIZATION_LEVEL2,
            _ => CompileFlags::OPTIMIZATION_LEVEL3,
        }
    }

    /// Clears optimization level bits and sets the specified level
    pub fn with_optimization_level(self, level: u32) -> Self {
        // Clear existing optimization flags
        let cleared = self
            & !(CompileFlags::OPTIMIZATION_LEVEL0
                | CompileFlags::OPTIMIZATION_LEVEL2
                | CompileFlags::OPTIMIZATION_LEVEL3);

        cleared | Self::optimization_level(level)
    }
}

impl Default for CompileFlags {
    fn default() -> Self {
        CompileFlags::empty()
    }
}

bitflags! {
    /// D3DCOMPILER_STRIP flags for stripping shader data
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StripFlags: u32 {
        /// Strip debug information
        const DEBUG_INFO = 1 << 0;

        /// Strip reflection data
        const REFLECTION_DATA = 1 << 1;

        /// Strip test blobs
        const TEST_BLOBS = 1 << 2;

        /// Strip private data
        const PRIVATE_DATA = 1 << 3;

        /// Strip root signature
        const ROOT_SIGNATURE = 1 << 4;
    }
}

impl Default for StripFlags {
    fn default() -> Self {
        StripFlags::empty()
    }
}

bitflags! {
    /// D3D_DISASM flags for disassembly output
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DisassembleFlags: u32 {
        /// Enable color-coded output
        const ENABLE_COLOR_CODE = 1 << 0;

        /// Print default values for parameters
        const ENABLE_DEFAULT_VALUE_PRINTS = 1 << 1;

        /// Number each instruction
        const ENABLE_INSTRUCTION_NUMBERING = 1 << 2;

        /// Include instruction cycle counts
        const ENABLE_INSTRUCTION_CYCLE = 1 << 3;

        /// Disable debug info in output
        const DISABLE_DEBUG_INFO = 1 << 4;

        /// Include byte offset for each instruction
        const ENABLE_INSTRUCTION_OFFSET = 1 << 5;

        /// Output only instructions (no declarations)
        const INSTRUCTION_ONLY = 1 << 6;

        /// Print hex literals
        const PRINT_HEX_LITERALS = 1 << 7;
    }
}

impl Default for DisassembleFlags {
    fn default() -> Self {
        DisassembleFlags::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_flags_combine() {
        let flags = CompileFlags::DEBUG | CompileFlags::WARNINGS_ARE_ERRORS;
        assert!(flags.contains(CompileFlags::DEBUG));
        assert!(flags.contains(CompileFlags::WARNINGS_ARE_ERRORS));
        assert!(!flags.contains(CompileFlags::SKIP_OPTIMIZATION));
    }

    #[test]
    fn test_optimization_level() {
        let flags = CompileFlags::DEBUG.with_optimization_level(3);
        assert!(flags.contains(CompileFlags::DEBUG));
        assert!(flags.contains(CompileFlags::OPTIMIZATION_LEVEL3));
    }
}
