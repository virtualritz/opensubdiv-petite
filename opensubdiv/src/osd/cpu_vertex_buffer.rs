use opensubdiv_sys as sys;
use std::convert::TryInto;

/// Concrete vertex buffer class for CPU subdivision.
///
/// [`CpuVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to
/// [`eval_stencils()`](crate::osd::cpu_evaluator::eval_stencils()).
pub struct CpuVertexBuffer(pub(crate) sys::osd::CpuVertexBufferPtr);

impl Drop for CpuVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::CpuVertexBuffer_destroy(self.0) }
    }
}

impl CpuVertexBuffer {
    #[inline]
    pub fn new(len_elements: u32, len_vertices: u32) -> CpuVertexBuffer {
        let ptr = unsafe {
            sys::osd::CpuVertexBuffer_Create(
                len_elements.try_into().unwrap(),
                len_vertices.try_into().unwrap(),
                std::ptr::null(),
            )
        };
        if ptr.is_null() {
            panic!("CpuVertexBuffer_Create returned null");
        }

        CpuVertexBuffer(ptr)
    }

    /// Returns how many elements defined in this vertex buffer.
    pub fn len_elements(&self) -> u32 {
        unsafe { sys::osd::CpuVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn len_vertices(&self) -> u32 {
        unsafe { sys::osd::CpuVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the contents of this vertex buffer as a slice of [`f32`].
    #[inline]
    pub fn bind_cpu_buffer(&self) -> &[f32] {
        let ptr = unsafe { sys::osd::CpuVertexBuffer_BindCpuBuffer(self.0) };
        if ptr.is_null() {
            panic!("CpuVertexBuffer_BindCpuBuffer() returned null");
        }

        unsafe {
            std::slice::from_raw_parts(
                ptr,
                (self.len_elements() * self.len_vertices()) as usize,
            )
        }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: u32,
        len_vertices: u32,
    ) {
        // do some basic error checking
        let len_elements = self.len_elements();

        if (start_vertex * len_elements) as usize > src.len() {
            panic!(
                "Start vertex is out of range of the src slice: {} ({})",
                start_vertex,
                src.len()
            );
        }

        if (len_vertices * len_elements) as usize > src.len() {
            panic!(
                "num vertices is out of range of the src slice: {} ({})",
                len_vertices,
                src.len()
            );
        }

        unsafe {
            sys::osd::CpuVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex.try_into().unwrap(),
                len_vertices.try_into().unwrap(),
                std::ptr::null(),
            );
        }
    }
}
