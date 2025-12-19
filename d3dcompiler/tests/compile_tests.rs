//! Integration tests for d3dcompiler
//!
//! These tests require d3dcompiler_47.dll to be present in the project root
//! or next to the test binary.

#![allow(unsafe_op_in_unsafe_fn)]

use d3dcompiler::*;
use std::ffi::CStr;
use std::ptr;

/// Helper to initialize and get blob data
unsafe fn get_blob_data(blob: *mut ID3DBlob) -> Vec<u8> {
    if blob.is_null() {
        return Vec::new();
    }
    let vtable = &*(*blob).vtable;
    let ptr = (vtable.GetBufferPointer)(blob);
    let size = (vtable.GetBufferSize)(blob);
    std::slice::from_raw_parts(ptr as *const u8, size).to_vec()
}

/// Helper to release a blob
unsafe fn release_blob(blob: *mut ID3DBlob) {
    if !blob.is_null() {
        let vtable = &*(*blob).vtable;
        (vtable.Release)(blob);
    }
}

/// Helper to get error message from blob
unsafe fn get_error_message(blob: *mut ID3DBlob) -> String {
    if blob.is_null() {
        return String::new();
    }
    let data = get_blob_data(blob);
    String::from_utf8_lossy(&data).to_string()
}

// Simple vertex shader
const VERTEX_SHADER: &[u8] = b"
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
\0";

// Simple pixel shader
const PIXEL_SHADER: &[u8] = b"
Texture2D tex : register(t0);
SamplerState samp : register(s0);

struct PS_INPUT {
    float4 pos : SV_POSITION;
    float2 uv : TEXCOORD0;
};

float4 main(PS_INPUT input) : SV_TARGET {
    return tex.Sample(samp, input.uv);
}
\0";

// Compute shader
const COMPUTE_SHADER: &[u8] = b"
RWStructuredBuffer<float> output : register(u0);
StructuredBuffer<float> input : register(t0);

[numthreads(64, 1, 1)]
void main(uint3 id : SV_DispatchThreadID) {
    output[id.x] = input[id.x] * 2.0;
}
\0";

// Shader with intentional error for testing error handling
const BAD_SHADER: &[u8] = b"
float4 main() : SV_TARGET {
    return undefined_variable;
}
\0";

// Hull and Domain shader for tessellation
const HULL_SHADER: &[u8] = b"
struct VS_OUTPUT {
    float4 pos : SV_POSITION;
};

struct HS_OUTPUT {
    float4 pos : SV_POSITION;
};

struct HS_CONSTANT_OUTPUT {
    float edges[3] : SV_TessFactor;
    float inside : SV_InsideTessFactor;
};

HS_CONSTANT_OUTPUT PatchConstantFunc(InputPatch<VS_OUTPUT, 3> patch) {
    HS_CONSTANT_OUTPUT output;
    output.edges[0] = 2.0;
    output.edges[1] = 2.0;
    output.edges[2] = 2.0;
    output.inside = 2.0;
    return output;
}

