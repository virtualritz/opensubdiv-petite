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

## Documentation

It is suggested you only build (and look at) the documentation of the high level
wrapper:

```shell
cargo doc -p opensubdiv --no-deps --open
```
## Versions

- `v0.1.x` – **OpenSubdiv `v3.4.4`**
