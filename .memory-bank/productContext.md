# Product Context: opensubdiv-petite

## Why This Exists

Pixar's OpenSubdiv is the industry standard for subdivision surface evaluation, but it's a C++ library with heavy template usage. Rust developers need safe, ergonomic access to this functionality for:

- Real-time 3D rendering (games, visualization).
- CAD/CAM applications requiring precise surface evaluation.
- Offline rendering pipelines.
- Mesh processing tools.

## Problems Solved

1. **Safety** -- Raw OpenSubdiv requires careful memory management and index validation. This wrapper enforces safety at compile time and runtime (topology validation).
2. **Ergonomics** -- C++ factory patterns, raw pointers, and signed indices are replaced with Rust idioms: `new()` constructors, `Result` types, unsigned indices, iterators.
3. **Build complexity** -- The sys crate handles the CMake + C++ compilation, so users just add a Cargo dependency.
4. **Integration** -- Bridges OpenSubdiv to the Rust ecosystem: Bevy, truck CAD kernel, wgpu compute.

## User Experience Goals

- Drop-in Cargo dependency for subdivision surfaces.
- Compile-time safety guarantees where possible.
- Feature flags to keep binary size small -- only pay for what you use.
- GPU backends (CUDA, wgpu) for production-scale evaluation.
- CAD export (STEP via truck) for engineering workflows.

## Target Users

- Rust game developers (via Bevy integration).
- CAD/geometry processing developers (via truck integration).
- Graphics researchers and tool builders.
- Anyone needing production-quality subdivision surfaces in Rust.
