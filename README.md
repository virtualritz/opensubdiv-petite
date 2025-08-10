# `opensubdiv-petite` <img src="osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

A selective Rust wrapper for *Pixar*’s
[*OpenSubdiv* library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

For more information on the high level wrapper see the README in the
`opensubdiv-petite` folder.

The repositoy comes with minimal dependencies. *OpenSubdiv* is tracked as a
*Git* submodule under `opensubdiv-petite-sys/OpenSubdiv`.

Either clone the repository with `--recursive` or, if you already cloned it and
forgot, simply do a

```shell
git submodule update --init
```

to pull them in.

## Building

This project uses [just](https://github.com/casey/just) for common build tasks. Run `just` to see available commands.

### Quick Start

```shell
# Build the project
just build

# Run tests
just test

# Build documentation
just doc
```

### Linux Build Requirements

On Ubuntu/Debian systems, you may encounter C++ standard library issues. We recommend using Clang 17 with libc++:

```shell
# Install required packages
sudo apt install -y clang-17 libc++-17-dev libc++abi-17-dev cmake

# Build with clang-17 and libc++
just build-linux-clang17

# Run tests with clang-17
just test-linux-clang17

# Run a specific example
just run-example-linux-clang17 far_tutorial_0
```

### Common Tasks

```shell
# Build commands
just build                      # Standard build
just build-release              # Release build with optimizations
just build-linux-clang17        # Build with clang-17 (Ubuntu/Debian)
just build-all-features         # Build with all features enabled

# Test commands
just test                       # Run all tests
just test-linux-clang17         # Run tests with clang-17
just test-linux-clang17-no-default  # Run tests without topology validation (faster)
just test-linux-clang17-specific <name>  # Run specific test with clang-17

# Development commands
just check                      # Check code without building
just clippy                     # Run clippy linter
just fmt                        # Format code
just doc                        # Build and open documentation

# Clean commands
just clean                      # Clean project build artifacts
just clean-all                  # Clean entire target directory

# Example commands
just run-example <name>         # Run an example
just run-example-linux-clang17 <name>  # Run example with clang-17
```

See the `justfile` for all available commands and their exact implementations.

## Documentation

It is suggested you only build (and look at) the documentation of the high level
wrapper:

```shell
cargo doc -p opensubdiv --no-deps --open
```

## GPU Backend Support

### Available GPU Backends

The following GPU backends are now available:

* **CUDA** (NVIDIA GPUs) - Enable with `cuda` feature flag
* **Metal** (Apple GPUs) - Enable with `metal` feature flag  
* **OpenCL** (Cross-platform) - Enable with `opencl` feature flag

### Known Issues

#### CUDA with GCC 13+
When building with CUDA support on systems with GCC 13 or newer, you may encounter compilation errors. CUDA 12.0 officially supports up to GCC 12. While we've added the `-allow-unsupported-compiler` flag to the build configuration, there are still compatibility issues with system headers.

**Workaround**: Use GCC 12 or earlier, or build without CUDA support on systems with GCC 13+.

## Help Wanted

Specifically (in no particular order) these are issues for which you can put your
hand up or just open a PR:

* [x] Add support for the *CUDA* backend (initial support added, GCC 13+ compatibility issues remain).
* [ ] [Add support for the *DX11* backend](https://github.com/virtualritz/opensubdiv-petite/issues/4).
* [x] Add support for the *Metal* backend (initial support added).
* [x] Add support for the *OpenCL* backend (initial support added).
* [ ] [Fix *OpenMP* detection on macOS](https://github.com/virtualritz/opensubdiv-petite/issues/2).
* [ ] [Fix `StencilTable`](https://github.com/virtualritz/opensubdiv-petite/issues/1).
* [ ] [Add `PatchTable`](https://github.com/virtualritz/opensubdiv-petite/issues/5).

## Versions

For now crate versions reflect code maturity on the Rust side. They are not in
any way related to the *OpenSubdiv* version that is wrapped.

- `opensubdiv-petite[-sys] v0.3.x` – *OpenSubdiv* `v3.6.x`
- `opensubdiv-petite[-sys] v0.2.x` – *OpenSubdiv* `v3.5.x`
- `opensubdiv-petite[-sys] v0.1.x` – *OpenSubdiv* `v3.4.x`
