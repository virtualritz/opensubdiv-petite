use super::buffer_descriptor::BufferDescriptor;
use super::metal_vertex_buffer::{MetalCommandBuffer, MetalDevice, MetalVertexBuffer};
use crate::far::StencilTable;

use opensubdiv_petite_sys as sys;

use crate::Error;
use std::marker::PhantomData;
use std::ptr::NonNull;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Safe wrapper for Metal compute encoder.
#[derive(Debug)]
pub struct MetalComputeEncoder<'a> {
    ptr: NonNull<std::ffi::c_void>,
    _marker: PhantomData<&'a std::ffi::c_void>,
}

impl<'a> MetalComputeEncoder<'a> {
    /// Create a new Metal compute encoder wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and remains valid
    /// for the lifetime 'a.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Option<MetalComputeEncoder<'a>> {
        NonNull::new(ptr).map(|ptr| MetalComputeEncoder {
            ptr,
            _marker: PhantomData,
        })
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

/// Generic static eval stencils function for Metal.
///
/// This function has a same signature as other device kernels have so that it
/// can be called in the same way from OsdMesh template interface.
///
/// # Arguments
///
/// * `src_buffer` - Input primvar buffer. Must have GetMTLBuffer() method
///   returning a Metal buffer for read.
/// * `src_desc` - Vertex buffer descriptor for the input buffer.
/// * `dst_buffer` - Output primvar buffer. Must have GetMTLBuffer() method
///   returning a Metal buffer for write.
/// * `dst_desc` - Vertex buffer descriptor for the output buffer.
/// * `stencil_table` - StencilTable or equivalent.
/// * `command_buffer` - Metal command buffer for execution.
/// * `compute_encoder` - Metal compute command encoder.
pub fn evaluate_stencils(
    src_buffer: &MetalVertexBuffer,
    src_desc: BufferDescriptor,
    dst_buffer: &mut MetalVertexBuffer,
    dst_desc: BufferDescriptor,
    stencil_table: &MetalStencilTable,
    command_buffer: &MetalCommandBuffer,
    compute_encoder: &MetalComputeEncoder,
) -> Result<()> {
    unsafe {
        if sys::osd::MTLComputeEvaluator_EvalStencils(
            src_buffer.0,
            src_desc.0,
            dst_buffer.0,
            dst_desc.0,
            stencil_table.ptr,
            command_buffer.as_ptr() as *const _,
            compute_encoder.as_ptr() as *const _,
        ) {
            Ok(())
        } else {
            Err(Error::EvalStencilsFailed)
        }
    }
}

pub struct MetalStencilTable<'a> {
    pub(crate) ptr: sys::osd::MetalStencilTablePtr,
    st: std::marker::PhantomData<&'a StencilTable>,
}

impl<'a> MetalStencilTable<'a> {
    /// Create a new Metal stencil table from a Far stencil table.
    pub fn new(st: &'a StencilTable, device: &MetalDevice) -> Result<MetalStencilTable<'a>> {
        let ptr = unsafe { sys::osd::MTLStencilTable_Create(st.0, device.as_ptr() as *const _) };
        if ptr.is_null() {
            return Err(Error::GpuBackend(
                "Could not create MetalStencilTable".to_string(),
            ));
        }

        Ok(MetalStencilTable {
            ptr,
            st: std::marker::PhantomData,
        })
    }
}

impl Drop for MetalStencilTable<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::osd::MTLStencilTable_destroy(self.ptr);
        }
    }
}
