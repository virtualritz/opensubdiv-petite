use super::buffer_descriptor::BufferDescriptor;
use super::metal_vertex_buffer::MetalVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Generic static eval stencils function for Metal.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// * `srcBuffer` - Input primvar buffer. Must have GetMTLBuffer() method
///   returning a Metal buffer for read
/// * `srcDesc` - vertex buffer descriptor for the input buffer
/// * `dstBuffer` -  Output primvar buffer must have GetMTLBuffer() method
///   returning a Metal buffer for write
/// * `dstDesc` - vertex buffer descriptor for the output buffer
/// * `stencilTable` - [StencilTable] or equivalent
/// * `commandBuffer` - Metal command buffer for execution
/// * `computeEncoder` - Metal compute command encoder
pub fn evaluate_stencils(
    src_buffer: &MetalVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut MetalVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &MetalStencilTable,
    command_buffer: *const std::ffi::c_void,
    compute_encoder: *const std::ffi::c_void,
) -> Result<()> {
    unsafe {
        if sys::osd::MTLComputeEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.ptr,
            command_buffer,
            compute_encoder,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}

pub struct MetalStencilTable<'a> {
    pub(crate) ptr: sys::osd::MetalStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> MetalStencilTable<'a> {
    pub fn new(st: &StencilTable, context: *const std::ffi::c_void) -> MetalStencilTable<'_> {
        let ptr = unsafe { sys::osd::MTLStencilTable_Create(st.0, context) };
        if ptr.is_null() {
            panic!("Could not create MetalStencilTable");
        }

        MetalStencilTable {
            ptr,
            st: std::marker::PhantomData,
        }
    }
}

impl Drop for MetalStencilTable<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::osd::MTLStencilTable_destroy(self.ptr);
        }
    }
}
