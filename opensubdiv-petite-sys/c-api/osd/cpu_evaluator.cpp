#include <opensubdiv/far/stencilTable.h>
#include <opensubdiv/osd/cpuEvaluator.h>
#include <opensubdiv/osd/cpuVertexBuffer.h>

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Osd::BufferDescriptor BufferDescriptor;
typedef OpenSubdiv::Osd::CpuVertexBuffer CpuVertexBuffer;

extern "C" {
bool CpuEvaluator_EvalStencils(CpuVertexBuffer* src_buffer,
                               BufferDescriptor src_desc,
                               CpuVertexBuffer* dst_buffer,
                               BufferDescriptor dst_desc,
                               StencilTable* stencil_table) {
    return OpenSubdiv::Osd::CpuEvaluator::EvalStencils(
        src_buffer, src_desc, dst_buffer, dst_desc, stencil_table);
}
}