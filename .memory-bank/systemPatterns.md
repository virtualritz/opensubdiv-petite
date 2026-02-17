# System Patterns

## Architecture

```
┌─────────────────────────────────────────────┐
│  opensubdiv-petite  (safe Rust API)         │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌──────────────┐  │
│  │ far │ │ osd │ │ bfr │ │ integrations │  │
│  └──┬──┘ └──┬──┘ └──┬──┘ └──────┬───────┘  │
│     └────────┴───────┴───────────┘           │
│                    │                         │
├────────────────────┼─────────────────────────┤
│  opensubdiv-petite-sys  (FFI bindings)       │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ bindings │  │  c-api   │  │  build.rs  │  │
│  │ (bindgen)│  │ (C shim) │  │(cmake+cc) │  │
│  └──────────┘  └──────────┘  └───────────┘  │
├──────────────────────────────────────────────┤
│  OpenSubdiv (vendored C++ submodule v3.7.x) │
└──────────────────────────────────────────────┘
```

## Key Patterns

### FFI Bridge Pattern

C++ templates cannot be called directly from Rust. The `c-api/` layer provides plain C functions that instantiate the templates, which bindgen can then generate Rust bindings for. The safe crate wraps these in idiomatic Rust types.

### Init Struct Pattern

Configuration is passed via init structs with `Default` implementations:

- `TopologyRefinerOptions` -- scheme, boundary interpolation, etc.
- `UniformRefinementOptions` -- refinement level, vertex ordering.
- `AdaptiveRefinementOptions` -- isolation level, crease patches.
- `StencilTableOptions` -- interpolation mode, offset generation.
- `PatchTableOptions` -- end cap type, triangle subdivision.
- `StepExportOptions` -- Gregory accuracy, stitching.

### Newtype Index Pattern

`Index(pub u32)` is `#[repr(transparent)]` with `bytemuck::Pod/Zeroable`. Converts to/from `u32` and `usize`. Used throughout for type-safe vertex/edge/face indices.

### Borrowed View Pattern

`TopologyLevel<'a>` and `StencilTableRef<'a>` are borrowed views into parent objects (`TopologyRefiner`, `PatchTable`). Lifetime ties prevent use-after-free.

### Feature-Gated Backends

GPU backends are behind feature flags. Each backend follows the same pattern:

- `XxxVertexBuffer` -- GPU-side vertex storage.
- `xxx_evaluator::evaluate_stencils()` -- GPU evaluation function.
- Backends: CPU (always), CUDA, Metal, OpenCL, wgpu.

### Export Trait Pattern

`PatchTableExt`, `PatchTableIgesExt`, `PatchTableObjExt` extend `PatchTable` with export capabilities via extension traits. This keeps the core type clean while allowing feature-gated functionality.

## Design Decisions

### Unsigned Indices

OpenSubdiv uses signed `int` for indices. This crate uses `u32`/`usize` because negative indices are never valid. The FFI boundary handles the cast.

### Topology Validation Default-On

`topology_validation` feature is enabled by default. It adds bounds checking in `TopologyDescriptor` construction. Disable for release performance.

### Knot Vectors

B-spline patches use uniform knots `[-3,-2,-1,0,1,2,3,4]`. This preserves C2 continuity. Clamped knots would create Bezier patches with kinks. This is intentional and must not be changed.

### Send + Sync

- `TopologyLevel`, `PatchTable`, `PatchMap`, `SurfaceFactory`, `Surface` -- `Send + Sync`.
- `TopologyRefiner`, `TopologyDescriptor` -- NOT `Send + Sync` (raw C++ pointers).

### Error Handling

`thiserror`-derived `Error` enum with variants for each failure mode. GPU-specific variants are feature-gated. `Result<T>` type alias used throughout.
