#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/virtualritz/opensubdiv/master/osd-logo.png"
)]
//! This is a wrapper around parts of [*Pixar’s
//! OpenSubdiv*](https://graphics.pixar.com/opensubdiv/).
//!
//! *OpenSubdiv* is a set of open source libraries that implement high
//! performance/parallel [subdivision surface](https://en.wikipedia.org/wiki/Subdivision_surface)
//! (subdiv) evaluation CPU and GPU architectures.
//!
//! The code is optimized for drawing deforming surfaces with static topology at
//! interactive framerates.
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
