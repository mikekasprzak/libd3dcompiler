# libd3dcompiler

Wraps `d3dcompiler_47.dll` with shims to run it as if it were a native Linux shared library. Primary use case for me is enabling cooking SM5/FXC shaders for Windows from Linux editor. Also includes Rust bindings and CLI for compiling to and inspecting DXBC shader blobs.

## Usage

Requires `d3dcompiler_47.dll` be *obtained* and dropped in repo root.

### CLI

```bash
d3dcrs compile shader.hlsl -e main -t ps_5_0 -o shader.dxbc
d3dcrs disasm shader.dxbc
d3dcrs reflect shader.dxbc
d3dcrs strip shader.dxbc -o stripped.dxbc
```

## Unreal Engine patch (4.27)

[unreal-4.27-cross-cook-content-windows-linux.patch](assets/unreal-4.27-cross-cook-content-windows-linux.patch)

![Cross-cook content for Windows](assets/cross-cook-content-for-windows.png)

### Setup

1. Build library
   ```bash
   cargo build --release
   ```

2. Apply the engine patch
   ```bash
   cd /path/to/UnrealEngine
   git apply /path/to/libd3dcompiler/assets/unreal-4.27-cross-cook-content-windows-linux.patch
   ```

3. Install libd3dcompiler.so
   ```bash
   mkdir -p Engine/Binaries/ThirdParty/D3DCompiler/Linux
   cp /path/to/libd3dcompiler/target/release/libd3dcompiler.so \
      Engine/Binaries/ThirdParty/D3DCompiler/Linux/
   ```

4. Rebuild engine
   ```bash
   ./Setup.sh && ./GenerateProjectFiles.sh && make ShaderCompileWorker UnrealEditor
   ```
