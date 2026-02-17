# Progress

## What Works

### Core API (Stable)

- `TopologyDescriptor` -- mesh input with creases, corners, holes.
- `TopologyRefiner` -- uniform and adaptive refinement.
- `TopologyLevel` -- full topology queries at each refinement level.
- `PrimvarRefiner` -- vertex, varying, face-varying, face-uniform interpolation.
- `StencilTable` -- stencil creation, iteration, value updates.
- `PatchTable` -- patch creation, basis evaluation, point evaluation, local points.
- `PatchMap` -- fast patch lookup by face/parametric coordinates.
- `Index` newtype -- safe index conversions.

### CPU Evaluation (Stable)

- `CpuVertexBuffer` -- vertex data storage.
- `cpu_evaluator::evaluate_stencils()` -- CPU stencil evaluation.

### CUDA Backend (Stable, feature-gated)

- `CudaVertexBuffer`, `CudaStencilTable`, `cuda_evaluator::evaluate_stencils()`.

### BFR (Stable)

- `SurfaceFactory` -- surface initialization from topology.
- `Surface` -- evaluation, control point queries.

### Exports (Stable)

- IGES B-spline export.
- OBJ B-spline surface export.
- OBJ polygon export.

### truck Integration (Active, feature-gated)

- B-spline surface extraction from regular patches.
- Gregory patch approximation (BSpline end caps and high-precision 8x8 fitting).
- STEP export via multiple strategies: `to_step_shell()`, `to_step_shell_fallback()`.
- Superpatch merging for adjacent regular patches.
- BFR mixed-mode surface generation.
- Edge stitching for watertight B-rep.
- Gap filling for extraordinary vertices.

## In Progress

### wgpu Compute Backend

- Position evaluation via WGSL compute shader: **works**.
- CPU vs GPU parity test: **passes**.
- Derivative evaluation: **not yet wired** (AIDEV-TODO).
- Pipeline structure with 17 bindings ready for derivatives.

## Not Yet Implemented

- **Metal backend** -- types exist but evaluator not implemented.
- **OpenCL backend** -- types exist but evaluator not implemented.
- **PTex support** -- feature flag exists, no implementation.
- **Full PatchTable coverage** -- some advanced features not exposed.

## Known Issues

- OpenMP detection broken on macOS.
- CUDA build requires GCC 12 workaround (`CUDA_HOST_COMPILER`).
- `TopologyRefiner` and `TopologyDescriptor` are not `Send + Sync`.
- Local point stencil tables may report `control_vertex_count() == 0` (handled in C++ layer).
- Per-vertex component cap in WGSL shader is 32 (may need increase for complex meshes).
