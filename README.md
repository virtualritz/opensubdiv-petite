# OpenSubdiv
A Rust wrapper for *Pixar*’s [*OpenSubdiv* library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

The repositoy comes with minimal dependencies. I.e. OpenSubdiv and
[GLFW](https://www.glfw.org/) are tracked as *Git* submodules under
`opensubdiv/dependencies`.

Either clone the repository with `--recursive` or, if you already cloned it and
forgot, simply do a `git submodule update --init` to pull them in.
## Features
There are several features to gate the resp.
[buildflags](https://github.com/PixarAnimationStudios/OpenSubdiv#useful-cmake-options-and-environment-variable)
when *OpenSubdiv* is built.

Almost all of them are not yet implemented.

- [ ] `clew` – TBD
- [ ] `cuda` – Adds support for the [*Nvidia CUDA*](https://developer.nvidia.com/cuda-toolkit)
    backend. *Not supported on macOS.*

    *CUDA* support is almost done (Rust API wrappers are there).

    It just require some more work in `build.rs`.
    Ideally, if the `cuda` feature flag is present, `build.rs` would detect a
    *CUDA* installation on *Linux* or *Windows* and configure the OpenSubdiv
    build resp. panic if no instalation can be found.
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

I.e. to build with OpenMP support on macOS make sure you install `LLVM` via
Homebrew before building:
```
brew install llvm
```
