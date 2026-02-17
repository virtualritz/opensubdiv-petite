# Project Brief: opensubdiv-petite

## Identity

Rust bindings for Pixar's OpenSubdiv library -- a "petite" wrapper providing safe, idiomatic access to high-performance subdivision surface evaluation.

## Two-Crate Structure

- **`opensubdiv-petite-sys`** (v0.3.1) -- Raw FFI bindings. Builds OpenSubdiv v3.7.x from a vendored git submodule via CMake + cc + bindgen. Ships a C wrapper layer (`c-api/`) that bridges Rust FFI to C++ templates.
- **`opensubdiv-petite`** (v0.3.1) -- Safe, idiomatic Rust API. Selective coverage of the most commonly used OpenSubdiv functionality.

## Scope

This is intentionally "petite" -- it does not expose the full OpenSubdiv API. Focus areas:

1. **Far (Feature Adaptive Representation)** -- `TopologyRefiner`, `TopologyDescriptor`, `TopologyLevel`, `PrimvarRefiner`, `StencilTable`, `PatchTable`.
2. **Osd (OpenSubdiv Draw)** -- `CpuVertexBuffer`, CPU/CUDA/Metal/OpenCL/wgpu evaluators, `BufferDescriptor`.
3. **Bfr (Base Face Representation)** -- `SurfaceFactory`, `Surface` for direct patch evaluation.
4. **Integrations** -- truck CAD kernel (STEP export), Bevy game engine, IGES/OBJ export, triangle mesh buffers.

## Core Design Principles

- Safe Rust API over C++ template-heavy code.
- Zero-cost abstractions over FFI calls.
- Init struct / builder patterns for configuration.
- `Result<T, Error>` for fallible operations.
- Feature gates for optional GPU backends and integrations.
- Unsigned index types (`u32`, `usize`) instead of signed integers.
- Topology validation enabled by default (disable for release performance).
