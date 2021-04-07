#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/virtualritz/opensubdiv/master/osd-logo.png"
)]
//! # Pixar OpenSubdiv Wrapper
//!
//! This is an oxidized wrapper around parts of [*Pixar’s
//! OpenSubdiv*](https://graphics.pixar.com/opensubdiv/).
//!
//! *OpenSubdiv* is a set of open source libraries that implement high
//! performance/parallel [subdivision surface](https://en.wikipedia.org/wiki/Subdivision_surface)
//! (subdiv) evaluation on CPU and GPU architectures.
//!
//! The code is optimized for drawing deforming surfaces with static topology at
//! interactive framerates.
//!
//! ## Limitations
//!
//! The original library does make use of templates in quite a few places.
//! The wrapper has specializations that cover the most common use case.
//!
//! C++ factory classes have been collapsed into the `new()` method of the resp.
//! struct that mirrors the class the C++ factory was building.
//!
//! ## API Changes From C++
//!
//! Many methods have slightly different names on the Rust side.
//!
//! Renaming was done considering these constraints:
//! * Be verbose consistently (the original API is quite verbose but does make
//!   use of abbreviations in some suprising places).
//! * Use canonical Rust naming (`num_vertices()` becomes `vertices_len()`).
//! * Use canonical Rust constructs (e.g. the builder pattern – or anti-pattern,
//!   depending whom you ask). I will probably switch this to an [init struct pattern](https://xaeroxe.github.io/init-struct-pattern/)
//!   soon.  Even though this means a minimal overhead for some structs which
//!   are better left for `bindgen` to define and then require copying.
//! * Be brief when possible. Example: `StencilTable::numStencils()` in C++
//!   becomes `StencilTable::len()` in Rust.
//! * Use usnigned integer types, specifically `u32`, instead of signed ones
//!   (`i32`) for anything that can only contain positive values (indices,
//!   sizes/lengths/counts, valences, arities, etc.). Types should express
//!   intent.  See also
//!   [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).
pub mod far;
pub mod osd;

pub use opensubdiv_sys::vtr::Index;

#[macro_use]
extern crate derive_more;

#[derive(Display, Debug)]
pub enum Error {
    #[display(fmt = "Failed to create TopologyRefiner")]
    CreateTopologyRefinerFailed,
    #[display(fmt = "Stencil evaluation failed")]
    EvalStencilsFailed,
}
