# Tech Context

## Language & Toolchain

- Rust (edition not specified in workspace Cargo.toml, inherited per crate).
- C++14 for OpenSubdiv and C wrapper layer.
- Workspace with `resolver = "2"`.

## Build System

1. **CMake** builds vendored OpenSubdiv from git submodule (`opensubdiv-petite-sys/OpenSubdiv/`).
2. **cc** compiles `c-api/**/*.cpp` as `osd-capi` static library.
3. **bindgen** generates `bindings.rs` from `wrapper.hpp`.
4. Links against: `osdCPU` (always), `osdGPU` (if CUDA/OpenCL/Metal).
5. Platform: libstdc++ (Linux), libc++ (macOS).
6. CUDA workaround: requires GCC 12 (`CUDA_HOST_COMPILER`).

## Dependencies

### sys crate

| Dep         | Version | Purpose                        |
| ----------- | ------- | ------------------------------ |
| derive_more | 2       | Derive macros                  |
| num_enum    | 0.7     | Enum <-> integer conversions   |
| cc          | 1       | C/C++ compilation (build)      |
| cmake       | 0.1     | CMake integration (build)      |
| bindgen     | 0.72    | FFI binding generation (build) |

### main crate

| Dep               | Version | Purpose                         |
| ----------------- | ------- | ------------------------------- |
| bytemuck          | 1       | Safe transmutes for vertex data |
| derive_more       | 2       | Derive macros                   |
| document-features | 0.2     | Feature flag documentation      |
| num_enum          | 0.7     | Enum conversions                |
| thiserror         | 2       | Error derive macros             |

### Optional deps

| Dep                 | Version | Feature                          |
| ------------------- | ------- | -------------------------------- |
| rayon               | 1       | `rayon` -- parallel iteration    |
| itertools           | 0.14    | `tri_mesh_buffers`               |
| ultraviolet         | 0.10    | `tri_mesh_buffers` -- math types |
| slice-of-array      | 0.3     | `tri_mesh_buffers`               |
| truck-geometry      | git     | `truck` -- CAD kernel            |
| truck-modeling      | git     | `truck` -- CAD modeling          |
| bevy                | 0.16    | `bevy` -- game engine            |
| smooth-bevy-cameras | 0.14    | `bevy` -- camera controls        |
| wgpu                | 24      | `wgpu` -- GPU compute            |

### Dev deps

| Dep          | Version | Purpose                      |
| ------------ | ------- | ---------------------------- |
| truck-stepio | git     | STEP file I/O for tests      |
| anyhow       | 1.0     | Error handling in examples   |
| glam         | 0.30    | Math types in examples       |
| pollster     | 0.3     | Async runtime for wgpu tests |

## Feature Flags

### Default: `topology_validation`

### GPU backends: `cuda`, `metal` (TODO), `opencl` (TODO), `wgpu` (in progress)

### CPU parallelism: `omp`/`openmp`, `rayon`

### Integrations: `truck`, `truck_export_boundary`, `bevy`, `tri_mesh_buffers`

### Other: `ptex` (TODO), `clew`, `b_spline_end_caps`

## Key Commands

```bash
cargo build                              # Build all
cargo test                               # Run tests (no feature-gated tests)
cargo test --features truck              # Run truck integration tests
cargo test --features wgpu               # Run wgpu tests
cargo run --example far_tutorial_0       # Basic example
cargo clippy --fix --allow-dirty         # Lint
cargo fmt                                # Format
cargo doc -p opensubdiv-petite --no-deps # Docs
```

## Test Infrastructure

- Expected results in `tests/expected_results/` (STEP, IGES, OBJ, TXT files).
- `RUST_UPDATE_EXPECTED_TEST_RESULTS=1` env var to regenerate expected results.
- Shared helpers in `tests/utils.rs`: `default_end_cap_type()`, `assert_file_matches()`, `test_output_path()`.
