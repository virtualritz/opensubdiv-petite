//! Tests for the osd module.

use opensubdiv_petite::{far, osd};

#[test]
fn test_buffer_descriptor_creation() {
    let desc = osd::BufferDescriptor::new(0, 3, 3);
    // BufferDescriptor should be created successfully.
    let _ = desc;
}

#[test]
fn test_cpu_vertex_buffer() {
    let num_vertices = 8;
    let num_elements = 3;
    
    let buffer = osd::CpuVertexBuffer::new(num_elements, num_vertices);
    
    // Should be able to bind the buffer.
    let data = buffer.bind_cpu_buffer();
    assert_eq!(data.len(), num_vertices * num_elements);
}

#[test]
fn test_cpu_vertex_buffer_update() {
    let num_vertices = 8;
    let num_elements = 3;
    
    let mut buffer = osd::CpuVertexBuffer::new(num_elements, num_vertices);
    
    let test_data = vec![1.0f32; num_vertices * num_elements];
    
    // Update the buffer with test data.
    buffer.update_data(&test_data, 0, num_vertices);
    
    let data = buffer.bind_cpu_buffer();
    assert_eq!(data.len(), test_data.len());
    
    // Verify the data was copied.
    for (i, &val) in data.iter().enumerate() {
        assert_eq!(val, test_data[i]);
    }
}

#[test]
fn test_cpu_evaluator() {
    // Create a simple topology.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 
        2, 3, 5, 4, 
        4, 5, 7, 6, 
        6, 7, 1, 0, 
        1, 7, 5, 3, 
        6, 0, 2, 4,
    ];
    
    let positions = [
        -0.5, -0.5,  0.5,
         0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,
         0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
    ];
    
    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices);
    let options = far::TopologyRefinerOptions::default();
    
    let mut refiner = far::TopologyRefiner::new(descriptor, options)
        .expect("Failed to create TopologyRefiner");
    
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: 1,
        ..Default::default()
    });
    
    let stencil_table = far::StencilTable::new(&refiner, far::StencilTableOptions::default());
    
    let n_coarse_verts = refiner.level(0).unwrap().vertex_count();
    let n_refined_verts = stencil_table.len();
    
    let mut src_buffer = osd::CpuVertexBuffer::new(3, n_coarse_verts);
    let mut dst_buffer = osd::CpuVertexBuffer::new(3, n_refined_verts);
    
    src_buffer.update_data(&positions, 0, n_coarse_verts);
    
    let src_desc = osd::BufferDescriptor::new(0, 3, 3);
    let dst_desc = osd::BufferDescriptor::new(0, 3, 3);
    
    // Evaluate stencils.
    osd::cpu_evaluator::evaluate_stencils(&src_buffer, src_desc, &mut dst_buffer, dst_desc, &stencil_table)
        .expect("Failed to evaluate stencils");
    
    // Check that we got refined vertices.
    let refined_data = dst_buffer.bind_cpu_buffer();
    assert_eq!(refined_data.len(), n_refined_verts * 3);
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_vertex_buffer() {
    let num_vertices = 8;
    let num_elements = 3;
    
    let buffer = osd::CudaVertexBuffer::new(num_elements, num_vertices);
    
    // Should be able to bind the buffer.
    let data = buffer.bind_cuda_buffer();
    assert_eq!(data.len(), num_vertices * num_elements);
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_evaluator() {
    // Create a simple topology.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 
        2, 3, 5, 4, 
        4, 5, 7, 6, 
        6, 7, 1, 0, 
        1, 7, 5, 3, 
        6, 0, 2, 4,
    ];
    
    let positions = [
        -0.5, -0.5,  0.5,
         0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,
         0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
    ];
    
    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices);
    let options = far::TopologyRefinerOptions::default();
    
    let mut refiner = far::TopologyRefiner::new(descriptor, options)
        .expect("Failed to create TopologyRefiner");
    
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: 1,
        ..Default::default()
    });
    
    let stencil_table = far::StencilTable::new(&refiner, far::StencilTableOptions::default());
    
    let n_coarse_verts = refiner.level(0).unwrap().vertex_count();
    let n_refined_verts = stencil_table.len();
    
    let mut src_buffer = osd::CudaVertexBuffer::new(3, n_coarse_verts);
    let mut dst_buffer = osd::CudaVertexBuffer::new(3, n_refined_verts);
    
    src_buffer.update_data(&positions, 0, n_coarse_verts);
    
    let src_desc = osd::BufferDescriptor::new(0, 3, 3);
    let dst_desc = osd::BufferDescriptor::new(0, 3, 3);
    
    let cuda_stencil_table = osd::CudaStencilTable::new(&stencil_table);
    
    // Evaluate stencils.
    osd::cuda_evaluator::evaluate_stencils(&src_buffer, src_desc, &mut dst_buffer, dst_desc, &cuda_stencil_table)
        .expect("Failed to evaluate stencils");
    
    // Check that we got refined vertices.
    let refined_data = dst_buffer.bind_cuda_buffer();
    assert_eq!(refined_data.len(), n_refined_verts * 3);
}