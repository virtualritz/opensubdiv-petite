use super::buffer_descriptor::BufferDescriptor;
use super::cpu_vertex_buffer::CpuVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Generic static eval stencils function.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// * `srcBuffer` - Input primvar buffer. Must have BindCpuBuffer() method
/// returning a const float pointer for read
/// * `srcDesc` - vertex buffer descriptor for the input buffer
/// * `dstBuffer` -  Output primvar buffer must have BindCpuBuffer() method
/// returning a float pointer for write
/// * `dstDesc` - vertex buffer descriptor for the output buffer
/// * `stencilTable` - [StencilTable] or equivalent
pub fn eval_stencils(
    src_buffer: &CpuVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut CpuVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &StencilTable,
) -> Result<()> {
    unsafe {
        if sys::osd::CpuEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.0,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}
