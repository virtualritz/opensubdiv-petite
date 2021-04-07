# opensubdiv <img src="osd-logo.png" alt="OpenSubdiv Logo" width="15%" align="right" align="top">

A Rust wrapper for *Pixar*’s
[*OpenSubdiv* library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

The repositoy comes with minimal dependencies. I.e. *OpenSubdiv* and
[GLFW](https://www.glfw.org/) are tracked as *Git* submodules under
`opensubdiv/dependencies`.

Either clone the repository with `--recursive` or, if you already cloned it and
forgot, simply do a

```shell
git submodule update --init
```

to pull them in.

## Documentation

It is suggested you only build (and look at) the documentation of the high level wrapper:
```
cargo doc -p opensubdiv --no-deps --open
```

## Features

There are several features to gate the resp. [build
flags](https://github.com/PixarAnimationStudios/OpenSubdiv#useful-cmake-options-and-environment-variables)
when *OpenSubdiv* is built.

Almost all of them are not yet implemented.

- [ ] `clew` – TBD
- [ ] `cuda` – Adds support for the [*Nvidia CUDA*](https://developer.nvidia.com/cuda-toolkit)
    backend. *Only valid on Linux/Windows.*
    *CUDA* support is almost done (Rust API wrappers are there).
    It just require some more work in `build.rs`.
    Ideally, if the `cuda` feature flag is present, `build.rs` would detect a
    *CUDA* installation on *Linux*/*Windows* and configure the *OpenSubdiv*
    build resp. panic if no installation can be found.
- [ ] `metal` – Adds support for the *Apple*
     [*Metal*](https://developer.apple.com/metal/) backend. *Only valid on
     macOS.*
- [ ] `opencl` – TBD
- [ ] `ptex` – TBD

### OpenMP Support on macOS

The library will be built with [OpenMP](https://www.openmp.org/) support on
*macOS* only if you have a
non-*Apple* *Clang* installed.

The `build.rs` looks in `/usr/local/opt/llvm/bin` for the `clang` and `clang++`
executables. This is the default location [Homebrew](https://brew.sh/) installs
`llvm` in.

I.e. to build with *OpenMP* support on *macOS* make sure you install `LLVM` via
*Homebrew* before building:

```shell
brew install llvm
```

## Limitations

The original library does make use of C++ templates in quite a few places.
The wrapper has specializations that cover the most common use cases.

C++ factory classes have been collapsed into the `new()` method of the resp.
struct that mirrors the class the C++ factory was building.

## API Changes From C++

Many methods have slightly different names on the Rust side.

Renaming was done considering these constraints:

- Be verbose consistently (the original API is quite verbose but does make
  use of abbreviations in some suprising places).
- Use canonical Rust naming (`num_vertices()` becomes `vertices_len()`)
- Use canonical Rust constructs (e.g. the builder pattern – or anti-pattern,
  depending whom you ask). I will probably switch this to an [init struct
  pattern](https://xaeroxe.github.io/init-struct-pattern/) soon.  Even though
  this means a minimal overhead for some structs which are better left for
  `bindgen` to define and then require copying.
- Be brief when possible. Example: `StencilTable::numStencils()` in C++
  becomes `StencilTable::len()` in Rust.
- Use usnigned integer types, specifically `u32`, instead of signed ones
  (`i32`) for anything that can only contain positive values (indices,
  sizes/lengths/counts, valences, arities, etc.).  Types should express intent.
  See also [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).
