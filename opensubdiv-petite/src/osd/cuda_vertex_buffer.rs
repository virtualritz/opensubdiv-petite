use crate::{Error, Result};
use opensubdiv_petite_sys as sys;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Safe wrapper for CUDA context.
#[derive(Debug)]
pub struct CudaContext<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> CudaContext<'a> {
    /// Create a new CUDA context wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<CudaContext<'a>> {
        NonNull::new(ptr).map(|ptr| CudaContext {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

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
    /// Create a new CUDA vertex buffer.
    #[inline]
    pub fn new(
        element_count: usize,
        vertex_count: usize,
        context: Option<&CudaContext>,
    ) -> Result<CudaVertexBuffer> {
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
            sys::osd::CudaVertexBuffer_Create(
                element_count_i32,
                vertex_count_i32,
                context.map_or(std::ptr::null(), |ctx| ctx.as_ptr() as *const _),
            )
        };
        if ptr.is_null() {
            return Err(Error::GpuBackend(
                "Failed to create CUDA vertex buffer".to_string(),
            ));
        }

        Ok(CudaVertexBuffer(ptr))
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn element_count(&self) -> usize {
        unsafe { sys::osd::CudaVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        unsafe { sys::osd::CudaVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the contents of this vertex buffer as a slice of [`f32`]s.
    #[inline]
    pub fn bind_cuda_buffer(&self) -> Result<&[f32]> {
        let ptr = unsafe { sys::osd::CudaVertexBuffer_BindCudaBuffer(self.0) };
        if ptr.is_null() {
            return Err(Error::NullPointer);
        }

        Ok(unsafe { std::slice::from_raw_parts(ptr, self.element_count() * self.vertex_count()) })
    }

    /// Update vertex data with a strongly-typed slice.
    ///
    /// Users can use bytemuck to cast flat arrays to the required format if
    /// needed.
    ///
    /// # Parameters
    /// - `vertices`: Slice of vertex data where each vertex has `N` elements.
    /// - `start_vertex`: Starting vertex index to update.
    /// - `context`: Optional [`CudaContext`] for the operation.
    ///
    /// # Errors
    /// Returns error if `N` doesn't match the buffer's `element_count` or if
    /// indices are out of bounds.
    #[inline]
    pub fn update_data<const N: usize>(
        &mut self,
        vertices: &[[f32; N]],
        start_vertex: usize,
        context: Option<&CudaContext>,
    ) -> Result<()> {
        let element_count = self.element_count();

        // Verify that N matches the buffer's element size
        if N != element_count {
            return Err(Error::InvalidBufferSize {
                expected: element_count,
                actual: N,
            });
        }

        let vertex_count = vertices.len();
        let total_vertices = self.vertex_count();

        // Check bounds
        if start_vertex + vertex_count > total_vertices {
            return Err(Error::IndexOutOfBounds {
                index: start_vertex + vertex_count,
                max: total_vertices,
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
            // Cast the slice to a flat f32 pointer
            let src_ptr = vertices.as_ptr() as *const f32;

            sys::osd::CudaVertexBuffer_UpdateData(
                self.0,
                src_ptr,
                start_vertex_i32,
                vertex_count_i32,
                context.map_or(std::ptr::null(), |ctx| ctx.as_ptr() as *const _),
            );
        }

        Ok(())
    }
}
