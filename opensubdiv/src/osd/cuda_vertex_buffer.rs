use opensubdiv_sys as sys;
use std::convert::TryInto;

/// Concrete vertex buffer class for CUDA subdivision.
///
/// [`CudaVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to ///
/// [`evaluate_stencils()`](crate::osd::cuda_evaluator::evaluate_stencils()).
pub struct CudaVertexBuffer(pub(crate) sys::osd::CudaVertexBufferPtr);

impl Drop for CudaVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::CudaVertexBuffer_destroy(self.0) }
    }
}

impl CudaVertexBuffer {
    #[inline]
    pub fn new(elements_len: usize, vertices_len: usize) -> CudaVertexBuffer {
        let ptr = unsafe {
            sys::osd::CudaVertexBuffer_Create(
                elements_len.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                std::ptr::null(),
            )
        };
        if ptr.is_null() {
            panic!("CudaVertexBuffer_Create returned null");
        }

        CudaVertexBuffer(ptr)
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn elements_len(&self) -> usize {
        unsafe { sys::osd::CudaVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertices_len(&self) -> usize {
        unsafe { sys::osd::CudaVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the contents of this vertex buffer as a slice of [`f32`]s.
    #[inline]
    pub fn bind_cuda_buffer(&self) -> &[f32] {
        let ptr = unsafe { sys::osd::CudaVertexBuffer_BindCudaBuffer(self.0) };
        if ptr.is_null() {
            panic!("CudaVertexBuffer_BindCudaBuffer() returned null");
        }

        unsafe {
            std::slice::from_raw_parts(
                ptr,
                (self.elements_len() * self.vertices_len()) as usize,
            )
        }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to *OpenSubdiv*..
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertices_len: usize,
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
            sys::osd::CudaVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                std::ptr::null(),
            );
        }
    }
}
