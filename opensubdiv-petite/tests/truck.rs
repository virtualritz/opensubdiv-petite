#[cfg(feature = "truck")]
mod test_utils;

#[cfg(feature = "truck")]
#[test]
fn test_truck_integration_compiles() {
    use opensubdiv_petite::truck_integration::TruckIntegrationError;
    
    // Just verify the module compiles and types are accessible
    let _error: TruckIntegrationError = TruckIntegrationError::InvalidControlPoints;
    
    // This test passes if it compiles
    assert!(true);
}

#[cfg(feature = "truck")]
#[test]
fn test_creased_cube_to_step() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use truck_stepio::out;
    use std::fs;
    
    // Define the creased cube vertices
    let vertex_positions = vec![
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 1, 3, 2,  // front
        2, 3, 5, 4,  // top
        4, 5, 7, 6,  // back
        6, 7, 1, 0,  // bottom
        0, 2, 4, 6,  // left
        1, 7, 5, 3,  // right
    ];

    // Define creases
    let crease_indices = vec![
        0, 1,  // bottom front edge
        1, 3,  // right front edge
        3, 2,  // top front edge
        2, 0,  // left front edge
    ];
    let crease_weights = vec![2.0, 2.0, 2.0, 2.0];

    // Create topology descriptor
    let mut descriptor = TopologyDescriptor::new(
        vertex_positions.len(),  // Number of vertices, not faces
        &face_vertex_counts,
        &face_vertex_indices,
    );
    descriptor.creases(&crease_indices, &crease_weights);

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();
    
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement to generate B-spline patches
    // Based on OpenSubdiv docs, adaptive refinement isolates irregular features
    // and generates B-spline patches for regular regions
    use opensubdiv_petite::far::AdaptiveRefinementOptions;
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2; // Refine to isolate irregular vertices
    
    refiner
        .refine_adaptive(adaptive_options, &[]);

    // Create patch table with B-spline patches for higher-order surfaces
    use opensubdiv_petite::far::{PatchTableOptions, EndCapType};
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // For uniform refinement, patch control point indices reference vertices from
    // both the base level and the last refinement level. We need to build a vertex
    // buffer containing all vertices.
    use opensubdiv_petite::far::PrimvarRefiner;
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    
    // Get total number of vertices across all levels
    let total_vertices = refiner.vertex_total_count();
    println!("Total vertices across all levels: {}", total_vertices);
    
    // Flatten original vertex positions for interpolation
    let flat_positions: Vec<f32> = vertex_positions
        .iter()
        .flat_map(|v| v.iter().copied())
        .collect();
    
    // Build vertex buffer with all refinement levels
    let mut all_vertices = Vec::with_capacity(total_vertices);
    
    // Add base level vertices
    for v in &vertex_positions {
        all_vertices.push(*v);
    }
    
    // Add refined vertices from each level
    // With adaptive refinement, we may have different levels
    let num_levels = refiner.refinement_levels();
    println!("Number of refinement levels: {}", num_levels);
    
    for level in 1..num_levels {
        if let Some(refined) = primvar_refiner.interpolate(level, 3, &flat_positions) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            println!("Level {} has {} vertices", level, level_vertices.len());
            all_vertices.extend_from_slice(&level_vertices);
        }
    }
    
    println!("Built vertex buffer with {} vertices", all_vertices.len());
    
    // Debug: Check patch table information
    println!("Number of patch arrays: {}", patch_table.patch_arrays_len());
    for i in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(i) {
            println!("Patch array {}: type={:?}, num_patches={}, num_control_vertices={}", 
                i, desc.patch_type(), patch_table.patch_array_patches_len(i), desc.control_vertices_len());
        }
    }
    
    // Convert patches to truck shell
    use opensubdiv_petite::truck_integration::PatchTableExt;
    
    // Convert patches to truck shell
    let shell = patch_table.to_truck_shell(&all_vertices)
        .expect("Failed to convert to truck shell");
    
    // Compress and export the shell as STEP
    let compressed = shell.compress();
    
    // Write to STEP file
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            file_name: "creased_cube.step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();

    // Save to test output directory and compare with expected
    let output_path = test_utils::test_output_path("creased_cube.step");
    fs::write(&output_path, &step_string).expect("Failed to write STEP file");
    
    test_utils::assert_file_matches(&output_path, "creased_cube.step");
    
    println!("Successfully generated creased_cube.step with higher-order surfaces");
}