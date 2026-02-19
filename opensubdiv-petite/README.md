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

### Cargo Features

### Versions

For now crate versions reflect code maturity on the Rust side. They are not
in any way related to the *OpenSubdiv* version that is wrapped.

- `v0.3.x` – *OpenSubdiv* `v3.7.x`
- `v0.2.x` – *OpenSubdiv* `v3.5.x`
- `v0.1.x` – *OpenSubdiv* `v3.4.x`

<!-- cargo-rdme end -->
