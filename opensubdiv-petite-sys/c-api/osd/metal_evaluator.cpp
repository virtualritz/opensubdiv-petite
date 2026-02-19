#ifdef __APPLE__
#include <opensubdiv/osd/mtlComputeEvaluator.h>
#include <opensubdiv/osd/mtlVertexBuffer.h>

typedef OpenSubdiv::Far::StencilTable StencilTable;
typedef OpenSubdiv::Far::LimitStencilTable LimitStencilTable;
typedef OpenSubdiv::Osd::MTLStencilTable MTLStencilTable;
typedef OpenSubdiv::Osd::MTLComputeEvaluator MTLComputeEvaluator;
typedef OpenSubdiv::Osd::MTLVertexBuffer MTLVertexBuffer;
typedef OpenSubdiv::Osd::BufferDescriptor BufferDescriptor;

// MTLStencilTable
extern "C"
{
    MTLStencilTable *MTLStencilTable_Create(const StencilTable *st, void *context)
    {
        return MTLStencilTable::Create(st, context);
    }

    void MTLStencilTable_destroy(MTLStencilTable *st)
    {
        delete st;
    }
}

// MTLComputeEvaluator
extern "C"
{
    bool MTLComputeEvaluator_EvalStencils(
        MTLVertexBuffer *src_buffer,
        BufferDescriptor src_desc,
        MTLVertexBuffer *dst_buffer,
        BufferDescriptor dst_desc,
        MTLStencilTable *stencil_table,
        void *command_buffer,
        void *compute_encoder)
    {
        return OpenSubdiv::Osd::MTLComputeEvaluator::EvalStencils(
            src_buffer, src_desc, dst_buffer, dst_desc, stencil_table, command_buffer,
            compute_encoder);
    }
}
#endif  // __APPLE__