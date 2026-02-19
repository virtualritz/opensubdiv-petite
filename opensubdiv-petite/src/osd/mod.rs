//! # OpenSubdiv
//!
//! `Osd` contains device-dependent code that makes [`far`](crate::far)
//! structures available on various backends such as *TBB*, *CUDA*, *OpenCL*,
//! *GLSL*, etc. The main roles of `osd` are:
//!
//! * **Refinement**<br> Compute stencil-based uniform/adaptive subdivision on
//!   *CPU/GPU* backends.
//!
//! * **Limit Stencil Evaluation**<br> Compute limit surfaces by limit stencils
//!   on *CPU/GPU* backends.
//!
//! * **Limit Evaluation with `PatchTable`**<br> Compute limit surfaces by patch
//!   evaluation on *CPU/GPU* backends.
//!
//! * **OpenGL/DX11/Metal Drawing with Hardware Tessellation**<br> Provide
//!   *GLSL/HLSL/Metal* tessellation functions for patch table.
//!
//! * **Interleaved/Batched Buffer Configuration**<br> Provide consistent buffer
//!   descriptor to deal with arbitrary buffer layout.
//!
//! * **Cross-Platform Implementation**<br> Provide convenient ways to interop
//!   between compute and draw APIs.
//!
//! These are independently used by clients. For example, a client can use only
//! the limit stencil evaluation, or a client can refine subdivision surfaces
//! and draw them with the PatchTable and Osd tessellation shaders. All device
//! specific evaluation kernels are implemented in the Evaluator classes. Since
//! Evaluators don't own vertex buffers, clients should provide their own
//! buffers as a source and destination. There are some interop classes defined
//! in Osd for convenience.
//!
//! OpenSubdiv utilizes a series of regression tests to compare and enforce
//! identical results across different computational devices.
pub mod buffer_descriptor;
pub use buffer_descriptor::*;

pub mod cpu_evaluator;

pub mod cpu_vertex_buffer;
pub use cpu_vertex_buffer::*;

#[cfg(feature = "cuda")]
pub mod cuda_vertex_buffer;
#[cfg(feature = "cuda")]
pub use cuda_vertex_buffer::*;

#[cfg(feature = "cuda")]
pub mod cuda_evaluator;
// Don't use wildcard export to avoid evaluate_stencils name conflicts
#[cfg(feature = "cuda")]
pub use cuda_evaluator::CudaStencilTable;

#[cfg(feature = "metal")]
pub mod metal_vertex_buffer;
#[cfg(feature = "metal")]
pub use metal_vertex_buffer::*;

#[cfg(feature = "metal")]
pub mod metal_evaluator;
// Don't use wildcard export to avoid evaluate_stencils name conflicts
#[cfg(feature = "metal")]
pub use metal_evaluator::MetalStencilTable;

#[cfg(feature = "opencl")]
pub mod opencl_vertex_buffer;
#[cfg(feature = "opencl")]
pub use opencl_vertex_buffer::*;

#[cfg(feature = "opencl")]
pub mod opencl_evaluator;
// Don't use wildcard export to avoid evaluate_stencils name conflicts
#[cfg(feature = "opencl")]
pub use opencl_evaluator::OpenClStencilTable;

#[cfg(feature = "openmp")]
pub mod omp_evaluator;

#[cfg(feature = "tbb")]
pub mod tbb_evaluator;

#[cfg(feature = "wgpu")]
pub mod wgpu;
