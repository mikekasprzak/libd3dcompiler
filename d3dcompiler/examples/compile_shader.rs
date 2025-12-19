//! Example: Compile an HLSL shader to DXBC bytecode
//!
//! Run with: cargo run --example compile_shader
//!
//! Make sure d3dcompiler_47.dll is in the project root or next to the binary.

use d3dcompiler::*;
use std::ptr;

const SHADER_SOURCE: &str = r#"
// Simple pixel shader that outputs a gradient based on UV coordinates
struct PS_INPUT {
    float4 pos : SV_POSITION;
    float2 uv : TEXCOORD0;
};

cbuffer Constants : register(b0) {
    float4 tint;
    float time;
};

float4 main(PS_INPUT input) : SV_TARGET {
    // Create animated gradient
    float2 uv = input.uv;
    float r = sin(uv.x * 3.14159 + time) * 0.5 + 0.5;
    float g = cos(uv.y * 3.14159 + time * 0.7) * 0.5 + 0.5;
    float b = sin((uv.x + uv.y) * 3.14159 + time * 1.3) * 0.5 + 0.5;

    return float4(r, g, b, 1.0) * tint;
}
"#;

fn main() {
    println!("D3D Shader Compiler Example");
    println!("===========================\n");

    // Prepare shader source
    let source = format!("{}\0", SHADER_SOURCE);
    let source_name = b"example.hlsl\0";
    let entry_point = b"main\0";
    let target = b"ps_5_0\0";

    println!("Shader source ({} bytes):", source.len());
    println!("----------------------------------------");
    for (i, line) in SHADER_SOURCE.lines().enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("----------------------------------------\n");

    unsafe {
        let mut code: *mut ID3DBlob = ptr::null_mut();
        let mut errors: *mut ID3DBlob = ptr::null_mut();

        println!("Compiling shader...");
        println!("  Entry point: main");
        println!("  Target: ps_5_0 (Pixel Shader, Shader Model 5.0)\n");

        let result = D3DCompile(
            source.as_ptr() as *const _,
            source.len() - 1,
            source_name.as_ptr() as *const _,
            ptr::null(),
            ptr::null_mut(),
            entry_point.as_ptr() as *const _,
            target.as_ptr() as *const _,
            0, // Flags1
            0, // Flags2
            &mut code,
            &mut errors,
        );

        if result != S_OK {
            eprintln!("Compilation FAILED (HRESULT: 0x{:08x})", result as u32);

            if !errors.is_null() {
                let vtable = &*(*errors).vtable;
                let ptr = (vtable.GetBufferPointer)(errors);
                let size = (vtable.GetBufferSize)(errors);
                let error_msg = std::slice::from_raw_parts(ptr as *const u8, size);
                eprintln!("\nError message:");
                eprintln!("{}", String::from_utf8_lossy(error_msg));
                (vtable.Release)(errors);
            }
            std::process::exit(1);
        }

        println!("Compilation SUCCEEDED!\n");

        // Get bytecode info
        let vtable = &*(*code).vtable;
        let bytecode_ptr = (vtable.GetBufferPointer)(code);
        let bytecode_size = (vtable.GetBufferSize)(code);
        let bytecode = std::slice::from_raw_parts(bytecode_ptr as *const u8, bytecode_size);

        println!("Bytecode size: {} bytes", bytecode_size);
        println!(
            "Magic: {:?}",
            std::str::from_utf8(&bytecode[0..4]).unwrap_or("????")
        );

        // Print hex dump of first 64 bytes
        println!("\nBytecode header (first 64 bytes):");
        for (i, chunk) in bytecode
            .iter()
            .take(64)
            .collect::<Vec<_>>()
            .chunks(16)
            .enumerate()
        {
            print!("{:04x}: ", i * 16);
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            print!("  ");
            for byte in chunk {
                let c = **byte as char;
                print!("{}", if c.is_ascii_graphic() { c } else { '.' });
            }
            println!();
        }

        // Try to disassemble
        println!("\n\nDisassembling...");
        let mut disasm: *mut ID3DBlob = ptr::null_mut();

        let result = D3DDisassemble(bytecode_ptr, bytecode_size, 0, ptr::null(), &mut disasm);

        if result == S_OK && !disasm.is_null() {
            let vtable = &*(*disasm).vtable;
            let ptr = (vtable.GetBufferPointer)(disasm);
            let size = (vtable.GetBufferSize)(disasm);
            let asm = std::slice::from_raw_parts(ptr as *const u8, size);
            let asm_str = String::from_utf8_lossy(asm);

            println!("\nDisassembly:");
            println!("----------------------------------------");
            // Print first 50 lines
            for (i, line) in asm_str.lines().take(50).enumerate() {
                println!("{:3}: {}", i + 1, line);
            }
            if asm_str.lines().count() > 50 {
                println!("... ({} more lines)", asm_str.lines().count() - 50);
            }
            println!("----------------------------------------");

            (vtable.Release)(disasm);
        } else {
            println!("Disassembly not available");
        }

        // Cleanup
        (vtable.Release)(code);
        if !errors.is_null() {
            let vtable = &*(*errors).vtable;
            (vtable.Release)(errors);
        }
    }

    println!("\nDone!");
}
