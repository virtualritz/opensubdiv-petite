#ifdef __APPLE__
#include <opensubdiv/osd/mtlVertexBuffer.h>

typedef OpenSubdiv::Osd::MTLVertexBuffer MTLVertexBuffer;

extern "C"
{
    /// Creator. Returns NULL if error.
    MTLVertexBuffer *
    MTLVertexBuffer_Create(int numElements, int numVertices, void *device)
    {
        return OpenSubdiv::Osd::MTLVertexBuffer::Create(
            numElements, numVertices, device);
    }

    /// Destructor.
    void MTLVertexBuffer_destroy(MTLVertexBuffer *vb)
    {
        delete vb;
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    void MTLVertexBuffer_UpdateData(
        MTLVertexBuffer *vb,
        const float *src,
        int startVertex,
        int numVertices,
        void *commandBuffer)
    {
        vb->UpdateData(src, startVertex, numVertices, commandBuffer);
    }

    /// Returns how many elements defined in this vertex buffer.
    int MTLVertexBuffer_GetNumElements(MTLVertexBuffer *vb)
    {
        return vb->GetNumElements();
    }

    /// Returns how many vertices allocated in this vertex buffer.
    int MTLVertexBuffer_GetNumVertices(MTLVertexBuffer *vb)
    {
        return vb->GetNumVertices();
    }

    /// Returns the MTL buffer
    void *MTLVertexBuffer_GetMTLBuffer(MTLVertexBuffer *vb)
    {
        return vb->GetMTLBuffer();
    }
}
#endif  // __APPLE__