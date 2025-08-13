use crate::{Error, Result};
use opensubdiv_petite_sys as sys;
use std::convert::TryInto;

/// Concrete vertex buffer class for CPU subdivision.
///
/// [`CpuVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to
/// [`evaluate_stencils()`](crate::osd::cpu_evaluator::evaluate_stencils()).
pub struct CpuVertexBuffer(pub(crate) sys::osd::CpuVertexBufferPtr);

impl Drop for CpuVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::CpuVertexBuffer_destroy(self.0) }
    }
}

impl CpuVertexBuffer {
    #[inline]
    pub fn new(element_count: usize, vertex_count: usize) -> Result<CpuVertexBuffer> {
        let element_count_i32 = element_count
            .try_into()
            .map_err(|_| Error::InvalidBufferSize {
                expected: element_count,
                actual: i32::MAX as usize,
            })?;
        let vertex_count_i32 = vertex_count
            .try_into()
            .map_err(|_| Error::InvalidBufferSize {
                expected: vertex_count,
                actual: i32::MAX as usize,
            })?;

        let ptr = unsafe {
            sys::osd::CpuVertexBuffer_Create(element_count_i32, vertex_count_i32, std::ptr::null())
        };
        if ptr.is_null() {
            return Err(Error::Ffi(
                "CpuVertexBuffer_Create returned null".to_string(),
            ));
        }

        Ok(CpuVertexBuffer(ptr))
    }

    /// Returns how many elements defined in this vertex buffer.
    pub fn element_count(&self) -> usize {
        unsafe { sys::osd::CpuVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        unsafe { sys::osd::CpuVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the contents of this vertex buffer as a slice of [`f32`].
    #[inline]
    pub fn bind_cpu_buffer(&self) -> Result<&[f32]> {
        let ptr = unsafe { sys::osd::CpuVertexBuffer_BindCpuBuffer(self.0) };
        if ptr.is_null() {
            return Err(Error::NullPointer);
        }

        Ok(unsafe { std::slice::from_raw_parts(ptr, self.element_count() * self.vertex_count()) })
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to Osd.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertex_count: usize,
    ) -> Result<()> {
        // do some basic error checking
        let element_count = self.element_count();

        if start_vertex * element_count > src.len() {
            return Err(Error::InvalidBufferSize {
                expected: start_vertex * element_count,
                actual: src.len(),
            });
        }

        if vertex_count * element_count > src.len() {
            return Err(Error::InvalidBufferSize {
                expected: vertex_count * element_count,
                actual: src.len(),
            });
        }

        let start_vertex_i32 = start_vertex
            .try_into()
            .map_err(|_| Error::InvalidBufferSize {
                expected: start_vertex,
                actual: i32::MAX as usize,
            })?;
        let vertex_count_i32 = vertex_count
            .try_into()
            .map_err(|_| Error::InvalidBufferSize {
                expected: vertex_count,
                actual: i32::MAX as usize,
            })?;

        unsafe {
            sys::osd::CpuVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex_i32,
                vertex_count_i32,
                std::ptr::null(),
            );
        }

        Ok(())
    }
}
