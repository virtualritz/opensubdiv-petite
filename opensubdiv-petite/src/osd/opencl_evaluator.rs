use super::buffer_descriptor::BufferDescriptor;
use super::opencl_vertex_buffer::{OpenCLCommandQueue, OpenCLContext, OpenCLVertexBuffer};
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
use std::ptr::NonNull;
use std::rc::Rc;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Safe wrapper for OpenCL kernel.
#[derive(Debug, Clone)]
pub struct OpenCLKernel {
    ptr: Rc<NonNull<std::ffi::c_void>>,
}

impl OpenCLKernel {
    /// Create a new OpenCL kernel wrapper from a raw pointer.
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

/// Generic static eval stencils function for OpenCL.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// * `srcBuffer` - Input primvar buffer. Must have BindCLBuffer() method
///   returning an OpenCL buffer for read
/// * `srcDesc` - vertex buffer descriptor for the input buffer
/// * `dstBuffer` -  Output primvar buffer must have BindCLBuffer() method
///   returning an OpenCL buffer for write
/// * `dstDesc` - vertex buffer descriptor for the output buffer
/// * `stencilTable` - [StencilTable] or equivalent
/// * `kernel` - OpenCL kernel for evaluation
/// * `commandQueue` - OpenCL command queue
pub fn evaluate_stencils(
    src_buffer: &OpenCLVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut OpenCLVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &OpenCLStencilTable,
    kernel: &OpenCLKernel,
    command_queue: &OpenCLCommandQueue,
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

pub struct OpenCLStencilTable<'a> {
    pub(crate) ptr: sys::osd::OpenCLStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> OpenCLStencilTable<'a> {
    /// Create a new OpenCL stencil table from a Far stencil table.
    pub fn new(st: &'a StencilTable, context: &OpenCLContext) -> OpenCLStencilTable<'a> {
        let ptr = unsafe { sys::osd::CLStencilTable_Create(st.0, context.as_ptr() as *const _) };
        if ptr.is_null() {
            panic!("Could not create OpenCLStencilTable");
        }

        OpenCLStencilTable {
            ptr,
            st: std::marker::PhantomData,
        }
    }
}

impl Drop for OpenCLStencilTable<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::osd::CLStencilTable_destroy(self.ptr);
        }
    }
}
