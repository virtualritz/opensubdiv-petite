# `opensubdiv-petite` <img src="../osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

<!-- cargo-rdme start -->

## Pixar OpenSubdiv Wrapper

This is a safe Rust wrapper around parts of [*Pixar's
OpenSubdiv*](https://graphics.pixar.com/opensubdiv/).

*OpenSubdiv* is a set of open source libraries that implement high
performance/parallel [subdivision surface](https://en.wikipedia.org/wiki/Subdivision_surface)
(subdiv) evaluation on CPU and GPU architectures.

The code is optimized for drawing deforming surfaces with static topology at
interactive framerates.

### Limitations

The original library does make use of templates in quite a few places.
The wrapper has specializations that cover the most common use case.

C++ factory classes have been collapsed into the `new()` method of the resp.
struct that mirrors the class the C++ factory was building.

### API Changes From C++

Many methods have slightly different names on the Rust side.

Renaming was done considering these constraints:
* Be verbose consistently (the original API is quite verbose but does make
  use of abbreviations in some surprising places).
* Use canonical Rust naming  – (`num_vertices()` becomes `vertex_count()`).
* Use canonically Rust constructs.  Most option/configuration `struct`s use the
  [init-`struct` pattern](https://xaeroxe.github.io/init-struct-pattern/). In
  places where it’s not possible to easily map to a Rust `struct`, the builder
  pattern (or anti-pattern, depending whom you ask) is used.
* Be brief when possible. Example: `StencilTable::numStencils()` in C++
  becomes `StencilTable::len()` in Rust.
* Use unsigned integer types, specifically `usize` and `u32`, instead of
  signed ones (`i32`) for anything that can only contain positive values
  (indices, sizes/lengths/counts, valences, arities, etc.).  Types should
  express intent.  See also
  [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).

### OpenSubdiv Backend Support

*OpenSubdiv* exposes several optional backends via CMake flags. The table
below shows which ones this wrapper supports today.

| Backend | Feature flag | Status |
|---------|-------------|--------|
| CPU (single-threaded) | — | Always enabled |
| TBB (CPU parallel) | `tbb` | Supported |
| CUDA (NVIDIA GPU) | `cuda` | Supported |
| Metal (Apple GPU) | `metal` | Supported |
| OpenCL | `opencl` | Supported |
| wgpu/WGSL (compute) | `wgpu` | Supported (Rust-native, not from C++) |
| OpenMP (CPU parallel) | `omp` | Supported (broken on macOS) |
| CLEW (OpenCL loader) | `clew` | Build flag only — no Rust API |
| PTex | `ptex` | Build flag only — no Rust API |
| OpenGL | — | Not yet supported |
| DirectX 11 | — | Not yet supported |

#### wgpu

The `wgpu` feature enables a **pure-Rust** GPU compute path for stencil
evaluation using WGSL shaders.  This is not an *OpenSubdiv* backend —
it uploads `StencilTable` data to `wgpu` storage buffers and
dispatches a WGSL compute shader.

```rust
use opensubdiv_petite::osd::wgpu::*;

// Upload stencil table to GPU.
let gpu_stencils = StencilTableGpu::from_cpu(&device, &stencil_table)?;

// Create the compute pipeline (once).
let pipeline = StencilEvalPipeline::new(&device, WgslModuleConfig::default());

// Evaluate: src_buffer → dst_buffer.
evaluate_stencils(
    &device, &queue, &pipeline, &gpu_stencils,
    &src_buffer, &dst_buffer, src_desc, dst_desc,
    0..gpu_stencils.stencil_count,
)?;
```

### Cargo Features

### Versions

For now crate versions reflect code maturity on the Rust side. They are not
in any way related to the *OpenSubdiv* version that is wrapped.

- `v0.3.x` – *OpenSubdiv* `v3.7.x`
- `v0.2.x` – *OpenSubdiv* `v3.5.x`
- `v0.1.x` – *OpenSubdiv* `v3.4.x`

<!-- cargo-rdme end -->

## Cargo Features

- **`bevy`** — Integration with Bevy game engine.
- **`clew`** — Use CLEW for OpenCL runtime loading.
- **`cuda`** — Enable CUDA GPU backend for NVIDIA GPUs.
  Requires the [CUDA Toolkit](https://developer.nvidia.com/cuda-toolkit).
  On Ubuntu: `sudo apt install nvidia-cuda-toolkit`. GCC 12 is recommended
  as the host compiler (`sudo apt install gcc-12 g++-12`).
- **`metal`** — Enable Metal GPU backend for Apple devices.
- **`omp`** — Enable OpenMP for CPU parallelization. Alias for `openmp`.
  Requires an OpenMP-capable compiler:
  - **Linux:** GCC has built-in support; Clang needs `libomp-dev` (`sudo apt install libomp-dev`).
  - **macOS:** Broken — Apple Clang lacks OpenMP. Homebrew Clang may work but is untested.
  - **Windows:** MSVC has built-in support.
- **`opencl`** — Enable OpenCL GPU backend for cross-platform GPU support.
  Requires an OpenCL SDK/ICD loader:
  - **Linux:** `sudo apt install ocl-icd-opencl-dev` (Debian/Ubuntu).
  - **macOS:** Included with Xcode.
  - **Windows:** Install the OpenCL SDK from your GPU vendor.
- **`openmp`** — Enable OpenMP for CPU parallelization (alias for `omp`).
- **`ptex`** — Enable PTex texture support.
- **`rayon`** — Enable parallel processing with `rayon`.
- **`tbb`** — Enable TBB (Threading Building Blocks) CPU backend.
  Requires TBB installed on your system:
  - **Linux:** `sudo apt install libtbb-dev` (Debian/Ubuntu) or `sudo dnf install tbb-devel` (Fedora).
  - **macOS:** `brew install tbb`.
  - **Windows:** Install [oneAPI TBB](https://github.com/oneapi-src/oneTBB) and ensure CMake can find it.
- **`tri_mesh_buffers`** — Enable triangle mesh buffer generation.
- **`topology_validation`** _(enabled by default)_ — Enable topology validation for debugging. Disable for release builds.
- **`wgpu`** — Enable WGSL compute path (wgpu).
- **`truck`** — Enable `truck` CAD kernel integration for B-rep export.
- **`truck_export_boundary`** — Export boundary curves when using `truck` integration.
- **`b_spline_end_caps`** — Use B-spline basis end caps instead of Gregory patches (legacy behavior).

## License

Apache-2.0
