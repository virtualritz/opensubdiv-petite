# `CLAUDE.md` -- opensubdiv-petite -- Rust bindings for Pixar's OpenSubdiv

## The Golden Rule

When unsure about implementation details, ALWAYS ask the developer.

## Project Context

This is a Rust wrapper for Pixar's OpenSubdiv library, providing safe bindings for high-performance subdivision surface evaluation. The project consists of two crates: `opensubdiv-petite-sys` (low-level FFI bindings) and `opensubdiv-petite` (high-level safe Rust API).

## Critical Architecture Decisions

### Two-Crate Structure

- **`opensubdiv-petite-sys`** -- Raw FFI bindings that build OpenSubdiv from source.
- **`opensubdiv-petite`** -- Safe, idiomatic Rust wrapper with selective API coverage.

### Selective API Coverage

This is a "petite" wrapper -- it doesn't expose the entire OpenSubdiv API. Focus on the most commonly used functionality for subdivision surface evaluation.

### Use External Crates Where Possible

- Always search crates.io for appropriate functionality before implementing from scratch.
- Current dependencies include `derive_more`, `num_enum`, and optional deps like `bevy`, `ultraviolet`.
- When in doubt, ALWAYS ask the developer.

### Core Components

#### Far (Feature Adaptive Representation)

- **[`TopologyRefiner`]** -- Main topology refinement interface.
- **[`TopologyDescriptor`]** -- Input mesh description.
- **[`TopologyLevel`]** -- Access to refined topology at specific levels.
- **[`PrimvarRefiner`]** -- Interpolates primitive variable data.
- **[`StencilTable`]** -- Efficient evaluation of refined positions.

#### OSD (OpenSubdiv Draw)

- **[`CpuVertexBuffer`]** -- CPU-side vertex data storage.
- **[`CpuEvaluator`]** -- CPU-based stencil evaluation.
- **[`CudaVertexBuffer`]** -- CUDA GPU vertex storage (feature-gated).
- **[`CudaEvaluator`]** -- CUDA GPU evaluation (feature-gated).

### Key Features

- Safe Rust API over C++ template-heavy code.
- Feature gates for optional GPU backends (CUDA, Metal, OpenCL).
- Topology validation enabled by default (disable for release builds).
- Index types use `usize`/`u32` instead of signed integers.

### Performance Requirements

- Zero-cost abstractions over FFI calls.
- Minimize allocations in hot paths.
- Support parallel evaluation via GPU backends.
- Preserve OpenSubdiv's optimized evaluation kernels.

### Design Patterns

- Use Rust's type system to enforce safety guarantees.
- Init struct pattern for configuration options.
- Builder pattern where init structs don't fit.
- Collapse C++ factory classes into `new()` methods.
- Use `Result<T, Error>` for fallible operations.

## Feature Flags

- `topology_validation` -- Enable expensive validation (default on).
- `cuda` -- NVIDIA CUDA backend support.
- `metal` -- Apple Metal backend (TODO).
- `opencl` -- OpenCL backend (TODO).
- `ptex` -- PTex texture support (TODO).
- `tri_mesh_buffers` -- Integration with mesh buffer types.

## External Crate Integration

- Compatibility with `bevy` for game engine integration.
- Optional `ultraviolet` for math types.
- Future compatibility targets: other graphics/game frameworks.

## Traits

- All public types must implement `Debug` and `Clone`.
- Implement `Copy` where trivially possible.
- Use `derive_more` for error types.

## Code Style and Patterns

### Guidelines:

#### Clippy And Rustfmt

- Use `cargo clippy` to check for lint warnings before committing. Make sure there are no outstanding warnings.
- Use `cargo fmt` to format code before committing. Make sure there are no formatting issues.

#### Anchor Comments

Add specially formatted comments for important implementation details.

- Use `AIDEV-NOTE:`, `AIDEV-TODO:`, or `AIDEV-QUESTION:` for AI/developer comments.
- **Important:** Always grep for existing `AIDEV-*` anchors before modifying code.
- **Update relevant anchors** when changing associated code.
- **Do not remove `AIDEV-NOTE`s** without explicit instruction.
- Add anchors for complex, important, confusing, or potentially buggy code.

### Rust Idioms

- DO NOT change public API without presenting a change proposal first.
- Write idiomatic Rust -- avoid C++/C patterns.
- PREFER functional style (map/collect) over imperative loops.
- USE type system to express constraints (unsigned for counts/indices).
- AVOID unnecessary allocations, conversions, copies.
- AVOID `unsafe` except at FFI boundaries.
- AVOID return statements -- use expression-oriented style.
- Prefer stack allocation, use `SmallVec` where sensible.

### Naming Conventions

- Follow Rust API guidelines for naming.
- Be consistently verbose (no surprising abbreviations).
- Use canonical Rust names: `len()` not `numStencils()`.
- Use `vertices_len()` not `num_vertices()`.
- Unsigned types for counts/sizes: `usize`, `u32`.

## FFI Safety Rules

1. **Always validate indices** before passing to C++.
2. **Check null pointers** from C++ before dereferencing.
3. **Ensure correct lifetime** for borrowed data.
4. **Document safety invariants** for each `unsafe` block.
5. **Validate array lengths** match expected sizes.

## What AI Must NEVER Do

1. **Never remove safety checks** -- They prevent segfaults.
2. **Never change FFI signatures** -- Breaks ABI compatibility.
3. **Never expose raw pointers** in public API.
4. **Never skip bounds checking** in topology validation mode.
5. **Never assume C++ exceptions** are handled (they abort).
6. **Never remove `AIDEV-` comments** without permission.

## Project Overview

OpenSubdiv-petite provides Rust bindings to efficiently evaluate subdivision surfaces using Pixar's battle-tested OpenSubdiv library. The wrapper focuses on the most commonly used APIs while providing a safe, idiomatic Rust interface.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a specific example
cargo run --example far_tutorial_0

# Run CUDA example (requires CUDA feature)
cargo run --example osd_tutorial_0_cuda --features cuda

# Build with optimizations
cargo build --release

# Build without topology validation (faster)
cargo build --release --no-default-features

# Format code
cargo fmt

# Run clippy linter
cargo clippy --fix --allow-dirty

# Check code without building
cargo check

# Build documentation
cargo doc -p opensubdiv-petite --no-deps --open

# Search with ripgrep
rg "AIDEV-"

# Find files
fd "\.rs$"
```

## Known Issues and TODOs

- [ ] PatchTable support not yet implemented.
- [ ] Metal backend pending implementation.
- [ ] OpenCL backend pending implementation.
- [ ] PTex support pending implementation.
- [ ] OpenMP detection broken on macOS.
- [ ] CUDA build configuration needs automation.

## Writing Instructions For User Interaction And Documentation

- Be concise and technical.
- AVOID marketing language or flattery.
- Use simple, direct sentences with technical jargon where appropriate.
- Do NOT overexplain basic concepts.
- AVOID generic claims without context.

## Documentation

- All code comments MUST end with a period.
- Doc comments end with period unless headlines.
- Use `---` for em-dash.
- Use `--` for en-dash.
- Enclose all type/keyword references in backticks: `TopologyRefiner`.
- First reference to external types should be linked: [`TopologyRefiner`].
- NEVER use fully qualified paths in doc links.

Remember: This wrapper prioritizes safety and idiomaticity over completeness. When in doubt about exposing unsafe functionality, choose the safe path.
