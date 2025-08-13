use super::buffer_descriptor::BufferDescriptor;
use super::opencl_vertex_buffer::{OpenClCommandQueue, OpenClContext, OpenClVertexBuffer};
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
use std::marker::PhantomData;
use std::ptr::NonNull;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Safe wrapper for OpenCL kernel.
#[derive(Debug)]
pub struct OpenClKernel<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> OpenClKernel<'a> {
    /// Create a new OpenCL kernel wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<OpenClKernel<'a>> {
        NonNull::new(ptr).map(|ptr| OpenClKernel {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Generic static eval stencils function for OpenCL.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// # Arguments
///
/// * `src_buffer` - Input primvar buffer. Must have BindCLBuffer() method
///   returning an OpenCL buffer for read.
/// * `src_desc` - Vertex buffer descriptor for the input buffer.
/// * `dst_buffer` - Output primvar buffer. Must have BindCLBuffer() method
///   returning an OpenCL buffer for write.
/// * `dst_desc` - Vertex buffer descriptor for the output buffer.
/// * `stencil_table` - StencilTable or equivalent.
/// * `kernel` - OpenCL kernel for evaluation.
/// * `command_queue` - OpenCL command queue.
pub fn evaluate_stencils(
    src_buffer: &OpenClVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut OpenClVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &OpenClStencilTable,
    kernel: &OpenClKernel,
    command_queue: &OpenClCommandQueue,
) -> Result<()> {
    unsafe {
        if sys::osd::CLEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.ptr,
            kernel.as_ptr() as *const _,
            command_queue.as_ptr() as *const _,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}

/// OpenCL-specific stencil table for GPU evaluation.
///
/// This wraps a [`StencilTable`] for use with OpenCL GPU evaluation.
/// The lifetime parameter ensures the underlying stencil table outlives this
/// wrapper.
pub struct OpenClStencilTable<'a> {
    pub(crate) ptr: sys::osd::OpenCLStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> OpenClStencilTable<'a> {
    /// Create a new OpenCL stencil table from a [`StencilTable`].
    ///
    /// # Parameters
    ///
    /// - `st` -- The [`StencilTable`] to wrap.
    /// - `context` -- The [`OpenClContext`] for GPU memory allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the OpenCL stencil table creation fails.
    pub fn new(
        st: &'a StencilTable,
        context: &OpenClContext,
    ) -> crate::Result<OpenClStencilTable<'a>> {
        let ptr = unsafe { sys::osd::CLStencilTable_Create(st.0, context.as_ptr() as *const _) };
        if ptr.is_null() {
            return Err(crate::Error::GpuBackend(
                "Could not create OpenCLStencilTable".to_string(),
            ));
        }

        Ok(OpenClStencilTable {
            ptr,
            st: std::marker::PhantomData,
        })
    }
}

impl Drop for OpenClStencilTable<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::osd::CLStencilTable_destroy(self.ptr);
        }
    }
}
