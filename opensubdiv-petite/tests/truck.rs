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

#[cfg(feature = "truck")]
#[test]
fn test_creased_cube_direct_nurbs_export() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
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
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );
    descriptor.creases(&crease_indices, &crease_weights);

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();
    
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    use opensubdiv_petite::far::AdaptiveRefinementOptions;
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table with B-spline patches
    use opensubdiv_petite::far::{PatchTableOptions, EndCapType};
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // Build complete vertex buffer
    use opensubdiv_petite::far::PrimvarRefiner;
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    
    let total_vertices = refiner.vertex_total_count();
    let flat_positions: Vec<f32> = vertex_positions
        .iter()
        .flat_map(|v| v.iter().copied())
        .collect();
    
    let mut all_vertices = Vec::with_capacity(total_vertices);
    
    // Add base level vertices
    for v in &vertex_positions {
        all_vertices.push(*v);
    }
    
    // Add refined vertices
    let num_levels = refiner.refinement_levels();
    for level in 1..num_levels {
        if let Some(refined) = primvar_refiner.interpolate(level, 3, &flat_positions) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }
    }
    
    // Generate STEP file directly from patches without truck
    let mut step_content = String::new();
    
    // STEP header
    step_content.push_str("ISO-10303-21;\n");
    step_content.push_str("HEADER;\n");
    step_content.push_str("FILE_DESCRIPTION(('Direct NURBS Export Test'),'2;1');\n");
    step_content.push_str("FILE_NAME('creased_cube_direct_nurbs.step','2024-01-01T00:00:00',(),(),'','','');\n");
    step_content.push_str("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\n");
    step_content.push_str("ENDSEC;\n");
    step_content.push_str("DATA;\n");
    
    let mut entity_id = 1;
    let mut surface_ids = Vec::new();
    
    // Export each patch as a B-spline surface
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            let num_patches = patch_table.patch_array_patches_len(array_idx);
            let control_verts_per_patch = desc.control_vertices_len();
            
            // Get patch control vertex indices
            if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
                for patch_idx in 0..num_patches {
                    let start_idx = patch_idx * control_verts_per_patch;
                    let end_idx = start_idx + control_verts_per_patch;
                    
                    // B-spline surfaces need 4x4 control points
                    if control_verts_per_patch == 16 {
                        // Create control points
                        let mut control_point_ids = Vec::new();
                        
                        for i in start_idx..end_idx {
                            let vert_idx = patch_vertices[i].0 as usize;
                        if vert_idx < all_vertices.len() {
                            let v = &all_vertices[vert_idx];
                            step_content.push_str(&format!(
                                "#{} = CARTESIAN_POINT('',({},{},{}));\n",
                                entity_id, v[0], v[1], v[2]
                            ));
                            control_point_ids.push(entity_id);
                            entity_id += 1;
                        }
                    }
                    
                    if control_point_ids.len() == 16 {
                        // Create B-spline surface with rational flag
                        step_content.push_str(&format!(
                            "#{} = B_SPLINE_SURFACE_WITH_KNOTS('',3,3,(",
                            entity_id
                        ));
                        
                        // Control points in 4x4 grid
                        for i in 0..4 {
                            step_content.push_str("(");
                            for j in 0..4 {
                                step_content.push_str(&format!("#{}", control_point_ids[i * 4 + j]));
                                if j < 3 { step_content.push_str(","); }
                            }
                            step_content.push_str(")");
                            if i < 3 { step_content.push_str(","); }
                        }
                        
                        step_content.push_str("),.UNSPECIFIED.,.F.,.F.,.F.,");
                        step_content.push_str("(4,4),"); // Knot multiplicities for U
                        step_content.push_str("(4,4),"); // Knot multiplicities for V
                        step_content.push_str("(0.,1.),"); // U knots
                        step_content.push_str("(0.,1.),"); // V knots
                        step_content.push_str(".UNSPECIFIED.);\n");
                        
                        surface_ids.push(entity_id);
                        entity_id += 1;
                    }
                }
            }
            }
        }
    }
    
    // Create geometric set to hold all surfaces
    step_content.push_str(&format!("#{} = GEOMETRIC_SET('',(", entity_id));
    for (i, surf_id) in surface_ids.iter().enumerate() {
        step_content.push_str(&format!("#{}", surf_id));
        if i < surface_ids.len() - 1 {
            step_content.push_str(",");
        }
    }
    step_content.push_str("));\n");
    
    step_content.push_str("ENDSEC;\n");
    step_content.push_str("END-ISO-10303-21;\n");
    
    // Save to test output directory
    let output_path = test_utils::test_output_path("creased_cube_direct_nurbs.step");
    fs::write(&output_path, &step_content).expect("Failed to write STEP file");
    
    test_utils::assert_file_matches(&output_path, "creased_cube_direct_nurbs.step");
    
    println!("Generated {} B-spline surfaces", surface_ids.len());
}