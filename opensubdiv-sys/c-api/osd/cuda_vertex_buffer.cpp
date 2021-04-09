#include <opensubdiv/osd/cudaVertexBuffer.h>

typedef OpenSubdiv::Osd::CudaVertexBuffer CudaVertexBuffer;

extern "C" {
/// Creator. Returns NULL if error.
CudaVertexBuffer* CudaVertexBuffer_Create(int numElements, int numVertices,
                                          void* deviceContext) {
    return OpenSubdiv::Osd::CudaVertexBuffer::Create(numElements, numVertices,
                                                     deviceContext);
}

/// Destructor.
void CudaVertexBuffer_destroy(CudaVertexBuffer* vb) { delete vb; }

/// This method is meant to be used in client code in order to provide
/// coarse vertices data to Osd.
void CudaVertexBuffer_UpdateData(CudaVertexBuffer* vb, const float* src,
                                 int startVertex, int numVertices,
                                 void* deviceContext) {
    vb->UpdateData(src, startVertex, numVertices, deviceContext);
}

/// Returns how many elements defined in this vertex buffer.
int CudaVertexBuffer_GetNumElements(CudaVertexBuffer* vb) {
    return vb->GetNumElements();
}

/// Returns how many vertices allocated in this vertex buffer.
int CudaVertexBuffer_GetNumVertices(CudaVertexBuffer* vb) {
    return vb->GetNumVertices();
}

/// Returns the address of CPU buffer
float* CudaVertexBuffer_BindCudaBuffer(CudaVertexBuffer* vb) {
    return vb->BindCudaBuffer();
}
}
