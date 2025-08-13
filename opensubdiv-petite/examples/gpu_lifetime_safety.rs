//! Example demonstrating lifetime safety with GPU context wrappers.
//!
//! This example shows how the new lifetime-based GPU context wrappers
//! prevent use-after-free bugs at compile time.

#[cfg(feature = "opencl")]
fn opencl_example() {
    use opensubdiv_petite::osd;

    // Create a mock OpenCL context pointer (in real code this would come from
    // OpenCL API)
    let mock_context_ptr = 0x1234 as *mut std::ffi::c_void;

    // Create a context wrapper with lifetime tracking
    let context = unsafe { osd::opencl_vertex_buffer::OpenClContext::from_ptr(mock_context_ptr) }
        .expect("Context should be created");

    // Create a vertex buffer that borrows the context
    // The buffer cannot outlive the context due to lifetime constraints
    let buffer =
        osd::OpenClVertexBuffer::new(3, 100, Some(&context)).expect("Buffer should be created");

    // This would not compile if we tried to move the buffer out of this scope
    // while the context is dropped, preventing use-after-free
    println!(
        "Created OpenCL buffer with {} vertices",
        buffer.vertex_count()
    );
}

#[cfg(feature = "cuda")]
fn cuda_example() {
    use opensubdiv_petite::osd;

    // Create a mock CUDA context pointer
    let mock_context_ptr = 0x5678 as *mut std::ffi::c_void;

    // Create a context wrapper with lifetime tracking
    let context = unsafe { osd::cuda_vertex_buffer::CudaContext::from_ptr(mock_context_ptr) }
        .expect("Context should be created");

    // The context can no longer be cloned, preventing multiple ownership
    // let context2 = context.clone(); // This would not compile!

    // Create a vertex buffer using the context
    let buffer =
        osd::CudaVertexBuffer::new(3, 100, Some(&context)).expect("Buffer should be created");

    println!(
        "Created CUDA buffer with {} vertices",
        buffer.vertex_count()
    );
}

fn main() {
    println!("GPU Lifetime Safety Example");
    println!("============================");

    #[cfg(feature = "opencl")]
    {
        println!("\nOpenCL Example:");
        opencl_example();
    }

    #[cfg(feature = "cuda")]
    {
        println!("\nCUDA Example:");
        cuda_example();
    }

    #[cfg(not(any(feature = "opencl", feature = "cuda")))]
    {
        println!("\nPlease enable opencl or cuda feature to run this example:");
        println!("cargo run --example gpu_lifetime_safety --features opencl");
    }
}
