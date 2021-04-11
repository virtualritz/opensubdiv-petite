use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CudaVertexBuffer_obj {
    _unused: [u8; 0],
}
pub type CudaVertexBufferPtr = *mut CudaVertexBuffer_obj;

extern "C" {
    /// Creator. Returns NULL if error.
    pub fn CudaVertexBuffer_Create(
        num_elements: i32,
        num_vertices: i32,
        device_context: *const c_void,
    ) -> CudaVertexBufferPtr;
    /// Destructor.
    pub fn CudaVertexBuffer_destroy(vb: CudaVertexBufferPtr);
    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    pub fn CudaVertexBuffer_UpdateData(
        vb: CudaVertexBufferPtr,
        src: *const f32,
        start_vertex: i32,
        num_vertices: i32,
        device_context: *const c_void,
    );
    /// Returns how many elements defined in this vertex buffer.
    pub fn CudaVertexBuffer_GetNumElements(vb: CudaVertexBufferPtr) -> i32;
    /// Returns how many vertices allocated in this vertex buffer.
    pub fn CudaVertexBuffer_GetNumVertices(vb: CudaVertexBufferPtr) -> i32;
    /// Returns the address of CPU buffer
    pub fn CudaVertexBuffer_BindCudaBuffer(
        vb: CudaVertexBufferPtr,
    ) -> *const f32;

}
