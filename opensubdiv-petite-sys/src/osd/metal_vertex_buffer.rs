use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MetalVertexBuffer_obj {
    _unused: [u8; 0],
}
pub type MetalVertexBufferPtr = *mut MetalVertexBuffer_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    /// Creator. Returns NULL if error.
    pub fn MTLVertexBuffer_Create(
        num_elements: i32,
        num_vertices: i32,
        device: *const c_void,
    ) -> MetalVertexBufferPtr;
    /// Destructor.
    pub fn MTLVertexBuffer_destroy(vb: MetalVertexBufferPtr);
    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    pub fn MTLVertexBuffer_UpdateData(
        vb: MetalVertexBufferPtr,
        src: *const f32,
        start_vertex: i32,
        num_vertices: i32,
        command_buffer: *const c_void,
    );
    /// Returns how many elements defined in this vertex buffer.
    pub fn MTLVertexBuffer_GetNumElements(vb: MetalVertexBufferPtr) -> i32;
    /// Returns how many vertices allocated in this vertex buffer.
    pub fn MTLVertexBuffer_GetNumVertices(vb: MetalVertexBufferPtr) -> i32;
    /// Returns the Metal buffer object
    pub fn MTLVertexBuffer_GetMTLBuffer(vb: MetalVertexBufferPtr) -> *const c_void;
}
