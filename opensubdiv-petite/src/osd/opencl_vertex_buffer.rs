use opensubdiv_petite_sys as sys;
use std::convert::TryInto;

/// Concrete vertex buffer class for OpenCL subdivision.
///
/// [`OpenCLVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to
/// [`evaluate_stencils()`](crate::osd::opencl_evaluator::evaluate_stencils()).
pub struct OpenCLVertexBuffer(pub(crate) sys::osd::OpenCLVertexBufferPtr);

impl Drop for OpenCLVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::CLVertexBuffer_destroy(self.0) }
    }
}

impl OpenCLVertexBuffer {
    #[inline]
    pub fn new(
        elements_len: usize,
        vertices_len: usize,
        cl_context: *const std::ffi::c_void,
    ) -> OpenCLVertexBuffer {
        let ptr = unsafe {
            sys::osd::CLVertexBuffer_Create(
                elements_len.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                cl_context,
            )
        };
        if ptr.is_null() {
            panic!("CLVertexBuffer_Create returned null");
        }

        OpenCLVertexBuffer(ptr)
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn elements_len(&self) -> usize {
        unsafe { sys::osd::CLVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertices_len(&self) -> usize {
        unsafe { sys::osd::CLVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the OpenCL buffer object.
    #[inline]
    pub fn bind_cl_buffer(
        &self,
        cl_command_queue: *const std::ffi::c_void,
    ) -> *const std::ffi::c_void {
        unsafe { sys::osd::CLVertexBuffer_BindCLBuffer(self.0, cl_command_queue) }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to *OpenSubdiv*.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertices_len: usize,
        cl_command_queue: *const std::ffi::c_void,
    ) {
        // do some basic error checking
        let elements_len = self.elements_len();

        if start_vertex * elements_len > src.len() {
            panic!(
                "Start vertex is out of range of the src slice: {} ({})",
                start_vertex,
                src.len()
            );
        }

        if vertices_len * elements_len > src.len() {
            panic!(
                "vertices_len is out of range of the src slice: {} ({})",
                vertices_len,
                src.len()
            );
        }

        unsafe {
            sys::osd::CLVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                cl_command_queue,
            );
        }
    }
}
