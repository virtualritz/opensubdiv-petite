use super::buffer_descriptor::BufferDescriptor;
use super::cuda_vertex_buffer::CudaVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Generic static eval stencils function.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// # Arguments
///
/// * `src_buffer` - Input primvar buffer. Must have BindCudaBuffer() method
///   returning a const float pointer for read.
/// * `src_desc` - Vertex buffer descriptor for the input buffer.
/// * `dst_buffer` - Output primvar buffer. Must have BindCudaBuffer() method
///   returning a float pointer for write.
/// * `dst_desc` - Vertex buffer descriptor for the output buffer.
/// * `stencil_table` - StencilTable or equivalent.
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

/// CUDA-specific stencil table for GPU evaluation.
///
/// This wraps a [`StencilTable`] for use with CUDA GPU evaluation.
/// The lifetime parameter ensures the underlying stencil table outlives this
/// wrapper.
pub struct CudaStencilTable<'a> {
    pub(crate) ptr: sys::osd::CudaStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> CudaStencilTable<'a> {
    /// Create a new CUDA stencil table from a [`StencilTable`].
    ///
    /// # Errors
    ///
    /// Returns an error if the CUDA stencil table creation fails.
    pub fn new(st: &StencilTable) -> crate::Result<CudaStencilTable<'_>> {
        let ptr = unsafe { sys::osd::CudaStencilTable_Create(st.0) };
        if ptr.is_null() {
            return Err(crate::Error::GpuBackend(
                "Could not create CudaStencilTable".to_string(),
            ));
        }

        Ok(CudaStencilTable {
            ptr,
            st: std::marker::PhantomData,
        })
    }
}
