//#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/virtualritz/opensubdiv/master/osd-logo.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Pixar OpenSubdiv Wrapper
//!
//! This is a safe Rust wrapper around parts of [*Pixar's
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
//!   use of abbreviations in some surprising places).
//! * Use canonical Rust naming  – (`num_vertices()` becomes `vertex_count()`).
//! * Use canonically Rust constructs.  Most option/configuration `struct`s use the
//!   [init-`struct` pattern](https://xaeroxe.github.io/init-struct-pattern/). In
//!   places where it’s not possible to easily map to a Rust `struct`, the builder
//!   pattern (or anti-pattern, depending whom you ask) is used.
//! * Be brief when possible. Example: `StencilTable::numStencils()` in C++
//!   becomes `StencilTable::len()` in Rust.
//! * Use unsigned integer types, specifically `usize` and `u32`, instead of
//!   signed ones (`i32`) for anything that can only contain positive values
//!   (indices, sizes/lengths/counts, valences, arities, etc.).  Types should
//!   express intent.  See also
//!   [here](https://github.com/PixarAnimationStudios/OpenSubdiv/issues/1222).
//!
//! ## Cargo Features
#![doc = document_features::document_features!()]
//!
//! ## Versions
//!
//! For now crate versions reflect code maturity on the Rust side. They are not
//! in any way related to the *OpenSubdiv* version that is wrapped.
//!
//! - `v0.3.x` – *OpenSubdiv* `v3.7.x`
//! - `v0.2.x` – *OpenSubdiv* `v3.5.x`
//! - `v0.1.x` – *OpenSubdiv* `v3.4.x`

pub mod bfr;
pub mod error;
pub mod far;
pub mod osd;

// Re-export error types for convenience
pub use error::{Error, Result};

#[cfg(feature = "tri_mesh_buffers")]
pub mod tri_mesh_buffers;

#[cfg(feature = "truck")]
pub mod truck;

pub mod iges_export;
pub mod obj_bspline_export;

/// A vertex, edge, or face index in the topology.
///
/// # Examples
///
/// ```
/// use opensubdiv_petite::Index;
///
/// // Create an index from a u32
/// let idx = Index::from(42u32);
/// assert_eq!(idx.0, 42);
///
/// // Convert back to u32
/// let value: u32 = idx.into();
/// assert_eq!(value, 42);
///
/// // Create from usize
/// let idx = Index::from(100usize);
/// let as_usize: usize = idx.into();
/// assert_eq!(as_usize, 100);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Index(pub u32);

impl From<u32> for Index {
    fn from(value: u32) -> Self {
        Index(value)
    }
}

impl From<Index> for u32 {
    fn from(index: Index) -> Self {
        index.0
    }
}

impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Index(value as u32)
    }
}

impl From<Index> for usize {
    fn from(index: Index) -> Self {
        index.0 as usize
    }
}
