#include <opensubdiv/far/stencilTable.h>
#include <opensubdiv/osd/cpuVertexBuffer.h>
#include <opensubdiv/osd/tbbEvaluator.h>

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Osd::BufferDescriptor BufferDescriptor;
typedef OpenSubdiv::Osd::CpuVertexBuffer CpuVertexBuffer;

extern "C"
{
    bool TbbEvaluator_EvalStencils(
        CpuVertexBuffer *src_buffer,
        BufferDescriptor src_desc,
        CpuVertexBuffer *dst_buffer,
        BufferDescriptor dst_desc,
        StencilTable *stencil_table)
    {
        return OpenSubdiv::Osd::TbbEvaluator::EvalStencils(
            src_buffer, src_desc, dst_buffer, dst_desc, stencil_table);
    }
}
