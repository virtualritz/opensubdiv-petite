# `opensubdiv-petite` <img src="osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

A selective Rust wrapper for _Pixar_’s
[_OpenSubdiv_ library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

For more information on the high level wrapper see the README in the
`opensubdiv-petite` folder.

The repositoy comes with minimal dependencies. _OpenSubdiv_ is tracked as a
_Git_ submodule under `opensubdiv-petite-sys/OpenSubdiv`.

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
just test-linux-clang17         # Run all tests with clang-17
just test-linux-clang17 <name>  # Run specific test with clang-17
just test-linux-clang17-nocapture       # Run all tests with output visible
just test-linux-clang17-nocapture <name> # Run specific test with output visible

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

- **CUDA** (NVIDIA GPUs) - Enable with `cuda` feature flag
- **Metal** (Apple GPUs) - Enable with `metal` feature flag
- **OpenCL** (Cross-platform) - Enable with `opencl` feature flag

### Known Issues

#### CUDA Compiler Compatibility

While CUDA documentation states support for GCC up to version 15, CUDA 12.0 has compatibility issues with GCC 13+ due to changes in C++ standard library headers (specifically `_Float32` types in math headers).

**Solution**:
The build system automatically detects and uses GCC 12 if available for CUDA compilation. On Ubuntu 24.04:

```bash
# Install GCC 12 (if not already installed)
sudo apt-get install gcc-12 g++-12

# Build with CUDA support (automatic GCC 12 detection)
cargo build --features cuda
```

The build script will automatically use GCC 12 for CUDA compilation while the rest of your system continues to use GCC 13+.

## Help Wanted

Contributions are welcome! Here are some areas where help is needed:

- [ ] [Add support for the _DX11_ backend](https://github.com/virtualritz/opensubdiv-petite/issues/4).
- [ ] [Fix _OpenMP_ detection on macOS](https://github.com/virtualritz/opensubdiv-petite/issues/2).
- [ ] Improve GPU backend support (CUDA/Metal/OpenCL) - currently experimental.
- [ ] Add more comprehensive examples and documentation.

## Recently Completed

- [x] CUDA backend support (with GCC 12 compatibility workaround).
- [x] Metal backend support.
- [x] OpenCL backend support.
- [x] `StencilTable` implementation.
- [x] `PatchTable` implementation with Gregory patches support.

## Versions

For now crate versions reflect code maturity on the Rust side. They are not in
any way related to the _OpenSubdiv_ version that is wrapped.

- `opensubdiv-petite[-sys] v0.3.x` – _OpenSubdiv_ `v3.6.x`
- `opensubdiv-petite[-sys] v0.2.x` – _OpenSubdiv_ `v3.5.x`
- `opensubdiv-petite[-sys] v0.1.x` – _OpenSubdiv_ `v3.4.x`
