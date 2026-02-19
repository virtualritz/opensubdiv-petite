#include <opensubdiv/osd/cpuVertexBuffer.h>

typedef OpenSubdiv::Osd::CpuVertexBuffer CpuVertexBuffer;

extern "C"
{
    /// Creator. Returns NULL if error.
    CpuVertexBuffer *
    CpuVertexBuffer_Create(int numElements, int numVertices, void *deviceContext)
    {
        return OpenSubdiv::Osd::CpuVertexBuffer::Create(
            numElements, numVertices, deviceContext);
    }

    /// Destructor.
    void CpuVertexBuffer_destroy(CpuVertexBuffer *vb)
    {
        delete vb;
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    void CpuVertexBuffer_UpdateData(
        CpuVertexBuffer *vb,
        const float *src,
        int startVertex,
        int numVertices,
        void *deviceContext)
    {
        vb->UpdateData(src, startVertex, numVertices, deviceContext);
    }

    /// Returns how many elements defined in this vertex buffer.
    int CpuVertexBuffer_GetNumElements(CpuVertexBuffer *vb)
    {
        return vb->GetNumElements();
    }

    /// Returns how many vertices allocated in this vertex buffer.
    int CpuVertexBuffer_GetNumVertices(CpuVertexBuffer *vb)
    {
        return vb->GetNumVertices();
    }

    /// Returns the address of CPU buffer
    float *CpuVertexBuffer_BindCpuBuffer(CpuVertexBuffer *vb)
    {
        return vb->BindCpuBuffer();
    }
}