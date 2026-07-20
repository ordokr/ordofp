# Original Source Attribution

## wgpu Device Initialization

**Source Repository**: https://github.com/gfx-rs/wgpu  
**License**: MIT OR Apache-2.0  
**Pinned Commit**: `0ac2da4b6baea94c1e9d9bb03bdcf69122da857a`  
**Original File**: `examples/standalone/01_hello_compute/src/main.rs`

### Adapted Files
- `core/src/par/backend/wgpu/device.rs` - Device initialization patterns
- `core/src/par/backend/wgpu/buffer.rs` - Buffer creation and mapping patterns

### Changes
- Adapted to OrdoFP naming conventions (Scholastic Latin)
- Integrated with ParFlumen Backend trait
- Added error handling and kernel caching

## WGSL Codegen Architecture

**Source Repository**: https://github.com/tracel-ai/burn  
**License**: MIT OR Apache-2.0  
**Original Location**: `crates/burn-wgpu/src/compiler/wgsl/`

### Adapted Files
- `core/src/par/codegen/wgsl.rs` - WGSL shader generation architecture

### Changes
- Simplified for ParFlumen Nodus IR
- Adapted to Rust syntax and OrdoFP patterns
