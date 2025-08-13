use crate::{Error, Result};
use opensubdiv_petite_sys as sys;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Safe wrapper for Metal device.
#[derive(Debug)]
pub struct MetalDevice<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> MetalDevice<'a> {
    /// Create a new Metal device wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<MetalDevice<'a>> {
        NonNull::new(ptr).map(|ptr| MetalDevice {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Safe wrapper for Metal command buffer.
#[derive(Debug)]
pub struct MetalCommandBuffer<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> MetalCommandBuffer<'a> {
    /// Create a new Metal command buffer wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<MetalCommandBuffer<'a>> {
        NonNull::new(ptr).map(|ptr| MetalCommandBuffer {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

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
    /// Create a new Metal vertex buffer.
    #[inline]
    pub fn new(
        element_count: usize,
        vertex_count: usize,
        device: Option<&MetalDevice>,
    ) -> Result<MetalVertexBuffer> {
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
            sys::osd::MTLVertexBuffer_Create(
                element_count_i32,
                vertex_count_i32,
                device.map_or(std::ptr::null(), |d| d.as_ptr() as *const _),
            )
        };
        if ptr.is_null() {
            return Err(Error::GpuBackend(
                "MTLVertexBuffer_Create returned null".to_string(),
            ));
        }

        Ok(MetalVertexBuffer(ptr))
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn element_count(&self) -> usize {
        unsafe { sys::osd::MTLVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertex_count(&self) -> usize {
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
        vertex_count: usize,
        command_buffer: &MetalCommandBuffer,
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

        unsafe {
            sys::osd::MTLVertexBuffer_UpdateData(
                self.0,
                src.as_ptr(),
                start_vertex
                    .try_into()
                    .map_err(|_| Error::InvalidBufferSize {
                        expected: start_vertex,
                        actual: i32::MAX as usize,
                    })?,
                vertex_count
                    .try_into()
                    .map_err(|_| Error::InvalidBufferSize {
                        expected: vertex_count,
                        actual: i32::MAX as usize,
                    })?,
                command_buffer.as_ptr() as *const _,
            );
        }

        Ok(())
    }
}
