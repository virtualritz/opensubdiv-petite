use super::buffer_descriptor::BufferDescriptor;
use super::opencl_vertex_buffer::OpenCLVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Generic static eval stencils function for OpenCL.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// * `srcBuffer` - Input primvar buffer. Must have BindCLBuffer() method
///   returning an OpenCL buffer for read
/// * `srcDesc` - vertex buffer descriptor for the input buffer
/// * `dstBuffer` -  Output primvar buffer must have BindCLBuffer() method
///   returning an OpenCL buffer for write
/// * `dstDesc` - vertex buffer descriptor for the output buffer
/// * `stencilTable` - [StencilTable] or equivalent
/// * `kernel` - OpenCL kernel for evaluation
/// * `commandQueue` - OpenCL command queue
pub fn evaluate_stencils(
    src_buffer: &OpenCLVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut OpenCLVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &OpenCLStencilTable,
    kernel: *const std::ffi::c_void,
    command_queue: *const std::ffi::c_void,
) -> Result<()> {
    unsafe {
        if sys::osd::CLEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.ptr,
            kernel,
            command_queue,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}

pub struct OpenCLStencilTable<'a> {
    pub(crate) ptr: sys::osd::OpenCLStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> OpenCLStencilTable<'a> {
    pub fn new(st: &StencilTable, cl_context: *const std::ffi::c_void) -> OpenCLStencilTable<'_> {
        let ptr = unsafe { sys::osd::CLStencilTable_Create(st.0, cl_context) };
        if ptr.is_null() {
            panic!("Could not create OpenCLStencilTable");
        }

        OpenCLStencilTable {
            ptr,
            st: std::marker::PhantomData,
        }
    }
}

impl Drop for OpenCLStencilTable<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::osd::CLStencilTable_destroy(self.ptr);
        }
    }
}
