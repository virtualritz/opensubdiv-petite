use crate::far::StencilTablePtr;
use crate::osd::BufferDescriptor;
use crate::osd::CpuVertexBufferPtr;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TbbEvaluator_obj {
    _unused: [u8; 0],
}
pub type TbbEvaluatorPtr = *mut TbbEvaluator_obj;

#[link(name = "osd-capi", kind = "static")]
unsafe extern "C" {
    pub fn TbbEvaluator_EvalStencils(
        src_buffer: CpuVertexBufferPtr,
        src_desc: BufferDescriptor,
        dst_buffer: CpuVertexBufferPtr,
        dst_desc: BufferDescriptor,
        stencil_table: StencilTablePtr,
    ) -> bool;
}
