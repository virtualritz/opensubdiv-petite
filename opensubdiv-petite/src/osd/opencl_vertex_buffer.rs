use crate::{Error, Result};
use opensubdiv_petite_sys as sys;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Safe wrapper for OpenCL context.
#[derive(Debug)]
pub struct OpenClContext<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> OpenClContext<'a> {
    /// Create a new OpenCL context wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<OpenClContext<'a>> {
        NonNull::new(ptr).map(|ptr| OpenClContext {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Safe wrapper for OpenCL command queue.
#[derive(Debug)]
pub struct OpenClCommandQueue<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> OpenClCommandQueue<'a> {
    /// Create a new OpenCL command queue wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<OpenClCommandQueue<'a>> {
        NonNull::new(ptr).map(|ptr| OpenClCommandQueue {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Concrete vertex buffer class for OpenCL subdivision.
///
/// [`OpenClVertexBuffer`] implements the VertexBufferInterface. An instance
/// of this buffer class can be passed to
/// [`evaluate_stencils()`](crate::osd::opencl_evaluator::evaluate_stencils()).
pub struct OpenClVertexBuffer(pub(crate) sys::osd::OpenCLVertexBufferPtr);

impl Drop for OpenClVertexBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::osd::CLVertexBuffer_destroy(self.0) }
    }
}

impl OpenClVertexBuffer {
    /// Create a new OpenCL vertex buffer.
    #[inline]
    pub fn new(
        element_count: usize,
        vertex_count: usize,
        context: Option<&OpenClContext>,
    ) -> Result<OpenClVertexBuffer> {
        let ptr = unsafe {
            sys::osd::CLVertexBuffer_Create(
                element_count
                    .try_into()
                    .map_err(|_| Error::InvalidBufferSize {
                        expected: element_count,
                        actual: i32::MAX as usize,
                    })?,
                vertex_count
                    .try_into()
                    .map_err(|_| Error::InvalidBufferSize {
                        expected: vertex_count,
                        actual: i32::MAX as usize,
                    })?,
                context.map_or(std::ptr::null(), |ctx| ctx.as_ptr() as *const _),
            )
        };
        if ptr.is_null() {
            return Err(Error::GpuBackend(
                "CLVertexBuffer_Create returned null".to_string(),
            ));
        }

        Ok(OpenClVertexBuffer(ptr))
    }

    /// Returns how many elements defined in this vertex buffer.
    #[inline]
    pub fn element_count(&self) -> usize {
        unsafe { sys::osd::CLVertexBuffer_GetNumElements(self.0) as _ }
    }

    /// Returns how many vertices allocated in this vertex buffer.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        unsafe { sys::osd::CLVertexBuffer_GetNumVertices(self.0) as _ }
    }

    /// Get the OpenCL buffer object.
    #[inline]
    pub fn bind_cl_buffer(&self, command_queue: &OpenClCommandQueue) -> *const std::ffi::c_void {
        unsafe { sys::osd::CLVertexBuffer_BindCLBuffer(self.0, command_queue.as_ptr() as *const _) }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to *OpenSubdiv*.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertex_count: usize,
        command_queue: &OpenClCommandQueue,
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
            sys::osd::CLVertexBuffer_UpdateData(
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
                command_queue.as_ptr() as *const _,
            );
        }

        Ok(())
    }
}
