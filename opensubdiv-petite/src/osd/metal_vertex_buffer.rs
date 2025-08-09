use opensubdiv_petite_sys as sys;
use std::convert::TryInto;

/// Concrete vertex buffer class for Metal subdivision.
///
/// [`MetalVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to
/// [`evaluate_stencils()`](crate::osd::metal_evaluator::evaluate_stencils()).
pub struct MetalVertexBuffer(pub(crate) sys::osd::MetalVertexBufferPtr);

impl Drop for MetalVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::MTLVertexBuffer_destroy(self.0) }
    }
}

impl MetalVertexBuffer {
    #[inline]
    pub fn new(
        elements_len: usize,
        vertices_len: usize,
        device: *const std::ffi::c_void,
    ) -> MetalVertexBuffer {
        let ptr = unsafe {
            sys::osd::MTLVertexBuffer_Create(
                elements_len.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                device,
            )
        };
        if ptr.is_null() {
            panic!("MTLVertexBuffer_Create returned null");
        }

        MetalVertexBuffer(ptr)
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn elements_len(&self) -> usize {
        unsafe { sys::osd::MTLVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertices_len(&self) -> usize {
        unsafe { sys::osd::MTLVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the Metal buffer object.
    #[inline]
    pub fn get_metal_buffer(&self) -> *const std::ffi::c_void {
        unsafe { sys::osd::MTLVertexBuffer_GetMTLBuffer(self.0) }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to *OpenSubdiv*.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertices_len: usize,
        command_buffer: *const std::ffi::c_void,
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
            sys::osd::MTLVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                command_buffer,
            );
        }
    }
}
