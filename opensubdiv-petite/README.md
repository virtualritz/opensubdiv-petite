# `opensubdiv-petite` <img src="../osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

A selective Rust wrapper for _Pixar_’s
[_OpenSubdiv_ library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

## Versions

For now crate versions reflect code maturity on the Rust side. They are not in
any way related to the _OpenSubdiv_ version that is wrapped.

- `v0.3.x` – _OpenSubdiv_ `v3.6.x`
- `v0.2.x` – _OpenSubdiv_ `v3.5.0`
- `v0.1.x` – _OpenSubdiv_ `v3.4.4`

## Build Requirements

### Ubuntu/Debian

This crate requires a working C++ compiler with standard library support. If you encounter errors about missing headers (e.g., `'cassert' file not found`), you'll need to specify the compiler and C++ standard library explicitly.

We recommend using Clang 17 with libc++:

```bash
# Install required packages
sudo apt install -y clang-17 libc++-17-dev libc++abi-17-dev cmake

# Build with the recommended compiler and linker flags
CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" \
RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" \
cargo build
```

**Known Issues:**

- The default system Clang (version 20+) may have compatibility issues with CUDA
- GNU `libstdc++` packages may have missing headers on some systems
- If you have `CC`/`CXX` environment variables set in your shell configuration (e.g., in `~/.bashrc` or `~/.zshrc`), they may override the build settings. Either unset them or use the explicit command above
- Linking errors with undefined symbols from `std::` namespace indicate a mismatch between the C++ standard library used for compilation vs linking. The `RUSTFLAGS` above ensure consistent linking

### macOS

The crate should build with the default Xcode toolchain. For OpenMP support, see the section below.

### Windows

Use MSVC or MinGW-w64. CUDA support requires MSVC.

## Features

There are several features to gate the resp. [build
flags](https://github.com/PixarAnimationStudios/OpenSubdiv#useful-cmake-options-and-environment-variables)
when _OpenSubdiv_ is built.

Almost all of them are not yet implemented.

- [ ] `clew` – TBD. Adds support for
      [_CLEW_](https://github.com/martijnberger/clew).
- [ ] `cuda` – Adds support for the [_Nvidia CUDA_](https://developer.nvidia.com/cuda-toolkit)
      backend. _Only valid on Linux/Windows._
      _CUDA_ support is almost done (Rust API wrappers are there).
      It just require some more work in `build.rs`.
      Ideally, if the `cuda` feature flag is present, `build.rs` would detect a
      _CUDA_ installation on _Linux_/_Windows_ and configure the _OpenSubdiv_
      build resp. panic if no installation can be found.
- [ ] TBD. `metal` – Adds support for the _Apple_
      [_Metal_](https://developer.apple.com/metal/) backend. _Only valid on
      macOS._
- [ ] `opencl` – TBD. Adds support for the
      [_OpenCL_](https://www.khronos.org/opencl/) backend.
- [ ] `ptex` – TBD. Adds support for [_PTex_](http://ptex.us/).
- [x] `topology_validation` – Do (expensive) validation of topology. This
      checks index bounds on the Rust side and activates a bunch of topology
      checks on the FFI side. _This is on by default!_
      Set `default-features = false` in `Cargo.toml` to switch this _off_ –
      suggested for `release` builds.
- [x] `bevy` – Enables integration with the Bevy game engine. This also enables
      the `tri_mesh_buffers` feature. See the `bevy` example for usage.

### OpenMP Support on macOS

_OpenMP_ detection is broken on the _CMake_ side on _macOS_. There are [a
bunch of issues](https://gitlab.kitware.com/cmake/cmake/-/issues?scope=all&state=opened&search=OpenMP) open in the CMake tracker. I added some comments [here](https://gitlab.kitware.com/cmake/cmake/-/issues/18470).

A workaround is likely possible. PRs welcome. If you need to make a fix on the
[_OpenSubdiv_](https://github.com/PixarAnimationStudios/OpenSubdiv) side, Pixar will probably also welcome a PR.

## Limitations

The original library does make use of C++ templates in quite a few places.
The wrapper has specializations that cover the most common use cases.

C++ factory classes have been collapsed into the `new()` method of the resp.
struct that mirrors the class the C++ factory was building.

## API Changes From C++

Many methods have slightly different names on the Rust side.

Renaming was done considering these constraints:

- Be verbose consistently (the original API is quite verbose but does make use
  of abbreviations in some suprising places).
- Use canonical Rust naming – (`num_vertices()` becomes `vertices_len()`).
- Use canonical Rust constructs. Most option/configuraion structs use the
  [init struct pattern](https://xaeroxe.github.io/init-struct-pattern/).
  In places where it’s not possible to easily map to a Rust struct, the builder
  pattern (or anti-pattern, depending whom you ask) is used.
- Be brief when possible. Example: `StencilTable::numStencils()` in C++
  becomes `StencilTable::len()` in Rust.
- Use unsigned integer types, specifically `usize` and `u32`, instead of
  signed ones (`i32`) for anything that can only contain positive values
  (indices, sizes/lengths/counts, valences, arities, etc.). Types should
  express intent. See also
  [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).
