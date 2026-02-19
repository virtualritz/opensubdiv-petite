#include <opensubdiv/osd/cudaEvaluator.h>
#include <opensubdiv/osd/cudaVertexBuffer.h>

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::LimitStencilTable LimitStencilTable;
typedef OpenSubdiv::Osd::CudaStencilTable CudaStencilTable;
typedef OpenSubdiv::Osd::CudaEvaluator CudaEvaluator;
typedef OpenSubdiv::Osd::CudaVertexBuffer CudaVertexBuffer;
typedef OpenSubdiv::Osd::BufferDescriptor BufferDescriptor;

// CudaStencilTable
extern "C"
{
    CudaStencilTable *CudaStencilTable_Create(const StencilTable *st)
    {
        return CudaStencilTable::Create(st);
    }

    // CudaStencilTable*
    // CudaStencilTable_CreateFromLimit(const LimitStencilTable* st) {
    //     return CudaStencilTable_Create(st);
    // }

    void CudaStencilTable_destroy(CudaStencilTable *st)
    {
        delete st;
    }
}

// CudaEvaluator
extern "C"
{
    bool CudaEvaluator_EvalStencils(
        CudaVertexBuffer *src_buffer,
        BufferDescriptor src_desc,
        CudaVertexBuffer *dst_buffer,
        BufferDescriptor dst_desc,
        CudaStencilTable *stencil_table)
    {
        return OpenSubdiv::Osd::CudaEvaluator::EvalStencils(
            src_buffer, src_desc, dst_buffer, dst_desc, stencil_table);
    }
}