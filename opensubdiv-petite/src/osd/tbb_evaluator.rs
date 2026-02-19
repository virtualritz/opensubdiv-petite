use super::buffer_descriptor::BufferDescriptor;
use super::cpu_vertex_buffer::CpuVertexBuffer;
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Evaluate stencils using Intel TBB for CPU parallelism.
///
/// This is a drop-in replacement for
/// [`super::cpu_evaluator::evaluate_stencils`] that uses TBB `parallel_for`
/// internally. It operates on the same [`CpuVertexBuffer`] type --- no separate
/// vertex buffer is needed.
///
/// * `src_buffer` -- Input primvar buffer.
/// * `src_desc` -- Vertex buffer descriptor for the input buffer.
/// * `dst_buffer` -- Output primvar buffer.
/// * `dst_desc` -- Vertex buffer descriptor for the output buffer.
/// * `stencil_table` -- A [`StencilTable`].
pub fn evaluate_stencils(
    src_buffer: &CpuVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut CpuVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &StencilTable,
) -> Result<()> {
    unsafe {
        if sys::osd::TbbEvaluator_EvalStencils(
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
