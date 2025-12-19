use d3dcompiler::{
    D3D_SHADER_MACRO, D3D11_SHADER_BUFFER_DESC, D3D11_SHADER_DESC, D3D11_SHADER_INPUT_BIND_DESC,
    D3D11_SIGNATURE_PARAMETER_DESC, D3DCompile, D3DDisassemble, D3DPreprocess, D3DReflect,
    ID3D11ShaderReflection, ID3DBlob, S_OK,
};
use std::ffi::{CStr, CString, c_void};
use std::path::PathBuf;

// IID_ID3D11ShaderReflection = {8d536ca1-0cca-4956-a837-786963755584}
const IID_ID3D11_SHADER_REFLECTION: [u8; 16] = [
    0xa1, 0x6c, 0x53, 0x8d, 0xca, 0x0c, 0x56, 0x49, 0xa8, 0x37, 0x78, 0x69, 0x63, 0x75, 0x55, 0x84,
];

fn shader_input_type_name(t: u32) -> &'static str {
    match t {
        0 => "cbuffer",
        1 => "tbuffer",
        2 => "texture",
        3 => "sampler",
        4 => "uav_rwtyped",
        5 => "structured",
        6 => "uav_rwstructured",
        7 => "byteaddress",
        8 => "uav_rwbyteaddress",
        9 => "uav_append_structured",
        10 => "uav_consume_structured",
        11 => "uav_rwstructured_with_counter",
        _ => "unknown",
    }
}

fn component_type_name(t: u32) -> &'static str {
    match t {
        0 => "unknown",
        1 => "uint",
        2 => "int",
        3 => "float",
        _ => "unknown",
    }
}

fn mask_to_string(mask: u8) -> String {
    let mut s = String::new();
    if mask & 1 != 0 {
        s.push('x');
    }
    if mask & 2 != 0 {
        s.push('y');
    }
    if mask & 4 != 0 {
        s.push('z');
    }
    if mask & 8 != 0 {
        s.push('w');
    }
    if s.is_empty() {
        s.push_str("none");
    }
    s
}

fn print_usage() {
    eprintln!("Usage: d3dcompiler <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  compile <file.hlsl> -e <entry> -t <target> [-o <output>]");
    eprintln!("      Compile HLSL shader to bytecode");
    eprintln!("      -e, --entry     Entry point function name");
    eprintln!("      -t, --target    Shader target (e.g., vs_5_0, ps_5_0, cs_5_0)");
    eprintln!("      -o, --output    Output file (default: <input>.dxbc)");
    eprintln!("      -O, --optimize  Optimization level 0-3 (default: 1)");
    eprintln!("      -D <name=value> Define preprocessor macro");
    eprintln!();
    eprintln!("  disasm <file.dxbc> [-o <output>]");
    eprintln!("      Disassemble shader bytecode");
    eprintln!("      -o, --output    Output file (default: stdout)");
    eprintln!();
    eprintln!("  preprocess <file.hlsl> [-o <output>]");
    eprintln!("      Preprocess HLSL source");
    eprintln!("      -o, --output    Output file (default: stdout)");
    eprintln!("      -D <name=value> Define preprocessor macro");
    eprintln!();
    eprintln!("  reflect <file.dxbc>");
    eprintln!("      Show shader reflection info (inputs, outputs, resources)");
    eprintln!();
    eprintln!("Shader targets:");
    eprintln!("  vs_5_0, vs_5_1    Vertex shader");
    eprintln!("  ps_5_0, ps_5_1    Pixel shader");
    eprintln!("  cs_5_0, cs_5_1    Compute shader");
    eprintln!("  gs_5_0, gs_5_1    Geometry shader");
    eprintln!("  hs_5_0, hs_5_1    Hull shader");
    eprintln!("  ds_5_0, ds_5_1    Domain shader");
}

