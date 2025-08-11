use crate::far::StencilTablePtr;
use crate::osd::BufferDescriptor;
use crate::osd::OpenCLVertexBufferPtr;
use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenCLStencilTable_obj {
    _unused: [u8; 0],
}
pub type OpenCLStencilTablePtr = *mut OpenCLStencilTable_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn CLStencilTable_Create(
        st: StencilTablePtr,
        cl_context: *const c_void,
    ) -> OpenCLStencilTablePtr;
    pub fn CLStencilTable_destroy(st: OpenCLStencilTablePtr);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenCLEvaluator_obj {
    _unused: [u8; 0],
}
pub type OpenCLEvaluatorPtr = *mut OpenCLEvaluator_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn CLEvaluator_EvalStencils(
        src_buffer: OpenCLVertexBufferPtr,
        src_desc: BufferDescriptor,
        dst_buffer: OpenCLVertexBufferPtr,
        dst_desc: BufferDescriptor,
        stencil_table: OpenCLStencilTablePtr,
        kernel: *const c_void,
        command_queue: *const c_void,
    ) -> bool;
}
