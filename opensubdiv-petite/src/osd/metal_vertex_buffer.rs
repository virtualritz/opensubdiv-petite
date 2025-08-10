use opensubdiv_petite_sys as sys;
use std::convert::TryInto;
use std::ptr::NonNull;
use std::rc::Rc;

/// Safe wrapper for Metal device.
#[derive(Debug, Clone)]
pub struct MetalDevice {
    ptr: Rc<NonNull<std::ffi::c_void>>,
}

impl MetalDevice {
    /// Create a new Metal device wrapper from a raw pointer.
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

/// Safe wrapper for Metal command buffer.
#[derive(Debug, Clone)]
pub struct MetalCommandBuffer {
    ptr: Rc<NonNull<std::ffi::c_void>>,
}

impl MetalCommandBuffer {
    /// Create a new Metal command buffer wrapper from a raw pointer.
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
        elements_len: usize,
        vertices_len: usize,
        device: &MetalDevice,
    ) -> MetalVertexBuffer {
        let ptr = unsafe {
            sys::osd::MTLVertexBuffer_Create(
                elements_len.try_into().unwrap(),
                vertices_len.try_into().unwrap(),
                device.as_ptr() as *const _,
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
        command_buffer: &MetalCommandBuffer,
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
                command_buffer.as_ptr() as *const _,
            );
        }
    }
}