fn blob_to_slice<'a>(blob: *mut ID3DBlob) -> &'a [u8] {
    unsafe {
        let ptr = ((*(*blob).vtable).GetBufferPointer)(blob);
        let size = ((*(*blob).vtable).GetBufferSize)(blob);
        std::slice::from_raw_parts(ptr as *const u8, size)
    }
}

fn release_blob(blob: *mut ID3DBlob) {
    unsafe {
        ((*(*blob).vtable).Release)(blob);
    }
}

fn compile_shader(args: &[String]) -> Result<(), String> {
    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut entry_point: Option<String> = None;
    let mut target: Option<String> = None;
    let mut opt_level: u32 = 1;
    let mut defines: Vec<(String, String)> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-e" | "--entry" => {
                i += 1;
                entry_point = Some(args.get(i).ok_or("Missing entry point")?.clone());
            }
            "-t" | "--target" => {
                i += 1;
                target = Some(args.get(i).ok_or("Missing target")?.clone());
            }
            "-o" | "--output" => {
                i += 1;
                output_file = Some(PathBuf::from(args.get(i).ok_or("Missing output file")?));
            }
            "-O" | "--optimize" => {
                i += 1;
                opt_level = args
                    .get(i)
                    .ok_or("Missing optimization level")?
                    .parse()
                    .map_err(|_| "Invalid optimization level")?;
            }
            arg if arg.starts_with("-D") => {
                let define = if arg.len() > 2 {
                    &arg[2..]
                } else {
                    i += 1;
                    args.get(i).ok_or("Missing define value")?
                };
                let (name, value) = define.split_once('=').unwrap_or((define, "1"));
                defines.push((name.to_string(), value.to_string()));
            }
            arg if !arg.starts_with('-') && input_file.is_none() => {
                input_file = Some(PathBuf::from(arg));
            }
            arg => return Err(format!("Unknown argument: {}", arg)),
        }
        i += 1;
    }

    let input_file = input_file.ok_or("Missing input file")?;
    let entry_point = entry_point.ok_or("Missing entry point (-e)")?;
    let target = target.ok_or("Missing target (-t)")?;
    let output_file = output_file.unwrap_or_else(|| input_file.with_extension("dxbc"));

    let source = std::fs::read_to_string(&input_file)
        .map_err(|e| format!("Failed to read {}: {}", input_file.display(), e))?;

    let source_name = CString::new(input_file.to_string_lossy().as_bytes()).unwrap();
    let entry_point_c = CString::new(entry_point.as_bytes()).unwrap();
    let target_c = CString::new(target.as_bytes()).unwrap();

    // Build defines
    let define_cstrings: Vec<(CString, CString)> = defines
        .iter()
        .map(|(n, v)| {
            (
                CString::new(n.as_bytes()).unwrap(),
                CString::new(v.as_bytes()).unwrap(),
            )
        })
        .collect();
    let mut d3d_defines: Vec<D3D_SHADER_MACRO> = define_cstrings
        .iter()
        .map(|(n, v)| D3D_SHADER_MACRO {
            Name: n.as_ptr(),
            Definition: v.as_ptr(),
        })
        .collect();
    d3d_defines.push(D3D_SHADER_MACRO {
        Name: std::ptr::null(),
        Definition: std::ptr::null(),
    });

    // Compile flags
    let mut flags1: u32 = 0;
    // D3DCOMPILE_DEBUG = 1, D3DCOMPILE_SKIP_OPTIMIZATION = 4
    // D3DCOMPILE_OPTIMIZATION_LEVEL0 = 1 << 14, LEVEL1 = 0, LEVEL2 = 2 << 14, LEVEL3 = 1 << 15
    match opt_level {
        0 => flags1 |= (1 << 14) | 4, // LEVEL0 + SKIP_OPTIMIZATION
        1 => {}                       // LEVEL1 (default)
        2 => flags1 |= 2 << 14,       // LEVEL2
        3 => flags1 |= 1 << 15,       // LEVEL3
        _ => return Err("Optimization level must be 0-3".to_string()),
    }

    let mut code_blob: *mut ID3DBlob = std::ptr::null_mut();
    let mut error_blob: *mut ID3DBlob = std::ptr::null_mut();

    let hr = unsafe {
        D3DCompile(
            source.as_ptr() as *const _,
            source.len(),
            source_name.as_ptr(),
            d3d_defines.as_ptr(),
            std::ptr::null_mut(), // No include handler
            entry_point_c.as_ptr(),
            target_c.as_ptr(),
            flags1,
            0,
            &mut code_blob,
            &mut error_blob,
        )
    };

    if hr != S_OK {
        let error_msg = if !error_blob.is_null() {
            let slice = blob_to_slice(error_blob);
            let msg = String::from_utf8_lossy(slice).to_string();
            release_blob(error_blob);
            msg
        } else {
            format!("Unknown error (HRESULT: 0x{:08x})", hr as u32)
        };
        return Err(format!("Compilation failed:\n{}", error_msg));
    }

    // Write output
    let bytecode = blob_to_slice(code_blob);
    std::fs::write(&output_file, bytecode)
        .map_err(|e| format!("Failed to write {}: {}", output_file.display(), e))?;

    eprintln!(
        "Compiled {} -> {} ({} bytes)",
        input_file.display(),
        output_file.display(),
        bytecode.len()
    );

    release_blob(code_blob);
    if !error_blob.is_null() {
        release_blob(error_blob);
    }

    Ok(())
}

