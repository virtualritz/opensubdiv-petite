use crate::far::StencilTablePtr;
use crate::osd::BufferDescriptor;
use crate::osd::CpuVertexBufferPtr;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuEvaluator_obj {
    _unused: [u8; 0],
}
pub type CpuEvaluatorPtr = *mut CpuEvaluator_obj;

extern "C" {
    pub fn CpuEvaluator_EvalStencils(
        src_buffer: CpuVertexBufferPtr,
        src_desc: BufferDescriptor,
        dst_buffer: CpuVertexBufferPtr,
        dst_desc: BufferDescriptor,
        stencil_table: StencilTablePtr,
    ) -> bool;
}
