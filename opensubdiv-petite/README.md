# Open**Subdiv** *Petite* <img src="../osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

A selective Rust wrapper for *Pixar*’s
[*OpenSubdiv* library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

## Versions

For now crate versions reflect code maturity on the Rust side. They are not in
any way related to the *OpenSubdiv* version that is wrapped.

- `v0.1.x` – *OpenSubdiv* `v3.4.4`

## Features

There are several features to gate the resp. [build
flags](https://github.com/PixarAnimationStudios/OpenSubdiv#useful-cmake-options-and-environment-variables)
when *OpenSubdiv* is built.

Almost all of them are not yet implemented.

- [ ] `clew` – TBD. Adds support for
      [*CLEW*](https://github.com/martijnberger/clew).
- [ ] `cuda` – Adds support for the [*Nvidia CUDA*](https://developer.nvidia.com/cuda-toolkit)
      backend. *Only valid on Linux/Windows.*
      *CUDA* support is almost done (Rust API wrappers are there).
      It just require some more work in `build.rs`.
      Ideally, if the `cuda` feature flag is present, `build.rs` would detect a
      *CUDA* installation on *Linux*/*Windows* and configure the *OpenSubdiv*
      build resp. panic if no installation can be found.
- [ ] TBD. `metal` – Adds support for the *Apple*
      [*Metal*](https://developer.apple.com/metal/) backend. *Only valid on
      macOS.*
- [ ] `opencl` – TBD. Adds support for the
      [*OpenCL*](https://www.khronos.org/opencl/) backend.
- [ ] `ptex` – TBD. Adds support for [*PTex*](http://ptex.us/).
- [x] `topology_validation` – Do (expensive) validation of topology.  This
      checks index bounds on the Rust side and activates a bunch of topology
      checks on the FFI side.  *This is on by default!*
      Set `default-features = false` in `Cargo.toml` to switch this *off* –
      suggested for `release` builds.

### OpenMP Support on macOS

*OpenMP* detection is broken on the *CMake* side on *macOS*.  There are [a
bunch of issues](https://gitlab.kitware.com/cmake/cmake/-/issues?scope=all&state=opened&search=OpenMP) open in the CMake tracker. I added some comments [here](https://gitlab.kitware.com/cmake/cmake/-/issues/18470).

A workaround is likely possible. PRs welcome. If you need to make a fix on the
[*OpenSubdiv*](https://github.com/PixarAnimationStudios/OpenSubdiv) side, Pixar will probably also welcome a PR.

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
- Use canonical Rust constructs.  Most option/configuraion structs use the
  [init struct pattern](https://xaeroxe.github.io/init-struct-pattern/).
  In places where it’s not possible to easily map to a Rust struct, the builder
  pattern (or anti-pattern, depending whom you ask) is used.
- Be brief when possible. Example: `StencilTable::numStencils()` in C++
  becomes `StencilTable::len()` in Rust.
- Use unsigned integer types, specifically `usize` and `u32`, instead of
  signed ones (`i32`) for anything that can only contain positive values
  (indices, sizes/lengths/counts, valences, arities, etc.).  Types should
  express intent.  See also
  [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).