fn disassemble_shader(args: &[String]) -> Result<(), String> {
    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                output_file = Some(PathBuf::from(args.get(i).ok_or("Missing output file")?));
            }
            arg if !arg.starts_with('-') && input_file.is_none() => {
                input_file = Some(PathBuf::from(arg));
            }
            arg => return Err(format!("Unknown argument: {}", arg)),
        }
        i += 1;
    }

    let input_file = input_file.ok_or("Missing input file")?;

    let bytecode = std::fs::read(&input_file)
        .map_err(|e| format!("Failed to read {}: {}", input_file.display(), e))?;

    let mut disasm_blob: *mut ID3DBlob = std::ptr::null_mut();

    let hr = unsafe {
        D3DDisassemble(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            0,
            std::ptr::null(),
            &mut disasm_blob,
        )
    };

    if hr != S_OK {
        return Err(format!("Disassembly failed (HRESULT: 0x{:08x})", hr as u32));
    }

    let disasm = blob_to_slice(disasm_blob);
    let disasm_str = String::from_utf8_lossy(disasm);

    if let Some(output) = output_file {
        std::fs::write(&output, disasm_str.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
        eprintln!(
            "Disassembled {} -> {}",
            input_file.display(),
            output.display()
        );
    } else {
        print!("{}", disasm_str);
    }

    release_blob(disasm_blob);
    Ok(())
}

