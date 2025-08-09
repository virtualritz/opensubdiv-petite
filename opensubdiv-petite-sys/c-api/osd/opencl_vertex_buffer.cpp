#include <opensubdiv/osd/clVertexBuffer.h>

typedef OpenSubdiv::Osd::CLVertexBuffer CLVertexBuffer;

extern "C" {
/// Creator. Returns NULL if error.
CLVertexBuffer* CLVertexBuffer_Create(int numElements, int numVertices,
                                      void* clContext) {
    return OpenSubdiv::Osd::CLVertexBuffer::Create(numElements, numVertices,
                                                   clContext);
}

/// Destructor.
void CLVertexBuffer_destroy(CLVertexBuffer* vb) { delete vb; }

/// This method is meant to be used in client code in order to provide
/// coarse vertices data to Osd.
void CLVertexBuffer_UpdateData(CLVertexBuffer* vb, const float* src,
                               int startVertex, int numVertices,
                               void* clCommandQueue) {
    vb->UpdateData(src, startVertex, numVertices, clCommandQueue);
}

/// Returns how many elements defined in this vertex buffer.
int CLVertexBuffer_GetNumElements(CLVertexBuffer* vb) {
    return vb->GetNumElements();
}

/// Returns how many vertices allocated in this vertex buffer.
int CLVertexBuffer_GetNumVertices(CLVertexBuffer* vb) {
    return vb->GetNumVertices();
}

/// Returns the CL buffer object
void* CLVertexBuffer_BindCLBuffer(CLVertexBuffer* vb, void* clCommandQueue) {
    return vb->BindCLBuffer(clCommandQueue);
}
}