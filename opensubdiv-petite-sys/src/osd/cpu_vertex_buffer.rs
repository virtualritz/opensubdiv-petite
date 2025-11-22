use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuVertexBuffer_obj {
    _unused: [u8; 0],
}
pub type CpuVertexBufferPtr = *mut CpuVertexBuffer_obj;

#[link(name = "osd-capi", kind = "static")]
unsafe extern "C" {
    /// Creator. Returns NULL if error.
    pub fn CpuVertexBuffer_Create(
        num_elements: i32,
        num_vertices: i32,
        device_context: *const c_void,
    ) -> CpuVertexBufferPtr;
    /// Destructor.
    pub fn CpuVertexBuffer_destroy(vb: CpuVertexBufferPtr);
    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    pub fn CpuVertexBuffer_UpdateData(
        vb: CpuVertexBufferPtr,
        src: *const f32,
        start_vertex: i32,
        num_vertices: i32,
        device_context: *const c_void,
    );
    /// Returns how many elements defined in this vertex buffer.
    pub fn CpuVertexBuffer_GetNumElements(vb: CpuVertexBufferPtr) -> i32;
    /// Returns how many vertices allocated in this vertex buffer.
    pub fn CpuVertexBuffer_GetNumVertices(vb: CpuVertexBufferPtr) -> i32;
    /// Returns the address of CPU buffer
    pub fn CpuVertexBuffer_BindCpuBuffer(vb: CpuVertexBufferPtr) -> *const f32;

}