fn preprocess_shader(args: &[String]) -> Result<(), String> {
    let mut input_file: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut defines: Vec<(String, String)> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                output_file = Some(PathBuf::from(args.get(i).ok_or("Missing output file")?));
            }
            arg if arg.starts_with("-D") => {
                let define = if arg.len() > 2 {
                    &arg[2..]
                } else {
                    i += 1;
                    args.get(i).ok_or("Missing define value")?
                };
                let (name, value) = define.split_once('=').unwrap_or((define, "1"));
                defines.push((name.to_string(), value.to_string()));
            }
            arg if !arg.starts_with('-') && input_file.is_none() => {
                input_file = Some(PathBuf::from(arg));
            }
            arg => return Err(format!("Unknown argument: {}", arg)),
        }
        i += 1;
    }

    let input_file = input_file.ok_or("Missing input file")?;

    let source = std::fs::read_to_string(&input_file)
        .map_err(|e| format!("Failed to read {}: {}", input_file.display(), e))?;

    let source_name = CString::new(input_file.to_string_lossy().as_bytes()).unwrap();

    // Build defines
    let define_cstrings: Vec<(CString, CString)> = defines
        .iter()
        .map(|(n, v)| {
            (
                CString::new(n.as_bytes()).unwrap(),
                CString::new(v.as_bytes()).unwrap(),
            )
        })
        .collect();
    let mut d3d_defines: Vec<D3D_SHADER_MACRO> = define_cstrings
        .iter()
        .map(|(n, v)| D3D_SHADER_MACRO {
            Name: n.as_ptr(),
            Definition: v.as_ptr(),
        })
        .collect();
    d3d_defines.push(D3D_SHADER_MACRO {
        Name: std::ptr::null(),
        Definition: std::ptr::null(),
    });

    let mut code_blob: *mut ID3DBlob = std::ptr::null_mut();
    let mut error_blob: *mut ID3DBlob = std::ptr::null_mut();

    let hr = unsafe {
        D3DPreprocess(
            source.as_ptr() as *const _,
            source.len(),
            source_name.as_ptr(),
            d3d_defines.as_ptr(),
            std::ptr::null_mut(),
            &mut code_blob,
            &mut error_blob,
        )
    };

    if hr != S_OK {
        let error_msg = if !error_blob.is_null() {
            let slice = blob_to_slice(error_blob);
            let msg = String::from_utf8_lossy(slice).to_string();
            release_blob(error_blob);
            msg
        } else {
            format!("Unknown error (HRESULT: 0x{:08x})", hr as u32)
        };
        return Err(format!("Preprocessing failed:\n{}", error_msg));
    }

    let preprocessed = blob_to_slice(code_blob);
    let preprocessed_str = String::from_utf8_lossy(preprocessed);

    if let Some(output) = output_file {
        std::fs::write(&output, preprocessed_str.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
        eprintln!(
            "Preprocessed {} -> {}",
            input_file.display(),
            output.display()
        );
    } else {
        print!("{}", preprocessed_str);
    }

    release_blob(code_blob);
    if !error_blob.is_null() {
        release_blob(error_blob);
    }

    Ok(())
}

