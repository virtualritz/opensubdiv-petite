#include <opensubdiv/osd/clComputeContext.h>
#include <opensubdiv/osd/clEvaluator.h>
#include <opensubdiv/osd/clVertexBuffer.h>

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::LimitStencilTable LimitStencilTable;
typedef OpenSubdiv::Osd::CLStencilTable CLStencilTable;
typedef OpenSubdiv::Osd::CLEvaluator CLEvaluator;
typedef OpenSubdiv::Osd::CLVertexBuffer CLVertexBuffer;
typedef OpenSubdiv::Osd::BufferDescriptor BufferDescriptor;

// CLStencilTable
extern "C" {
CLStencilTable* CLStencilTable_Create(const StencilTable* st, void* clContext) {
    return CLStencilTable::Create(st, clContext);
}

void CLStencilTable_destroy(CLStencilTable* st) { delete st; }
}

// CLEvaluator
extern "C" {
bool CLEvaluator_EvalStencils(CLVertexBuffer* src_buffer,
                              BufferDescriptor src_desc,
                              CLVertexBuffer* dst_buffer,
                              BufferDescriptor dst_desc,
                              CLStencilTable* stencil_table,
                              void* kernel,
                              void* command_queue) {
    return OpenSubdiv::Osd::CLEvaluator::EvalStencils(
        src_buffer, src_desc, dst_buffer, dst_desc, stencil_table, 
        kernel, command_queue);
}
}