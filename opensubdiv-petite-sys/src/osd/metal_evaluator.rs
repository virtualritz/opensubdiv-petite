use crate::far::StencilTablePtr;
use crate::osd::BufferDescriptor;
use crate::osd::MetalVertexBufferPtr;
use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MetalStencilTable_obj {
    _unused: [u8; 0],
}
pub type MetalStencilTablePtr = *mut MetalStencilTable_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn MTLStencilTable_Create(
        st: StencilTablePtr,
        context: *const c_void,
    ) -> MetalStencilTablePtr;
    pub fn MTLStencilTable_destroy(st: MetalStencilTablePtr);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MetalComputeEvaluator_obj {
    _unused: [u8; 0],
}
pub type MetalComputeEvaluatorPtr = *mut MetalComputeEvaluator_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn MTLComputeEvaluator_EvalStencils(
        src_buffer: MetalVertexBufferPtr,
        src_desc: BufferDescriptor,
        dst_buffer: MetalVertexBufferPtr,
        dst_desc: BufferDescriptor,
        stencil_table: MetalStencilTablePtr,
        command_buffer: *const c_void,
        compute_encoder: *const c_void,
    ) -> bool;
}