fn reflect_shader(args: &[String]) -> Result<(), String> {
    let input_file = args.first().ok_or("Missing input file")?;
    let input_file = PathBuf::from(input_file);

    let bytecode = std::fs::read(&input_file)
        .map_err(|e| format!("Failed to read {}: {}", input_file.display(), e))?;

    let mut reflector: *mut ID3D11ShaderReflection = std::ptr::null_mut();

    let hr = unsafe {
        D3DReflect(
            bytecode.as_ptr() as *const _,
            bytecode.len(),
            IID_ID3D11_SHADER_REFLECTION.as_ptr() as *const _,
            &mut reflector as *mut _ as *mut *mut c_void,
        )
    };

    if hr != S_OK {
        return Err(format!("D3DReflect failed (HRESULT: 0x{:08x})", hr as u32));
    }

    unsafe {
        let vtable = &*(*reflector).vtable;

        // Get shader description
        let mut desc: D3D11_SHADER_DESC = std::mem::zeroed();
        let hr = (vtable.GetDesc)(reflector, &mut desc);
        if hr != S_OK {
            return Err(format!("GetDesc failed (HRESULT: 0x{:08x})", hr as u32));
        }

        // Parse version
        let shader_type = match (desc.Version >> 16) & 0xFFFF {
            0xFFFE => "vs",
            0xFFFF => "ps",
            0x4753 => "gs",
            0x4853 => "hs",
            0x4453 => "ds",
            0x4353 => "cs",
            _ => "unknown",
        };
        let major = (desc.Version >> 4) & 0xF;
        let minor = desc.Version & 0xF;

        println!("Shader: {}_{}.{}", shader_type, major, minor);
        if !desc.Creator.is_null() {
            println!(
                "Creator: {}",
                CStr::from_ptr(desc.Creator).to_string_lossy()
            );
        }
        println!("Instructions: {}", desc.InstructionCount);
        println!("Temp registers: {}", desc.TempRegisterCount);
        println!();

        // Input parameters
        if desc.InputParameters > 0 {
            println!("Input Parameters ({}):", desc.InputParameters);
            for i in 0..desc.InputParameters {
                let mut param: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
                if (vtable.GetInputParameterDesc)(reflector, i, &mut param) == S_OK {
                    let name = if !param.SemanticName.is_null() {
                        CStr::from_ptr(param.SemanticName).to_string_lossy()
                    } else {
                        "???".into()
                    };
                    println!(
                        "  [{:2}] {}{}: {} {} (mask: {})",
                        param.Register,
                        name,
                        if param.SemanticIndex > 0 {
                            format!("{}", param.SemanticIndex)
                        } else {
                            String::new()
                        },
                        component_type_name(param.ComponentType),
                        mask_to_string(param.Mask),
                        mask_to_string(param.ReadWriteMask)
                    );
                }
            }
            println!();
        }

        // Output parameters
        if desc.OutputParameters > 0 {
            println!("Output Parameters ({}):", desc.OutputParameters);
            for i in 0..desc.OutputParameters {
                let mut param: D3D11_SIGNATURE_PARAMETER_DESC = std::mem::zeroed();
                if (vtable.GetOutputParameterDesc)(reflector, i, &mut param) == S_OK {
                    let name = if !param.SemanticName.is_null() {
                        CStr::from_ptr(param.SemanticName).to_string_lossy()
                    } else {
                        "???".into()
                    };
                    println!(
                        "  [{:2}] {}{}: {} {} (mask: {})",
                        param.Register,
                        name,
                        if param.SemanticIndex > 0 {
                            format!("{}", param.SemanticIndex)
                        } else {
                            String::new()
                        },
                        component_type_name(param.ComponentType),
                        mask_to_string(param.Mask),
                        mask_to_string(param.ReadWriteMask)
                    );
                }
            }
            println!();
        }

        // Bound resources
        if desc.BoundResources > 0 {
            println!("Bound Resources ({}):", desc.BoundResources);
            for i in 0..desc.BoundResources {
                let mut bind_desc: D3D11_SHADER_INPUT_BIND_DESC = std::mem::zeroed();
                if (vtable.GetResourceBindingDesc)(reflector, i, &mut bind_desc) == S_OK {
                    let name = if !bind_desc.Name.is_null() {
                        CStr::from_ptr(bind_desc.Name).to_string_lossy()
                    } else {
                        "???".into()
                    };
                    println!(
                        "  [{}:{}] {} ({})",
                        shader_input_type_name(bind_desc.Type)
                            .chars()
                            .next()
                            .unwrap_or('?'),
                        bind_desc.BindPoint,
                        name,
                        shader_input_type_name(bind_desc.Type)
                    );
                }
            }
            println!();
        }

        // Constant buffers
        if desc.ConstantBuffers > 0 {
            println!("Constant Buffers ({}):", desc.ConstantBuffers);
            for i in 0..desc.ConstantBuffers {
                let cb = (vtable.GetConstantBufferByIndex)(reflector, i);
                if !cb.is_null() {
                    let cb_vtable = &*(*cb).vtable;
                    let mut cb_desc: D3D11_SHADER_BUFFER_DESC = std::mem::zeroed();
                    if (cb_vtable.GetDesc)(cb, &mut cb_desc) == S_OK {
                        let name = if !cb_desc.Name.is_null() {
                            CStr::from_ptr(cb_desc.Name).to_string_lossy()
                        } else {
                            "???".into()
                        };
                        println!(
                            "  [{}] {} ({} bytes, {} variables)",
                            i, name, cb_desc.Size, cb_desc.Variables
                        );
                    }
                }
            }
            println!();
        }

        // Release
        (vtable.Release)(reflector);
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let command_args = &args[2..];

    let result = match command.as_str() {
        "compile" => compile_shader(command_args),
        "disasm" | "disassemble" => disassemble_shader(command_args),
        "preprocess" | "pp" => preprocess_shader(command_args),
        "reflect" => reflect_shader(command_args),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
