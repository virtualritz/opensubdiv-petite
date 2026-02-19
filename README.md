# `opensubdiv-petite` <img src="osd-logo.png" alt="OpenSubdiv Logo" width="15%" padding-bottom="5%" align="right" align="top">

A safe Rust wrapper for _Pixar_'s
[_OpenSubdiv_ library](http://graphics.pixar.com/opensubdiv/docs/intro.html).

See the [`opensubdiv-petite` crate README](opensubdiv-petite/README.md) for API
documentation, limitations, and feature flags.

## Getting the Source

_OpenSubdiv_ is tracked as a _Git_ submodule under
`opensubdiv-petite-sys/OpenSubdiv`.

Clone with `--recursive`, or pull the submodule into an existing checkout:

```shell
git submodule update --init
```

## Build Requirements

### Linux (Ubuntu/Debian)

Clang 17 with libc++ is recommended:

```shell
sudo apt install -y clang-17 libc++-17-dev libc++abi-17-dev cmake

CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" \
RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" \
cargo build
```

Or, if you use [just](https://github.com/casey/just):

```shell
just build-linux-clang17
```

**Known issues:**

- System Clang 20+ may have CUDA compatibility problems.
- GNU `libstdc++` may have missing headers on some distros.
- Shell-level `CC`/`CXX` overrides can conflict --- unset them or use the
  explicit command above.

### macOS

Builds with the default Xcode toolchain.

_OpenMP_ detection is broken on the _CMake_ side.
See [CMake issue #18470](https://gitlab.kitware.com/cmake/cmake/-/issues/18470).

### Windows

Use MSVC or MinGW-w64. CUDA support requires MSVC.

## Building & Common Tasks

This project uses [just](https://github.com/casey/just). Run `just` to list
all recipes. Highlights:

```shell
just build                # standard build
just test                 # run tests
just doc                  # build & open docs
just clippy               # lint
just fmt                  # format
```

Linux-specific variants (`*-linux-clang17`) set the compiler flags
automatically. See the `justfile` for details.

## GPU Backend Support

### CUDA

Enable with `--features cuda`. The build system auto-detects GCC 12 for CUDA
compilation when GCC 13+ is the system default:

```shell
sudo apt-get install gcc-12 g++-12
cargo build --features cuda
```

## Versions

| Crate version | _OpenSubdiv_ version |
| ------------- | -------------------- |
| `v0.3.x`      | `v3.7.x`             |
| `v0.2.x`      | `v3.5.x`             |
| `v0.1.x`      | `v3.4.x`             |

## Help Wanted

- [Add _DX11_ backend support](https://github.com/virtualritz/opensubdiv-petite/issues/4).
- [Fix _OpenMP_ detection on macOS](https://github.com/virtualritz/opensubdiv-petite/issues/2).
