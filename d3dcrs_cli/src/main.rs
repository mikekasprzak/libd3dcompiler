//! D3DCompiler CLI tool using safe Rust API

use clap::{Parser, Subcommand, ValueEnum};
use d3dcrs::{
    BlobPart, CompileBuilder, CompileFlags, DisassembleBuilder, PreprocessBuilder,
    ShaderReflection, ShaderTarget, StripFlags, get_blob_part, get_debug_info, get_input_signature,
    get_output_signature, set_blob_part, strip_shader,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "d3dcrs")]
#[command(about = "D3DCompiler command-line tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile HLSL shader to bytecode
    Compile {
        /// Input HLSL file
        input: PathBuf,

        /// Entry point function name
        #[arg(short, long)]
        entry: String,

        /// Shader target (e.g., vs_5_0, ps_5_0)
        #[arg(short, long, value_enum)]
        target: Target,

        /// Output file (default: <input>.dxbc)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level 0-3
        #[arg(short = 'O', long, default_value = "1", value_parser = clap::value_parser!(u8).range(0..=3))]
        optimize: u8,

        /// Preprocessor defines (NAME=VALUE or NAME)
        #[arg(short = 'D', long = "define", value_name = "NAME=VALUE")]
        defines: Vec<String>,
    },

    /// Disassemble shader bytecode
    #[command(alias = "disassemble")]
    Disasm {
        /// Input DXBC file
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Preprocess HLSL source
    #[command(alias = "pp")]
    Preprocess {
        /// Input HLSL file
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Preprocessor defines (NAME=VALUE or NAME)
        #[arg(short = 'D', long = "define", value_name = "NAME=VALUE")]
        defines: Vec<String>,
    },

    /// Show shader reflection info
    Reflect {
        /// Input DXBC file
        input: PathBuf,
    },

    /// Strip debug info and/or reflection data from shader
    Strip {
        /// Input DXBC file
        input: PathBuf,

        /// Output file (default: <input>.stripped.dxbc)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Keep debug information (stripped by default)
        #[arg(long)]
        keep_debug: bool,

        /// Keep reflection data (stripped by default)
        #[arg(long)]
        keep_reflection: bool,

        /// Also strip test blobs
        #[arg(long)]
        test_blobs: bool,
    },

    /// Extract a part from shader blob
    Extract {
        /// Input DXBC file
        input: PathBuf,

        /// Output file (default: stdout for signatures, required for binary)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Part to extract
        #[arg(short, long, value_enum)]
        part: ExtractPart,
    },

    /// Inject private data into shader blob
    Inject {
        /// Input DXBC file
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Private data file to inject
        #[arg(short, long)]
        data: PathBuf,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Target {
    // Vertex shaders
    #[value(name = "vs_4_0")]
    Vs40,
    #[value(name = "vs_4_1")]
    Vs41,
    #[value(name = "vs_5_0")]
    Vs50,
    #[value(name = "vs_5_1")]
    Vs51,
    // Pixel shaders
    #[value(name = "ps_4_0")]
    Ps40,
    #[value(name = "ps_4_1")]
    Ps41,
    #[value(name = "ps_5_0")]
    Ps50,
    #[value(name = "ps_5_1")]
    Ps51,
    // Geometry shaders
    #[value(name = "gs_4_0")]
    Gs40,
    #[value(name = "gs_4_1")]
    Gs41,
    #[value(name = "gs_5_0")]
    Gs50,
    #[value(name = "gs_5_1")]
    Gs51,
    // Compute shaders
    #[value(name = "cs_4_0")]
    Cs40,
    #[value(name = "cs_4_1")]
    Cs41,
    #[value(name = "cs_5_0")]
    Cs50,
    #[value(name = "cs_5_1")]
    Cs51,
    // Hull shaders
    #[value(name = "hs_5_0")]
    Hs50,
    #[value(name = "hs_5_1")]
    Hs51,
    // Domain shaders
    #[value(name = "ds_5_0")]
    Ds50,
    #[value(name = "ds_5_1")]
    Ds51,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ExtractPart {
    /// Input signature
    #[value(name = "input-sig")]
    InputSignature,
    /// Output signature
    #[value(name = "output-sig")]
    OutputSignature,
    /// Debug info
    #[value(name = "debug")]
    DebugInfo,
    /// Shader bytecode (SHEX/SHDR)
    #[value(name = "bytecode")]
    Bytecode,
    /// All blob parts info (list only)
    #[value(name = "list")]
    List,
}

impl From<Target> for ShaderTarget {
    fn from(t: Target) -> Self {
        match t {
            Target::Vs40 => ShaderTarget::VS_4_0,
            Target::Vs41 => ShaderTarget::VS_4_1,
            Target::Vs50 => ShaderTarget::VS_5_0,
            Target::Vs51 => ShaderTarget::VS_5_1,
            Target::Ps40 => ShaderTarget::PS_4_0,
            Target::Ps41 => ShaderTarget::PS_4_1,
            Target::Ps50 => ShaderTarget::PS_5_0,
            Target::Ps51 => ShaderTarget::PS_5_1,
            Target::Gs40 => ShaderTarget::GS_4_0,
            Target::Gs41 => ShaderTarget::GS_4_1,
            Target::Gs50 => ShaderTarget::GS_5_0,
            Target::Gs51 => ShaderTarget::GS_5_1,
            Target::Cs40 => ShaderTarget::CS_4_0,
            Target::Cs41 => ShaderTarget::CS_4_1,
            Target::Cs50 => ShaderTarget::CS_5_0,
            Target::Cs51 => ShaderTarget::CS_5_1,
            Target::Hs50 => ShaderTarget::HS_5_0,
            Target::Hs51 => ShaderTarget::HS_5_1,
            Target::Ds50 => ShaderTarget::DS_5_0,
            Target::Ds51 => ShaderTarget::DS_5_1,
        }
    }
}

fn parse_define(s: &str) -> (String, String) {
    s.split_once('=')
        .map(|(n, v)| (n.to_string(), v.to_string()))
        .unwrap_or_else(|| (s.to_string(), "1".to_string()))
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

fn compile_shader(
    input: PathBuf,
    entry: String,
    target: Target,
    output: Option<PathBuf>,
    optimize: u8,
    defines: Vec<String>,
) -> Result<(), String> {
    let output = output.unwrap_or_else(|| input.with_extension("dxbc"));

    let source = std::fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let flags = match optimize {
        0 => CompileFlags::OPTIMIZATION_LEVEL0 | CompileFlags::SKIP_OPTIMIZATION,
        1 => CompileFlags::OPTIMIZATION_LEVEL1,
        2 => CompileFlags::OPTIMIZATION_LEVEL2,
        3 => CompileFlags::OPTIMIZATION_LEVEL3,
        _ => unreachable!(),
    };

    let mut builder = CompileBuilder::new(&source, &entry, target.into())
        .source_name(&input.to_string_lossy())
        .flags(flags);

    for def in &defines {
        let (name, value) = parse_define(def);
        builder = builder.define(&name, &value);
    }

    let result = builder.compile().map_err(|e| format!("{}", e))?;

    let bytecode = result.bytecode.as_bytes();
    std::fs::write(&output, bytecode)
        .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;

    eprintln!(
        "Compiled {} -> {} ({} bytes)",
        input.display(),
        output.display(),
        bytecode.len()
    );

    if let Some(warnings) = result.warnings {
        eprintln!("Warnings:\n{}", warnings);
    }

    Ok(())
}

fn disassemble_shader(input: PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    let bytecode =
        std::fs::read(&input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let disasm = DisassembleBuilder::new(&bytecode)
        .disassemble()
        .map_err(|e| format!("{}", e))?;

    let disasm_str = disasm.to_string_lossy();

    if let Some(output) = output {
        std::fs::write(&output, disasm_str.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
        eprintln!("Disassembled {} -> {}", input.display(), output.display());
    } else {
        print!("{}", disasm_str);
    }

    Ok(())
}

fn preprocess_shader(
    input: PathBuf,
    output: Option<PathBuf>,
    defines: Vec<String>,
) -> Result<(), String> {
    let source = std::fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let mut builder = PreprocessBuilder::new(&source).source_name(&input.to_string_lossy());

    for def in &defines {
        let (name, value) = parse_define(def);
        builder = builder.define(&name, &value);
    }

    let result = builder.preprocess().map_err(|e| format!("{}", e))?;
    let preprocessed_str = result.source.to_string_lossy();

    if let Some(output) = output {
        std::fs::write(&output, preprocessed_str.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
        eprintln!("Preprocessed {} -> {}", input.display(), output.display());
    } else {
        print!("{}", preprocessed_str);
    }

    if let Some(warnings) = result.warnings {
        eprintln!("Warnings:\n{}", warnings);
    }

    Ok(())
}

fn reflect_shader(input: PathBuf) -> Result<(), String> {
    let bytecode =
        std::fs::read(&input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let reflection =
        ShaderReflection::new(&bytecode).map_err(|e| format!("Reflection failed: {}", e))?;

    let desc = reflection
        .desc()
        .map_err(|e| format!("Failed to get shader desc: {}", e))?;

    let shader_type = match (desc.version >> 16) & 0xFFFF {
        0xFFFE => "vs",
        0xFFFF => "ps",
        0x4753 => "gs",
        0x4853 => "hs",
        0x4453 => "ds",
        0x4353 => "cs",
        _ => "unknown",
    };
    let major = (desc.version >> 4) & 0xF;
    let minor = desc.version & 0xF;

    println!("Shader: {}_{}.{}", shader_type, major, minor);
    println!("Creator: {}", desc.creator);
    println!("Instructions: {}", desc.instruction_count);
    println!("Temp registers: {}", desc.temp_register_count);
    println!();

    // Input parameters
    let inputs: Vec<_> = reflection.input_parameters().collect();
    if !inputs.is_empty() {
        println!("Input Parameters ({}):", inputs.len());
        for param in inputs {
            let semantic_index = if param.semantic_index > 0 {
                format!("{}", param.semantic_index)
            } else {
                String::new()
            };
            println!(
                "  [{:2}] {}{}: {:?} {} (mask: {})",
                param.register,
                param.semantic_name,
                semantic_index,
                param.component_type,
                mask_to_string(param.mask),
                mask_to_string(param.read_write_mask)
            );
        }
        println!();
    }

    // Output parameters
    let outputs: Vec<_> = reflection.output_parameters().collect();
    if !outputs.is_empty() {
        println!("Output Parameters ({}):", outputs.len());
        for param in outputs {
            let semantic_index = if param.semantic_index > 0 {
                format!("{}", param.semantic_index)
            } else {
                String::new()
            };
            println!(
                "  [{:2}] {}{}: {:?} {} (mask: {})",
                param.register,
                param.semantic_name,
                semantic_index,
                param.component_type,
                mask_to_string(param.mask),
                mask_to_string(param.read_write_mask)
            );
        }
        println!();
    }

    // Bound resources
    let bindings: Vec<_> = reflection.resource_bindings().collect();
    if !bindings.is_empty() {
        println!("Bound Resources ({}):", bindings.len());
        for binding in bindings {
            let type_char = format!("{:?}", binding.resource_type)
                .chars()
                .next()
                .unwrap_or('?');
            println!(
                "  [{}:{}] {} ({:?})",
                type_char, binding.bind_point, binding.name, binding.resource_type
            );
        }
        println!();
    }

    // Constant buffers
    let cbs: Vec<_> = reflection.constant_buffers().collect();
    if !cbs.is_empty() {
        println!("Constant Buffers ({}):", cbs.len());
        for (i, cb) in cbs.iter().enumerate() {
            if let Ok(cb_desc) = cb.desc() {
                println!(
                    "  [{}] {} ({} bytes, {} variables)",
                    i, cb_desc.name, cb_desc.size, cb_desc.variables
                );

                for var in cb.variables() {
                    if let Ok(var_desc) = var.desc() {
                        println!(
                            "      +{:3}: {} ({} bytes)",
                            var_desc.start_offset, var_desc.name, var_desc.size
                        );
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}

fn strip_shader_cmd(
    input: PathBuf,
    output: Option<PathBuf>,
    keep_debug: bool,
    keep_reflection: bool,
    test_blobs: bool,
) -> Result<(), String> {
    let output = output.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        input.with_file_name(format!("{}.stripped.dxbc", stem))
    });

    let bytecode =
        std::fs::read(&input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let mut flags = StripFlags::empty();
    if !keep_debug {
        flags |= StripFlags::DEBUG_INFO;
    }
    if !keep_reflection {
        flags |= StripFlags::REFLECTION_DATA;
    }
    if test_blobs {
        flags |= StripFlags::TEST_BLOBS;
    }

    if flags.is_empty() {
        return Err("Nothing to strip (--keep-debug and --keep-reflection both specified)".into());
    }

    let stripped = strip_shader(&bytecode, flags).map_err(|e| format!("{}", e))?;

    let original_size = bytecode.len();
    let stripped_size = stripped.len();

    std::fs::write(&output, stripped.as_bytes())
        .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;

    eprintln!(
        "Stripped {} -> {} ({} -> {} bytes, saved {})",
        input.display(),
        output.display(),
        original_size,
        stripped_size,
        original_size - stripped_size
    );

    Ok(())
}

fn extract_part(input: PathBuf, output: Option<PathBuf>, part: ExtractPart) -> Result<(), String> {
    let bytecode =
        std::fs::read(&input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    match part {
        ExtractPart::InputSignature => {
            let sig = get_input_signature(&bytecode).map_err(|e| format!("{}", e))?;
            write_blob_output(&sig, output, "input signature")
        }
        ExtractPart::OutputSignature => {
            let sig = get_output_signature(&bytecode).map_err(|e| format!("{}", e))?;
            write_blob_output(&sig, output, "output signature")
        }
        ExtractPart::DebugInfo => {
            let debug = get_debug_info(&bytecode).map_err(|e| format!("{}", e))?;
            write_blob_output(&debug, output, "debug info")
        }
        ExtractPart::Bytecode => {
            let code =
                get_blob_part(&bytecode, BlobPart::LegacyShader).map_err(|e| format!("{}", e))?;
            write_blob_output(&code, output, "bytecode")
        }
        ExtractPart::List => {
            println!("Blob parts in {}:", input.display());
            println!("  Size: {} bytes", bytecode.len());

            // Try to extract each part and report
            let parts = [
                ("Input Signature", BlobPart::InputSignature),
                ("Output Signature", BlobPart::OutputSignature),
                ("Patch Constant Signature", BlobPart::PatchConstantSignature),
                ("All Signatures", BlobPart::AllSignatures),
                ("Debug Info", BlobPart::DebugInfo),
                ("Private Data", BlobPart::PrivateData),
                ("Root Signature", BlobPart::RootSignature),
                ("Debug Name", BlobPart::DebugName),
            ];

            for (name, part) in parts {
                if let Ok(blob) = get_blob_part(&bytecode, part) {
                    println!("  {}: {} bytes", name, blob.len())
                }
            }

            Ok(())
        }
    }
}

fn write_blob_output(
    blob: &d3dcrs::Blob,
    output: Option<PathBuf>,
    name: &str,
) -> Result<(), String> {
    if let Some(output) = output {
        std::fs::write(&output, blob.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
        eprintln!(
            "Extracted {} -> {} ({} bytes)",
            name,
            output.display(),
            blob.len()
        );
    } else {
        // Write to stdout as hex dump for binary data
        use std::io::Write;
        std::io::stdout()
            .write_all(blob.as_bytes())
            .map_err(|e| format!("Failed to write to stdout: {}", e))?;
    }
    Ok(())
}

fn inject_private_data(input: PathBuf, output: PathBuf, data: PathBuf) -> Result<(), String> {
    let bytecode =
        std::fs::read(&input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    let private_data =
        std::fs::read(&data).map_err(|e| format!("Failed to read {}: {}", data.display(), e))?;

    let result = set_blob_part(&bytecode, BlobPart::PrivateData, &private_data)
        .map_err(|e| format!("{}", e))?;

    std::fs::write(&output, result.as_bytes())
        .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;

    eprintln!(
        "Injected {} bytes from {} into {} -> {}",
        private_data.len(),
        data.display(),
        input.display(),
        output.display()
    );

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compile {
            input,
            entry,
            target,
            output,
            optimize,
            defines,
        } => compile_shader(input, entry, target, output, optimize, defines),
        Commands::Disasm { input, output } => disassemble_shader(input, output),
        Commands::Preprocess {
            input,
            output,
            defines,
        } => preprocess_shader(input, output, defines),
        Commands::Reflect { input } => reflect_shader(input),
        Commands::Strip {
            input,
            output,
            keep_debug,
            keep_reflection,
            test_blobs,
        } => strip_shader_cmd(input, output, keep_debug, keep_reflection, test_blobs),
        Commands::Extract {
            input,
            output,
            part,
        } => extract_part(input, output, part),
        Commands::Inject {
            input,
            output,
            data,
        } => inject_private_data(input, output, data),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
