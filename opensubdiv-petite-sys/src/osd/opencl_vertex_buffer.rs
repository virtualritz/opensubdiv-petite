use std::os::raw::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenCLVertexBuffer_obj {
    _unused: [u8; 0],
}
pub type OpenCLVertexBufferPtr = *mut OpenCLVertexBuffer_obj;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    /// Creator. Returns NULL if error.
    pub fn CLVertexBuffer_Create(
        num_elements: i32,
        num_vertices: i32,
        cl_context: *const c_void,
    ) -> OpenCLVertexBufferPtr;
    /// Destructor.
    pub fn CLVertexBuffer_destroy(vb: OpenCLVertexBufferPtr);
    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    pub fn CLVertexBuffer_UpdateData(
        vb: OpenCLVertexBufferPtr,
        src: *const f32,
        start_vertex: i32,
        num_vertices: i32,
        cl_command_queue: *const c_void,
    );
    /// Returns how many elements defined in this vertex buffer.
    pub fn CLVertexBuffer_GetNumElements(vb: OpenCLVertexBufferPtr) -> i32;
    /// Returns how many vertices allocated in this vertex buffer.
    pub fn CLVertexBuffer_GetNumVertices(vb: OpenCLVertexBufferPtr) -> i32;
    /// Returns the OpenCL buffer object
    pub fn CLVertexBuffer_BindCLBuffer(
        vb: OpenCLVertexBufferPtr,
        cl_command_queue: *const c_void,
    ) -> *const c_void;
}
