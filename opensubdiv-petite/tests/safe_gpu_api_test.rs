#[test]
fn test_safe_wrappers_creation() {
    use opensubdiv_petite::osd;
    use std::ptr;

    // Test that we can create safe wrappers from null pointers
    #[cfg(feature = "opencl")]
    {
        let context =
            unsafe { osd::opencl_vertex_buffer::OpenCLContext::from_ptr(ptr::null_mut()) };
        assert!(context.is_none());

        let queue =
            unsafe { osd::opencl_vertex_buffer::OpenCLCommandQueue::from_ptr(ptr::null_mut()) };
        assert!(queue.is_none());

        let kernel = unsafe { osd::opencl_evaluator::OpenCLKernel::from_ptr(ptr::null_mut()) };
        assert!(kernel.is_none());
    }

    #[cfg(feature = "cuda")]
    {
        let context = unsafe { osd::cuda_vertex_buffer::CudaContext::from_ptr(ptr::null_mut()) };
        assert!(context.is_none());
    }

    #[cfg(all(feature = "metal", target_os = "macos"))]
    {
        let device = unsafe { osd::metal_vertex_buffer::MetalDevice::from_ptr(ptr::null_mut()) };
        assert!(device.is_none());

        let cmd_buffer =
            unsafe { osd::metal_vertex_buffer::MetalCommandBuffer::from_ptr(ptr::null_mut()) };
        assert!(cmd_buffer.is_none());

        let encoder =
            unsafe { osd::metal_evaluator::MetalComputeEncoder::from_ptr(ptr::null_mut()) };
        assert!(encoder.is_none());
    }
}

#[test]
fn test_wrapper_clone() {
    // Test that wrappers can be cloned
    use opensubdiv_petite::osd;

    // Create a non-null pointer for testing
    let test_ptr = 0x1234 as *mut std::ffi::c_void;

    #[cfg(feature = "opencl")]
    {
        let context =
            unsafe { osd::opencl_vertex_buffer::OpenCLContext::from_ptr(test_ptr) }.unwrap();
        let _context2 = context.clone();
        // Both clones should be valid
    }

    #[cfg(feature = "cuda")]
    {
        let context = unsafe { osd::cuda_vertex_buffer::CudaContext::from_ptr(test_ptr) }.unwrap();
        let _context2 = context.clone();
        // Both clones should be valid
    }
}
