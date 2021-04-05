use super::buffer_descriptor::BufferDescriptor;
use super::cuda_vertex_buffer::CudaVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Generic static eval stencils function.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// * `srcBuffer` - Input primvar buffer. Must have BindCudaBuffer() method
/// returning a const float pointer for read
/// * `srcDesc` - vertex buffer descriptor for the input buffer
/// * `dstBuffer` -  Output primvar buffer must have BindCudaBuffer() method
/// returning a float pointer for write
/// * `dstDesc` - vertex buffer descriptor for the output buffer
/// * `stencilTable` - [StencilTable] or equivalent
pub fn evaluate_stencils(
    src_buffer: &CudaVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut CudaVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &CudaStencilTable,
) -> Result<()> {
    unsafe {
        if sys::osd::CudaEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.ptr,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}

pub struct CudaStencilTable<'a> {
    pub(crate) ptr: sys::osd::CudaStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> CudaStencilTable<'a> {
    pub fn new(st: &StencilTable) -> CudaStencilTable {
        let ptr = unsafe { sys::osd::CudaStencilTable_Create(st.0) };
        if ptr.is_null() {
            panic!("Could not create CudaStencilTable");
        }

        CudaStencilTable {
            ptr,
            st: std::marker::PhantomData,
        }
    }
}
