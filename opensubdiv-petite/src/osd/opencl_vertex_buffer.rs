use opensubdiv_petite_sys as sys;
use std::convert::TryInto;
use std::ptr::NonNull;
use std::rc::Rc;

/// Safe wrapper for OpenCL context.
#[derive(Debug, Clone)]
pub struct OpenCLContext {
    ptr: Rc<NonNull<std::ffi::c_void>>,
}

impl OpenCLContext {
    /// Create a new OpenCL context wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime of this wrapper.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr: Rc::new(ptr) })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Safe wrapper for OpenCL command queue.
#[derive(Debug, Clone)]
pub struct OpenCLCommandQueue {
    ptr: Rc<NonNull<std::ffi::c_void>>,
}

impl OpenCLCommandQueue {
    /// Create a new OpenCL command queue wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime of this wrapper.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr: Rc::new(ptr) })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

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
    /// Create a new OpenCL vertex buffer.
    #[inline]
    pub fn new(
        elements_len: usize,
        vertices_len: usize,
        context: &OpenCLContext,
    ) -> OpenCLVertexBuffer {
        let ptr = unsafe {
            sys::osd::CLVertexBuffer_Create(
                elements_len.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                context.as_ptr() as *const _,
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
    pub fn bind_cl_buffer(&self, command_queue: &OpenCLCommandQueue) -> *const std::ffi::c_void {
        unsafe { sys::osd::CLVertexBuffer_BindCLBuffer(self.0, command_queue.as_ptr() as *const _) }
    }

    /// This method is meant to be used in client code in order to provide
    /// coarse vertices data to *OpenSubdiv*.
    #[inline]
    pub fn update_data(
        &mut self,
        src: &[f32],
        start_vertex: usize,
        vertices_len: usize,
        command_queue: &OpenCLCommandQueue,
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
                command_queue.as_ptr() as *const _,
            );
        }
    }
}
