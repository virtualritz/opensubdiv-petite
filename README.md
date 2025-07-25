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

# Build with recommended compiler on Linux (clang-17)
just build-linux-clang17

# Run tests
just test

# Build documentation
just doc
```

See the `justfile` for all available commands including release builds, cleaning, and running examples.

## Documentation

It is suggested you only build (and look at) the documentation of the high level
wrapper:

```shell
cargo doc -p opensubdiv --no-deps --open
```

## Help Wanted

This is an early release. None of the GPU acceleration backends are yet exposed
on the Rust side.

Specifically (in no particular order) these are issue for which you can put your
hand up or just open a PR:

* [ ] Add support for the *CUDA* backend/[ensure *CUDA* code works](https://github.com/virtualritz/opensubdiv-petite/issues/6).
* [ ] [Add support for the *DX11* backend](https://github.com/virtualritz/opensubdiv-petite/issues/4).
* [ ] [Add support for the *Metal* backend](https://github.com/virtualritz/opensubdiv-petite/issues/3).
* [ ] [Fix *OpenMP* detection on macOS](https://github.com/virtualritz/opensubdiv-petite/issues/2).
* [ ] [Fix `StencilTable`](https://github.com/virtualritz/opensubdiv-petite/issues/1).
* [ ] [Add `PatchTable`](https://github.com/virtualritz/opensubdiv-petite/issues/5).

## Versions

For now crate versions reflect code maturity on the Rust side. They are not in
any way related to the *OpenSubdiv* version that is wrapped.

- `opensubdiv-petite[-sys] v0.3.x` – *OpenSubdiv* `v3.6.x`
- `opensubdiv-petite[-sys] v0.2.x` – *OpenSubdiv* `v3.5.x`
- `opensubdiv-petite[-sys] v0.1.x` – *OpenSubdiv* `v3.4.x`
