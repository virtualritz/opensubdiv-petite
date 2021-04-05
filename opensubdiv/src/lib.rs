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
