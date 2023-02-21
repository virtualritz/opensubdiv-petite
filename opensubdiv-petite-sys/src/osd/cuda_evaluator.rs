use crate::far::StencilTablePtr;
use crate::osd::BufferDescriptor;
use crate::osd::CudaVertexBufferPtr;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CudaStencilTable_obj {
    _unused: [u8; 0],
}
pub type CudaStencilTablePtr = *mut CudaStencilTable_obj;

#[link(name = "osl-capi", kind = "static")]
extern "C" {
    pub fn CudaStencilTable_Create(st: StencilTablePtr) -> CudaStencilTablePtr;
    // pub fn CudaStencilTable_CreateFromLimit(st: LimitStencilTablePtr) ->
    // CudaStencilTablePtr;
    pub fn CudaStencilTable_destroy(st: CudaStencilTablePtr);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CudaEvaluator_obj {
    _unused: [u8; 0],
}
pub type CudaEvaluatorPtr = *mut CudaEvaluator_obj;

#[link(name = "osl-capi", kind = "static")]
extern "C" {
    pub fn CudaEvaluator_EvalStencils(
        src_buffer: CudaVertexBufferPtr,
        src_desc: BufferDescriptor,
        dst_buffer: CudaVertexBufferPtr,
        dst_desc: BufferDescriptor,
        stencil_table: CudaStencilTablePtr,
    ) -> bool;
}