[domain(\"tri\")]
[partitioning(\"fractional_odd\")]
[outputtopology(\"triangle_cw\")]
[outputcontrolpoints(3)]
[patchconstantfunc(\"PatchConstantFunc\")]
HS_OUTPUT main(InputPatch<VS_OUTPUT, 3> patch, uint id : SV_OutputControlPointID) {
    HS_OUTPUT output;
    output.pos = patch[id].pos;
    return output;
}
\0";

#[test]
fn test_compile_vertex_shader() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1, // exclude null terminator from length
            c"vertex.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Vertex shader compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(
            !bytecode.is_empty(),
            "Compiled bytecode should not be empty"
        );

        // DXBC bytecode starts with "DXBC" magic
        assert_eq!(
            &bytecode[0..4],
            b"DXBC",
            "Bytecode should start with DXBC magic"
        );

        println!(
            "Vertex shader compiled successfully: {} bytes",
            bytecode.len()
        );

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_pixel_shader() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            PIXEL_SHADER.as_ptr() as *const _,
            PIXEL_SHADER.len() - 1,
            c"pixel.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Pixel shader compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(!bytecode.is_empty());
        assert_eq!(&bytecode[0..4], b"DXBC");

        println!(
            "Pixel shader compiled successfully: {} bytes",
            bytecode.len()
        );

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_hull_shader() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            HULL_SHADER.as_ptr() as *const _,
            HULL_SHADER.len() - 1,
            c"hull.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"hs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Hull shader compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(!bytecode.is_empty());
        assert_eq!(&bytecode[0..4], b"DXBC");

        println!(
            "Hull shader compiled successfully: {} bytes",
            bytecode.len()
        );

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_compute_shader() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            COMPUTE_SHADER.as_ptr() as *const _,
            COMPUTE_SHADER.len() - 1,
            c"compute.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"cs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Compute shader compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(!bytecode.is_empty());
        assert_eq!(&bytecode[0..4], b"DXBC");

        println!(
            "Compute shader compiled successfully: {} bytes",
            bytecode.len()
        );

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_with_error() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            BAD_SHADER.as_ptr() as *const _,
            BAD_SHADER.len() - 1,
            c"bad.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        // Should fail
        assert_ne!(result, S_OK, "Bad shader should fail to compile");

        // Should have error message
        assert!(!errors.is_null(), "Should have error blob");
        let err_msg = get_error_message(errors);
        assert!(!err_msg.is_empty(), "Error message should not be empty");
        assert!(
            err_msg.contains("undefined") || err_msg.contains("undeclared"),
            "Error should mention undefined variable: {}",
            err_msg
        );

        println!("Got expected error: {}", err_msg.trim());

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_with_defines() {
    let shader = b"
#ifdef USE_RED
float4 main() : SV_TARGET { return float4(1, 0, 0, 1); }
#else
float4 main() : SV_TARGET { return float4(0, 1, 0, 1); }
#endif
\0";

    // Define USE_RED
    let defines = [
        D3D_SHADER_MACRO {
            Name: c"USE_RED".as_ptr(),
            Definition: c"1".as_ptr(),
        },
        D3D_SHADER_MACRO {
            Name: ptr::null(),
            Definition: ptr::null(),
        },
    ];

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            shader.as_ptr() as *const _,
            shader.len() - 1,
            c"defines.hlsl".as_ptr(),
            defines.as_ptr(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Shader with defines failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(!bytecode.is_empty());

        println!(
            "Shader with defines compiled successfully: {} bytes",
            bytecode.len()
        );

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_with_optimization_levels() {
    let shader = b"
float4 main(float4 pos : SV_POSITION) : SV_TARGET {
    float x = pos.x;
    float y = pos.y;
    float z = x + y;
    float w = z * 2.0;
    return float4(w, w, w, 1.0);
}
\0";

    // Test different optimization levels
    let opt_flags = [
        (0u32, "O0 (none)"),                 // No optimization
        (1u32 << 14, "O1"),                  // D3DCOMPILE_OPTIMIZATION_LEVEL1
        (1u32 << 15, "O2"),                  // D3DCOMPILE_OPTIMIZATION_LEVEL2
        ((1u32 << 14) | (1u32 << 15), "O3"), // D3DCOMPILE_OPTIMIZATION_LEVEL3
    ];

    for (flags, name) in opt_flags {
        unsafe {
            let mut code: *mut ID3DBlob = ptr::null_mut();
            let mut errors: *mut ID3DBlob = ptr::null_mut();

            let result = D3DCompile(
                shader.as_ptr() as *const _,
                shader.len() - 1,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                c"main".as_ptr(),
                c"ps_5_0".as_ptr(),
                flags,
                0,
                &mut code,
                &mut errors,
            );

            if result != S_OK {
                let err_msg = get_error_message(errors);
                release_blob(errors);
                panic!("Compilation with {} failed: {}", name, err_msg);
            }

            let bytecode = get_blob_data(code);
            println!("Optimization {}: {} bytes", name, bytecode.len());

            release_blob(code);
            release_blob(errors);
        }
    }
}

#[test]
fn test_disassemble() {
    // First compile a shader
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            PIXEL_SHADER.as_ptr() as *const _,
            PIXEL_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK, "Initial compilation should succeed");

        let bytecode = get_blob_data(code);

        // Now disassemble it
        let mut disasm: *mut ID3DBlob = ptr::null_mut();

        let result = D3DDisassemble(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            0,
            ptr::null(),
            &mut disasm,
        );

        if result != S_OK {
            release_blob(code);
            panic!("Disassembly failed with code: 0x{:08x}", result as u32);
        }

        let disasm_text = get_blob_data(disasm);
        let text = String::from_utf8_lossy(&disasm_text);

        assert!(!text.is_empty(), "Disassembly should not be empty");
        assert!(text.contains("ps_5_0"), "Should contain shader model");

        println!(
            "Disassembly:\n{}",
            text.chars().take(500).collect::<String>()
        );

        release_blob(disasm);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_create_blob() {
    unsafe {
        let mut blob: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCreateBlob(1024, &mut blob);

        assert_eq!(result, S_OK, "D3DCreateBlob should succeed");
        assert!(!blob.is_null(), "Blob should not be null");

        let vtable = &*(*blob).vtable;
        let size = (vtable.GetBufferSize)(blob);
        assert_eq!(size, 1024, "Blob should be 1024 bytes");

        let ptr = (vtable.GetBufferPointer)(blob);
        assert!(!ptr.is_null(), "Buffer pointer should not be null");

        // Write some data
        let data = ptr as *mut u8;
        for i in 0..1024 {
            *data.add(i) = (i & 0xFF) as u8;
        }

        println!("Created and used 1024-byte blob successfully");

        release_blob(blob);
    }
}

#[test]
fn test_preprocess() {
    let shader = b"
#define PI 3.14159
#define SCALE 2.0

float4 main(float4 pos : SV_POSITION) : SV_TARGET {
    return pos * PI * SCALE;
}
\0";

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DPreprocess(
            shader.as_ptr() as *const _,
            shader.len() - 1,
            c"preprocess.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Preprocessing failed: {}", err_msg);
        }

        let preprocessed = get_blob_data(code);
        let text = String::from_utf8_lossy(&preprocessed);

        assert!(!text.is_empty(), "Preprocessed output should not be empty");
        // After preprocessing, PI and SCALE should be expanded
        assert!(text.contains("3.14159"), "Should contain expanded PI value");
        assert!(text.contains("2.0"), "Should contain expanded SCALE value");
        // Original defines should be gone
        assert!(!text.contains("#define PI"), "Defines should be expanded");

        println!("Preprocessed output:\n{}", text);

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile_shader_model_variations() {
    let simple_ps = b"
float4 main() : SV_TARGET {
    return float4(1, 0, 0, 1);
}
\0";

    // Test various shader models
    let models = ["ps_4_0", "ps_4_1", "ps_5_0", "ps_5_1"];

    for model in models {
        let model_cstr = format!("{}\0", model);

        unsafe {
            let mut code: *mut ID3DBlob = ptr::null_mut();
            let mut errors: *mut ID3DBlob = ptr::null_mut();

            let result = D3DCompile(
                simple_ps.as_ptr() as *const _,
                simple_ps.len() - 1,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                c"main".as_ptr(),
                model_cstr.as_ptr() as *const _,
                0,
                0,
                &mut code,
                &mut errors,
            );

            if result == S_OK {
                let bytecode = get_blob_data(code);
                println!("Shader model {}: {} bytes", model, bytecode.len());
            } else {
                let err_msg = get_error_message(errors);
                println!("Shader model {} not supported: {}", model, err_msg.trim());
            }

            release_blob(code);
            release_blob(errors);
        }
    }
}

#[test]
fn test_strip_shader() {
    // Compile with debug info
    let shader = b"
// This is a comment that should be stripped
float4 main(float4 pos : SV_POSITION) : SV_TARGET {
    float x = pos.x; // inline comment
    return float4(x, x, x, 1);
}
\0";

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        // Compile with debug info
        let d3dcompile_debug = 1u32 << 0;

        let result = D3DCompile(
            shader.as_ptr() as *const _,
            shader.len() - 1,
            c"strip_test.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            d3dcompile_debug,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Compilation failed: {}", err_msg);
        }

        let original_bytecode = get_blob_data(code);
        let original_size = original_bytecode.len();

        // Strip debug info
        let mut stripped: *mut ID3DBlob = ptr::null_mut();
        let d3dcompiler_strip_debug_info = 1u32 << 0;

        let result = D3DStripShader(
            original_bytecode.as_ptr() as *const _,
            original_bytecode.len(),
            d3dcompiler_strip_debug_info,
            &mut stripped,
        );

        if result == S_OK && !stripped.is_null() {
            let stripped_bytecode = get_blob_data(stripped);
            println!(
                "Original: {} bytes, Stripped: {} bytes (saved {} bytes)",
                original_size,
                stripped_bytecode.len(),
                original_size - stripped_bytecode.len()
            );
            release_blob(stripped);
        } else {
            println!("Strip not supported or failed (this may be expected)");
        }

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_compile2_with_secondary_data() {
    let shader = b"
float4 main() : SV_TARGET {
    return float4(1, 0, 0, 1);
}
\0";

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        // D3DCompile2 with no secondary data (should behave like D3DCompile)
        let result = D3DCompile2(
            shader.as_ptr() as *const _,
            shader.len() - 1,
            c"compile2.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            0,           // SecondaryDataFlags
            ptr::null(), // pSecondaryData
            0,           // SecondaryDataSize
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("D3DCompile2 failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);
        assert!(!bytecode.is_empty());
        assert_eq!(&bytecode[0..4], b"DXBC");

        println!("D3DCompile2 succeeded: {} bytes", bytecode.len());

        release_blob(code);
        release_blob(errors);
    }
}

// Benchmark-style test for compilation performance
#[test]
fn test_compilation_performance() {
    let shader = b"
cbuffer CB : register(b0) {
    float4x4 mvp;
    float4 color;
    float time;
};

struct VS_INPUT {
    float3 pos : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
};

struct VS_OUTPUT {
    float4 pos : SV_POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR;
};

VS_OUTPUT main(VS_INPUT input) {
    VS_OUTPUT output;
    output.pos = mul(float4(input.pos, 1.0), mvp);
    output.normal = input.normal;
    output.uv = input.uv;
    output.color = color * (sin(time) * 0.5 + 0.5);
    return output;
}
\0";

    let iterations = 10;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        unsafe {
            let mut code: *mut ID3DBlob = ptr::null_mut();
            let mut errors: *mut ID3DBlob = ptr::null_mut();

            let result = D3DCompile(
                shader.as_ptr() as *const _,
                shader.len() - 1,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                c"main".as_ptr(),
                c"vs_5_0".as_ptr(),
                0,
                0,
                &mut code,
                &mut errors,
            );

            assert_eq!(result, S_OK);
            release_blob(code);
            release_blob(errors);
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Compiled {} shaders in {:?} ({:.2} ms per shader)",
        iterations,
        elapsed,
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
}

// ============================================================================
// Shader Reflection API Tests
// ============================================================================

/// Helper to release a reflection interface
unsafe fn release_reflection(refl: *mut ID3D11ShaderReflection) {
    if !refl.is_null() {
        let vtable = &*(*refl).vtable;
        (vtable.Release)(refl);
    }
}

/// IID for ID3D11ShaderReflection: {8d536ca1-0cca-4956-a837-786963755584}
const IID_ID3D11SHADERREFLECTION: [u8; 16] = [
    0xa1, 0x6c, 0x53, 0x8d, 0xca, 0x0c, 0x56, 0x49, 0xa8, 0x37, 0x78, 0x69, 0x63, 0x75, 0x55, 0x84,
];

#[test]
fn test_reflect_vertex_shader() {
    unsafe {
        // First compile a shader
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            c"reflect_test.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);

        // Get reflection interface
        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        let result = D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        assert_eq!(result, S_OK, "D3DReflect should succeed");
        assert!(!reflector.is_null(), "Reflector should not be null");

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        // Get shader description
        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        let result = (vtable.GetDesc)(refl, &mut desc);
        assert_eq!(result, S_OK, "GetDesc should succeed");

        println!("Shader Description:");
        println!("  Version: 0x{:08x}", desc.Version);
        println!("  Constant Buffers: {}", desc.ConstantBuffers);
        println!("  Bound Resources: {}", desc.BoundResources);
        println!("  Input Parameters: {}", desc.InputParameters);
        println!("  Output Parameters: {}", desc.OutputParameters);
        println!("  Instruction Count: {}", desc.InstructionCount);

        // Verify we have expected inputs (position, texcoord)
        assert!(
            desc.InputParameters >= 2,
            "Should have at least 2 input parameters"
        );
        // Verify we have expected outputs
        assert!(
            desc.OutputParameters >= 1,
            "Should have at least 1 output parameter"
        );
        // Should have constant buffer
        assert!(
            desc.ConstantBuffers >= 1,
            "Should have at least 1 constant buffer"
        );

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_input_parameters() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        let result = D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );
        assert_eq!(result, S_OK);

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        (vtable.GetDesc)(refl, &mut desc);

        println!("Input Parameters:");
        for i in 0..desc.InputParameters {
            let mut param: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
            let result = (vtable.GetInputParameterDesc)(refl, i, &mut param);
            assert_eq!(
                result, S_OK,
                "GetInputParameterDesc should succeed for index {}",
                i
            );

            let semantic = if !param.SemanticName.is_null() {
                CStr::from_ptr(param.SemanticName).to_string_lossy()
            } else {
                "<null>".into()
            };

            println!(
                "  [{}] Semantic: {}{}, Register: {}, Mask: 0x{:02x}",
                i, semantic, param.SemanticIndex, param.Register, param.Mask
            );
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_output_parameters() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        (vtable.GetDesc)(refl, &mut desc);

        println!("Output Parameters:");
        for i in 0..desc.OutputParameters {
            let mut param: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
            let result = (vtable.GetOutputParameterDesc)(refl, i, &mut param);
            assert_eq!(result, S_OK);

            let semantic = if !param.SemanticName.is_null() {
                CStr::from_ptr(param.SemanticName).to_string_lossy()
            } else {
                "<null>".into()
            };

            println!(
                "  [{}] Semantic: {}{}, Register: {}, Mask: 0x{:02x}",
                i, semantic, param.SemanticIndex, param.Register, param.Mask
            );
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_constant_buffer() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        (vtable.GetDesc)(refl, &mut desc);

        println!("Constant Buffers: {}", desc.ConstantBuffers);

        for i in 0..desc.ConstantBuffers {
            let cb = (vtable.GetConstantBufferByIndex)(refl, i);
            assert!(!cb.is_null(), "Constant buffer {} should not be null", i);

            let cb_vtable = &*(*cb).vtable;

            let mut cb_desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
            let result = (cb_vtable.GetDesc)(cb, &mut cb_desc);
            assert_eq!(result, S_OK, "GetDesc should succeed for CB {}", i);

            let name = if !cb_desc.Name.is_null() {
                CStr::from_ptr(cb_desc.Name).to_string_lossy()
            } else {
                "<null>".into()
            };

            println!(
                "  CB[{}]: Name='{}', Size={}, Variables={}",
                i, name, cb_desc.Size, cb_desc.Variables
            );

            // Get variables in this constant buffer
            for j in 0..cb_desc.Variables {
                let var = (cb_vtable.GetVariableByIndex)(cb, j);
                assert!(
                    !var.is_null(),
                    "Variable {} in CB {} should not be null",
                    j,
                    i
                );

                let var_vtable = &*(*var).vtable;
                let mut var_desc: D3D11_SHADER_VARIABLE_DESC = std::mem::zeroed();
                let result = (var_vtable.GetDesc)(var, &mut var_desc);
                assert_eq!(result, S_OK);

                let var_name = if !var_desc.Name.is_null() {
                    CStr::from_ptr(var_desc.Name).to_string_lossy()
                } else {
                    "<null>".into()
                };

                println!(
                    "    Var[{}]: Name='{}', Offset={}, Size={}",
                    j, var_name, var_desc.StartOffset, var_desc.Size
                );

                // Get variable type
                let var_type = (var_vtable.GetType)(var);
                if !var_type.is_null() {
                    let type_vtable = &*(*var_type).vtable;
                    let mut type_desc: D3D11_SHADER_TYPE_DESC = std::mem::zeroed();
                    let result = (type_vtable.GetDesc)(var_type, &mut type_desc);
                    if result == S_OK {
                        println!(
                            "      Type: Class={}, Rows={}, Cols={}",
                            type_desc.Class, type_desc.Rows, type_desc.Columns
                        );
                    }
                }
            }
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_get_constant_buffer_by_name() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        // Look up "Constants" buffer by name
        let cb = (vtable.GetConstantBufferByName)(refl, c"Constants".as_ptr());
        // Note: May return a "null" CB object (with null vtable entries) rather than null pointer
        // This is Windows D3D behavior - it returns a valid object that returns error on GetDesc

        if !cb.is_null() {
            let cb_vtable = &*(*cb).vtable;
            let mut cb_desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
            let result = (cb_vtable.GetDesc)(cb, &mut cb_desc);
            if result == S_OK {
                let name = if !cb_desc.Name.is_null() {
                    CStr::from_ptr(cb_desc.Name).to_string_lossy()
                } else {
                    "<null>".into()
                };
                println!("Found Constants buffer by name: '{}'", name);
                assert!(name.contains("Constants") || name.contains("$Globals"));
            }
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_resource_bindings() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            PIXEL_SHADER.as_ptr() as *const _,
            PIXEL_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        (vtable.GetDesc)(refl, &mut desc);

        println!("Bound Resources: {}", desc.BoundResources);

        for i in 0..desc.BoundResources {
            let mut bind_desc: D3D11_SHADER_INPUT_BIND_DESC = std::mem::zeroed();
            let result = (vtable.GetResourceBindingDesc)(refl, i, &mut bind_desc);
            assert_eq!(result, S_OK, "GetResourceBindingDesc should succeed");

            let name = if !bind_desc.Name.is_null() {
                CStr::from_ptr(bind_desc.Name).to_string_lossy()
            } else {
                "<null>".into()
            };

            println!(
                "  Resource[{}]: Name='{}', Type={}, BindPoint={}, BindCount={}",
                i, name, bind_desc.Type, bind_desc.BindPoint, bind_desc.BindCount
            );
        }

        // Pixel shader should have texture and sampler bindings
        assert!(
            desc.BoundResources >= 2,
            "Pixel shader should have at least texture and sampler"
        );

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_instruction_counts() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mov_count = (vtable.GetMovInstructionCount)(refl);
        let movc_count = (vtable.GetMovcInstructionCount)(refl);
        let conv_count = (vtable.GetConversionInstructionCount)(refl);
        let bitwise_count = (vtable.GetBitwiseInstructionCount)(refl);

        println!("Instruction Counts:");
        println!("  MOV: {}", mov_count);
        println!("  MOVC: {}", movc_count);
        println!("  Conversion: {}", conv_count);
        println!("  Bitwise: {}", bitwise_count);

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_compute_shader_thread_group() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            COMPUTE_SHADER.as_ptr() as *const _,
            COMPUTE_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"cs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut z: u32 = 0;
        let total = (vtable.GetThreadGroupSize)(refl, &mut x, &mut y, &mut z);

        println!("Thread Group Size: {}x{}x{} = {}", x, y, z, total);

        // Our compute shader uses [numthreads(64, 1, 1)]
        assert_eq!(x, 64, "X thread count should be 64");
        assert_eq!(y, 1, "Y thread count should be 1");
        assert_eq!(z, 1, "Z thread count should be 1");
        assert_eq!(total, 64, "Total thread count should be 64");

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_min_feature_level() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut level: u32 = 0;
        let result = (vtable.GetMinFeatureLevel)(refl, &mut level);

        if result == S_OK {
            println!("Min Feature Level: 0x{:x}", level);
        } else {
            println!(
                "GetMinFeatureLevel not supported (result: 0x{:08x})",
                result as u32
            );
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_requires_flags() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            COMPUTE_SHADER.as_ptr() as *const _,
            COMPUTE_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"cs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let flags = (vtable.GetRequiresFlags)(refl);
        println!("Requires Flags: 0x{:016x}", flags);

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

// ============================================================================
// ID3DBlob Reference Counting Tests
// ============================================================================

#[test]
fn test_blob_addref_release() {
    unsafe {
        let mut blob: *mut ID3DBlob = ptr::null_mut();
        let result = D3DCreateBlob(256, &mut blob);
        assert_eq!(result, S_OK);
        assert!(!blob.is_null());

        let vtable = &*(*blob).vtable;

        // AddRef should return 2
        let count = (vtable.AddRef)(blob);
        assert_eq!(count, 2, "After AddRef, count should be 2");

        // AddRef again should return 3
        let count = (vtable.AddRef)(blob);
        assert_eq!(count, 3, "After second AddRef, count should be 3");

        // Release should return 2
        let count = (vtable.Release)(blob);
        assert_eq!(count, 2, "After Release, count should be 2");

        // Release should return 1
        let count = (vtable.Release)(blob);
        assert_eq!(count, 1, "After Release, count should be 1");

        // Final release should return 0 and free
        let count = (vtable.Release)(blob);
        assert_eq!(count, 0, "Final Release should return 0");

        println!("AddRef/Release test passed");
    }
}

#[test]
fn test_blob_buffer_operations() {
    unsafe {
        let mut blob: *mut ID3DBlob = ptr::null_mut();
        let size = 512usize;
        let result = D3DCreateBlob(size, &mut blob);
        assert_eq!(result, S_OK);

        let vtable = &*(*blob).vtable;

        // Check size
        let actual_size = (vtable.GetBufferSize)(blob);
        assert_eq!(actual_size, size, "Buffer size should match requested size");

        // Get pointer and write data
        let ptr = (vtable.GetBufferPointer)(blob);
        assert!(!ptr.is_null());

        // Write a pattern
        let data = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }

        // Read back and verify
        let read_ptr = (vtable.GetBufferPointer)(blob);
        let read_data = std::slice::from_raw_parts(read_ptr as *const u8, size);
        for (i, byte) in read_data.iter().enumerate() {
            assert_eq!(*byte, (i % 256) as u8, "Data mismatch at offset {}", i);
        }

        println!("Buffer operations test passed");
        release_blob(blob);
    }
}

// ============================================================================
// D3DGetBlobPart / D3DSetBlobPart Tests
// ============================================================================

// D3D_BLOB_PART constants
#[allow(dead_code)]
const D3D_BLOB_INPUT_SIGNATURE_BLOB: u32 = 0;
#[allow(dead_code)]
const D3D_BLOB_OUTPUT_SIGNATURE_BLOB: u32 = 1;
#[allow(dead_code)]
const D3D_BLOB_INPUT_AND_OUTPUT_SIGNATURE_BLOB: u32 = 2;
#[allow(dead_code)]
const D3D_BLOB_PATCH_CONSTANT_SIGNATURE_BLOB: u32 = 3;
#[allow(dead_code)]
const D3D_BLOB_ALL_SIGNATURE_BLOB: u32 = 4;
#[allow(dead_code)]
const D3D_BLOB_DEBUG_INFO: u32 = 5;
#[allow(dead_code)]
const D3D_BLOB_LEGACY_SHADER: u32 = 6;
#[allow(dead_code)]
const D3D_BLOB_XNA_PREPASS_SHADER: u32 = 7;
#[allow(dead_code)]
const D3D_BLOB_XNA_SHADER: u32 = 8;
#[allow(dead_code)]
const D3D_BLOB_PDB: u32 = 9;
#[allow(dead_code)]
const D3D_BLOB_PRIVATE_DATA: u32 = 10;

#[test]
fn test_get_blob_part_signatures() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        // Try to get input signature
        let mut input_sig: *mut ID3DBlob = ptr::null_mut();
        let result = D3DGetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            D3D_BLOB_INPUT_SIGNATURE_BLOB,
            0,
            &mut input_sig,
        );

        if result == S_OK && !input_sig.is_null() {
            let sig_data = get_blob_data(input_sig);
            println!("Input signature blob: {} bytes", sig_data.len());
            release_blob(input_sig);
        } else {
            println!(
                "Could not get input signature (result: 0x{:08x})",
                result as u32
            );
        }

        // Try to get output signature
        let mut output_sig: *mut ID3DBlob = ptr::null_mut();
        let result = D3DGetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            D3D_BLOB_OUTPUT_SIGNATURE_BLOB,
            0,
            &mut output_sig,
        );

        if result == S_OK && !output_sig.is_null() {
            let sig_data = get_blob_data(output_sig);
            println!("Output signature blob: {} bytes", sig_data.len());
            release_blob(output_sig);
        } else {
            println!(
                "Could not get output signature (result: 0x{:08x})",
                result as u32
            );
        }

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_get_blob_part_debug_info() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        // Compile with debug info
        let d3dcompile_debug = 1u32 << 0;

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            c"debug_test.hlsl".as_ptr(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            d3dcompile_debug,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        // Try to get debug info
        let mut debug_info: *mut ID3DBlob = ptr::null_mut();
        let result = D3DGetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            D3D_BLOB_DEBUG_INFO,
            0,
            &mut debug_info,
        );

        if result == S_OK && !debug_info.is_null() {
            let debug_data = get_blob_data(debug_info);
            println!("Debug info blob: {} bytes", debug_data.len());
            release_blob(debug_info);
        } else {
            println!(
                "Could not get debug info (result: 0x{:08x}) - may not be present",
                result as u32
            );
        }

        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_set_blob_part_private_data() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            PIXEL_SHADER.as_ptr() as *const _,
            PIXEL_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);
        let original_size = bytecode.len();

        // Add private data
        let private_data = b"Custom private data for testing!";
        let mut new_shader: *mut ID3DBlob = ptr::null_mut();

        let result = D3DSetBlobPart(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            D3D_BLOB_PRIVATE_DATA,
            0,
            private_data.as_ptr() as *const _,
            private_data.len(),
            &mut new_shader,
        );

        if result == S_OK && !new_shader.is_null() {
            let new_bytecode = get_blob_data(new_shader);
            println!(
                "Original size: {} bytes, With private data: {} bytes",
                original_size,
                new_bytecode.len()
            );
            assert!(
                new_bytecode.len() >= original_size,
                "New shader should be at least as large as original"
            );

            // Verify it's still valid DXBC
            assert_eq!(&new_bytecode[0..4], b"DXBC", "Should still be valid DXBC");

            release_blob(new_shader);
        } else {
            println!("D3DSetBlobPart failed (result: 0x{:08x})", result as u32);
        }

        release_blob(code);
        release_blob(errors);
    }
}

// ============================================================================
// Shader Type Reflection Tests (ID3D11ShaderReflectionType)
// ============================================================================

// Shader with more complex types for type reflection testing
const COMPLEX_TYPES_SHADER: &[u8] = b"
cbuffer ComplexBuffer : register(b0) {
    float4x4 matrix1;
    float4 vector1;
    float3 vector3;
    float2 vector2;
    float scalar;
    int intValue;
    uint uintValue;
    bool boolValue;
};

struct CustomStruct {
    float4 position;
    float3 normal;
    float2 texcoord;
};

cbuffer StructBuffer : register(b1) {
    CustomStruct myStruct;
    float4 extraData[4];
};

float4 main(float4 pos : SV_POSITION) : SV_TARGET {
    return mul(pos, matrix1) + vector1 + float4(vector3, 0) + float4(vector2, 0, 0);
}
\0";

#[test]
fn test_reflection_complex_types() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            COMPLEX_TYPES_SHADER.as_ptr() as *const _,
            COMPLEX_TYPES_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("Compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        (vtable.GetDesc)(refl, &mut desc);

        println!(
            "Complex Types Shader - Constant Buffers: {}",
            desc.ConstantBuffers
        );

        for i in 0..desc.ConstantBuffers {
            let cb = (vtable.GetConstantBufferByIndex)(refl, i);
            if cb.is_null() {
                continue;
            }

            let cb_vtable = &*(*cb).vtable;
            let mut cb_desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
            if (cb_vtable.GetDesc)(cb, &mut cb_desc) != S_OK {
                continue;
            }

            let cb_name = if !cb_desc.Name.is_null() {
                CStr::from_ptr(cb_desc.Name).to_string_lossy()
            } else {
                "<null>".into()
            };

            println!(
                "\nCB[{}]: '{}' - {} variables",
                i, cb_name, cb_desc.Variables
            );

            for j in 0..cb_desc.Variables {
                let var = (cb_vtable.GetVariableByIndex)(cb, j);
                if var.is_null() {
                    continue;
                }

                let var_vtable = &*(*var).vtable;
                let mut var_desc: D3D11_SHADER_VARIABLE_DESC = std::mem::zeroed();
                if (var_vtable.GetDesc)(var, &mut var_desc) != S_OK {
                    continue;
                }

                let var_name = if !var_desc.Name.is_null() {
                    CStr::from_ptr(var_desc.Name).to_string_lossy()
                } else {
                    "<null>".into()
                };

                let var_type = (var_vtable.GetType)(var);
                if !var_type.is_null() {
                    let type_vtable = &*(*var_type).vtable;
                    let mut type_desc: D3D11_SHADER_TYPE_DESC = std::mem::zeroed();
                    if (type_vtable.GetDesc)(var_type, &mut type_desc) == S_OK {
                        let type_name = if !type_desc.Name.is_null() {
                            CStr::from_ptr(type_desc.Name).to_string_lossy()
                        } else {
                            "<unnamed>".into()
                        };

                        println!(
                            "  Var[{}]: '{}' - Class={}, Type={}, Rows={}, Cols={}, Elements={}, TypeName='{}'",
                            j,
                            var_name,
                            type_desc.Class,
                            type_desc.Type,
                            type_desc.Rows,
                            type_desc.Columns,
                            type_desc.Elements,
                            type_name
                        );

                        // Test member access for struct types
                        if type_desc.Members > 0 {
                            println!("    Members: {}", type_desc.Members);
                            for m in 0..type_desc.Members {
                                let member_type = (type_vtable.GetMemberTypeByIndex)(var_type, m);
                                let member_name = (type_vtable.GetMemberTypeName)(var_type, m);

                                let member_name_str = if !member_name.is_null() {
                                    CStr::from_ptr(member_name).to_string_lossy()
                                } else {
                                    "<null>".into()
                                };

                                if !member_type.is_null() {
                                    let member_type_vtable = &*(*member_type).vtable;
                                    let mut member_type_desc: D3D11_SHADER_TYPE_DESC =
                                        std::mem::zeroed();
                                    if (member_type_vtable.GetDesc)(
                                        member_type,
                                        &mut member_type_desc,
                                    ) == S_OK
                                    {
                                        println!(
                                            "      Member[{}]: '{}' - Rows={}, Cols={}",
                                            m,
                                            member_name_str,
                                            member_type_desc.Rows,
                                            member_type_desc.Columns
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_is_sample_frequency_shader() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            PIXEL_SHADER.as_ptr() as *const _,
            PIXEL_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"ps_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let is_sample_freq = (vtable.IsSampleFrequencyShader)(refl);
        println!("IsSampleFrequencyShader: {}", is_sample_freq != 0);

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_num_interface_slots() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let num_slots = (vtable.GetNumInterfaceSlots)(refl);
        println!("NumInterfaceSlots: {}", num_slots);

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

#[test]
fn test_reflection_gs_input_primitive() {
    // Geometry shader for testing GS-specific reflection
    const GEOMETRY_SHADER: &[u8] = b"
struct GS_INPUT {
    float4 pos : SV_POSITION;
};

struct GS_OUTPUT {
    float4 pos : SV_POSITION;
};

[maxvertexcount(3)]
void main(triangle GS_INPUT input[3], inout TriangleStream<GS_OUTPUT> stream) {
    for (int i = 0; i < 3; i++) {
        GS_OUTPUT output;
        output.pos = input[i].pos;
        stream.Append(output);
    }
}
\0";

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            GEOMETRY_SHADER.as_ptr() as *const _,
            GEOMETRY_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"gs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            let err_msg = get_error_message(errors);
            release_blob(errors);
            panic!("GS compilation failed: {}", err_msg);
        }

        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        let input_primitive = (vtable.GetGSInputPrimitive)(refl);
        println!("GS Input Primitive: {}", input_primitive);
        // D3D_PRIMITIVE_TRIANGLE = 6
        assert!(input_primitive > 0, "Should have valid input primitive");

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}

// ============================================================================
// Variable GetBuffer Test
// ============================================================================

#[test]
fn test_variable_get_buffer() {
    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        let result = D3DCompile(
            VERTEX_SHADER.as_ptr() as *const _,
            VERTEX_SHADER.len() - 1,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            c"main".as_ptr(),
            c"vs_5_0".as_ptr(),
            0,
            0,
            &mut code,
            &mut errors,
        );

        assert_eq!(result, S_OK);
        let bytecode = get_blob_data(code);

        let mut reflector: *mut std::ffi::c_void = ptr::null_mut();
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11SHADERREFLECTION.as_ptr() as *const _,
            &mut reflector,
        );

        let refl = reflector as *mut ID3D11ShaderReflection;
        let vtable = &*(*refl).vtable;

        // Get first constant buffer
        let cb = (vtable.GetConstantBufferByIndex)(refl, 0);
        if !cb.is_null() {
            let cb_vtable = &*(*cb).vtable;

            // Get first variable
            let var = (cb_vtable.GetVariableByIndex)(cb, 0);
            if !var.is_null() {
                let var_vtable = &*(*var).vtable;

                // GetBuffer should return the parent constant buffer
                let parent_cb = (var_vtable.GetBuffer)(var);
                assert!(!parent_cb.is_null(), "GetBuffer should return parent CB");

                // Verify it's the same buffer by checking the description
                let parent_cb_vtable = &*(*parent_cb).vtable;
                let mut parent_desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
                let result = (parent_cb_vtable.GetDesc)(parent_cb, &mut parent_desc);
                if result == S_OK {
                    let name = if !parent_desc.Name.is_null() {
                        CStr::from_ptr(parent_desc.Name).to_string_lossy()
                    } else {
                        "<null>".into()
                    };
                    println!("Variable's parent buffer: '{}'", name);
                }
            }
        }

        release_reflection(refl);
        release_blob(code);
        release_blob(errors);
    }
}
