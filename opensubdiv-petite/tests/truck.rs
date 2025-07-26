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
    
    // Debug: First convert to surfaces to understand the issue
    println!("\n=== DEBUG: Truck Conversion ===");
    match patch_table.to_truck_surfaces(&all_vertices) {
        Ok(surfaces) => {
            println!("Successfully converted {} surfaces", surfaces.len());
            // Print first surface details
            if let Some(first_surface) = surfaces.first() {
                use truck_geometry::prelude::ParametricSurface;
                println!("First surface corners:");
                println!("  (0,0): {:?}", first_surface.subs(0.0, 0.0));
                println!("  (1,0): {:?}", first_surface.subs(1.0, 0.0));
                println!("  (0,1): {:?}", first_surface.subs(0.0, 1.0));
                println!("  (1,1): {:?}", first_surface.subs(1.0, 1.0));
            }
        }
        Err(e) => {
            println!("Surface conversion failed: {:?}", e);
        }
    }
    
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
    step_content.push_str("FILE_NAME('creased_cube_direct_nurbs.step','2024-01-01T00:00:00',(''),(''),\n");
    step_content.push_str("  'OpenSubdiv STEP Exporter','OpenSubdiv STEP Exporter','');\n");
    step_content.push_str("FILE_SCHEMA(('AP203'));\n");
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
    
    // Create shape representation for surfaces
    let shape_rep_id = entity_id;
    
    step_content.push_str(&format!("#{} = GEOMETRICALLY_BOUNDED_SURFACE_SHAPE_REPRESENTATION('',(", shape_rep_id));
    for (i, surf_id) in surface_ids.iter().enumerate() {
        step_content.push_str(&format!("#{}", surf_id));
        if i < surface_ids.len() - 1 {
            step_content.push_str(",");
        }
    }
    step_content.push_str("),#1000);\n");
    
    // Add a geometric representation context (required for shape representation)
    step_content.push_str("#1000 = (GEOMETRIC_REPRESENTATION_CONTEXT(3)\n");
    step_content.push_str("GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#1001))\n");
    step_content.push_str("GLOBAL_UNIT_ASSIGNED_CONTEXT((#1002,#1003,#1004))\n");
    step_content.push_str("REPRESENTATION_CONTEXT('ID1','3D'));\n");
    step_content.push_str("#1001 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-06),#1002,'distance accuracy','');\n");
    step_content.push_str("#1002 = (LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));\n");
    step_content.push_str("#1003 = (NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));\n");
    step_content.push_str("#1004 = (NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT());\n");
    
    step_content.push_str("ENDSEC;\n");
    step_content.push_str("END-ISO-10303-21;\n");
    
    // Save to test output directory
    let output_path = test_utils::test_output_path("creased_cube_direct_nurbs.step");
    fs::write(&output_path, &step_content).expect("Failed to write STEP file");
    
    test_utils::assert_file_matches(&output_path, "creased_cube_direct_nurbs.step");
    
    println!("Generated {} B-spline surfaces", surface_ids.len());
}

#[cfg(feature = "truck")]
#[test]
fn test_simple_cube_direct_nurbs_export() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        AdaptiveRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use std::fs;
    
    // Define a simple cube (no creases)
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

    // Create topology descriptor (no creases)
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    let total_vertices = refiner.vertex_total_count();
    let flat_positions: Vec<f32> = vertex_positions
        .iter()
        .flat_map(|v| v.iter().copied())
        .collect();
    
    let mut all_vertices = Vec::with_capacity(total_vertices);
    for v in &vertex_positions {
        all_vertices.push(*v);
    }
    
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
    
    // Generate improved STEP file with simpler structure
    let mut step_content = String::new();
    
    // STEP header
    step_content.push_str("ISO-10303-21;\n");
    step_content.push_str("HEADER;\n");
    step_content.push_str("FILE_DESCRIPTION(('Simple Cube NURBS Export'),'2;1');\n");
    step_content.push_str("FILE_NAME('simple_cube_nurbs.step','2024-01-01T00:00:00',(''),(''),\n");
    step_content.push_str("  'OpenSubdiv STEP Exporter','OpenSubdiv STEP Exporter','');\n");
    step_content.push_str("FILE_SCHEMA(('AP203'));\n");
    step_content.push_str("ENDSEC;\n");
    step_content.push_str("DATA;\n");
    
    let mut entity_id = 1;
    let mut surface_ids = Vec::new();
    
    // Export each patch as a B-spline surface
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            let num_patches = patch_table.patch_array_patches_len(array_idx);
            let control_verts_per_patch = desc.control_vertices_len();
            
            if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
                for patch_idx in 0..num_patches {
                    let start_idx = patch_idx * control_verts_per_patch;
                    let end_idx = start_idx + control_verts_per_patch;
                    
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
                            // Create B_SPLINE_SURFACE
                            let surface_id = entity_id;
                            step_content.push_str(&format!(
                                "#{} = B_SPLINE_SURFACE_WITH_KNOTS('',3,3,(",
                                surface_id
                            ));
                            
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
                            step_content.push_str("(4,4),"); // U knot multiplicities
                            step_content.push_str("(4,4),"); // V knot multiplicities
                            step_content.push_str("(0.,1.),"); // U knots
                            step_content.push_str("(0.,1.),"); // V knots
                            step_content.push_str(".UNSPECIFIED.);\n");
                            
                            surface_ids.push(surface_id);
                            entity_id += 1;
                        }
                    }
                }
            }
        }
    }
    
    // Add product structure
    let mut current_id = entity_id;
    
    // Application protocol
    step_content.push_str(&format!("#{} = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #{});\n", current_id, current_id + 1));
    current_id += 1;
    step_content.push_str(&format!("#{} = APPLICATION_CONTEXT('core data for automotive mechanical design processes');\n", current_id));
    let app_context_id = current_id;
    current_id += 1;
    
    // Shape definition
    step_content.push_str(&format!("#{} = SHAPE_DEFINITION_REPRESENTATION(#{}, #{});\n", current_id, current_id + 1, current_id + 7));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT_DEFINITION_SHAPE('','', #{});\n", current_id, current_id + 1));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT_DEFINITION('design','', #{}, #{});\n", current_id, current_id + 1, current_id + 4));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT_DEFINITION_FORMATION('','', #{});\n", current_id, current_id + 1));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT('Simple Cube','Simple Cube','', (#{}) );\n", current_id, current_id + 1));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT_CONTEXT('', #{}, 'mechanical');\n", current_id, app_context_id));
    current_id += 1;
    step_content.push_str(&format!("#{} = PRODUCT_DEFINITION_CONTEXT('part definition', #{}, 'design');\n", current_id, app_context_id));
    current_id += 1;
    
    // Shape representation
    let shape_rep_id = current_id;
    step_content.push_str(&format!("#{} = GEOMETRICALLY_BOUNDED_SURFACE_SHAPE_REPRESENTATION('',(", shape_rep_id));
    for (i, surf_id) in surface_ids.iter().enumerate() {
        step_content.push_str(&format!("#{}", surf_id));
        if i < surface_ids.len() - 1 {
            step_content.push_str(",");
        }
    }
    step_content.push_str(&format!("),#{});\n", current_id + 1));
    current_id += 1;
    
    // Geometric representation context
    step_content.push_str(&format!("#{} = (\n", current_id));
    step_content.push_str("    GEOMETRIC_REPRESENTATION_CONTEXT(3)\n");
    step_content.push_str(&format!("    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{}))\n", current_id + 1));
    step_content.push_str(&format!("    GLOBAL_UNIT_ASSIGNED_CONTEXT((#{}, #{}, #{}))\n", current_id + 2, current_id + 3, current_id + 4));
    step_content.push_str("    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')\n");
    step_content.push_str(");\n");
    current_id += 1;
    
    // Units
    step_content.push_str(&format!("#{} = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #{}, 'distance_accuracy_value','confusion accuracy');\n", current_id, current_id + 1));
    current_id += 1;
    step_content.push_str(&format!("#{} = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );\n", current_id));
    current_id += 1;
    step_content.push_str(&format!("#{} = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );\n", current_id));
    current_id += 1;
    step_content.push_str(&format!("#{} = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );\n", current_id));
    
    step_content.push_str("ENDSEC;\n");
    step_content.push_str("END-ISO-10303-21;\n");
    
    // Save to test output directory
    let output_path = test_utils::test_output_path("simple_cube_nurbs.step");
    fs::write(&output_path, &step_content).expect("Failed to write STEP file");
    
    test_utils::assert_file_matches(&output_path, "simple_cube_nurbs.step");
    
    println!("Generated {} B-spline surfaces", surface_ids.len());
}

#[cfg(feature = "truck")]
#[test]
fn test_simple_cube_to_step() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        AdaptiveRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use truck_stepio::out;
    use std::fs;
    
    // Define a simple cube (no creases)
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

    // Create topology descriptor (no creases)
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table with B-spline patches
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // Build vertex buffer
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
    
    // Add refined vertices from each level
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
    
    // Debug: First convert to surfaces to understand the issue
    println!("\n=== DEBUG: Truck Conversion ===");
    match patch_table.to_truck_surfaces(&all_vertices) {
        Ok(surfaces) => {
            println!("Successfully converted {} surfaces", surfaces.len());
            // Print first surface details
            if let Some(first_surface) = surfaces.first() {
                use truck_geometry::prelude::ParametricSurface;
                println!("First surface corners:");
                println!("  (0,0): {:?}", first_surface.subs(0.0, 0.0));
                println!("  (1,0): {:?}", first_surface.subs(1.0, 0.0));
                println!("  (0,1): {:?}", first_surface.subs(0.0, 1.0));
                println!("  (1,1): {:?}", first_surface.subs(1.0, 1.0));
            }
        }
        Err(e) => {
            println!("Surface conversion failed: {:?}", e);
        }
    }
    
    // Convert patches to truck shell
    let shell = patch_table.to_truck_shell(&all_vertices)
        .expect("Failed to convert to truck shell");
    
    // Compress and export the shell as STEP
    let compressed = shell.compress();
    
    // Write to STEP file
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            file_name: "simple_cube.step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();

    // Save to test output directory and compare with expected
    let output_path = test_utils::test_output_path("simple_cube.step");
    fs::write(&output_path, &step_string).expect("Failed to write STEP file");
    
    test_utils::assert_file_matches(&output_path, "simple_cube.step");
    
    println!("Successfully generated simple_cube.step");
}

#[cfg(feature = "truck")]
#[test]
fn test_debug_patch_positions() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        AdaptiveRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use std::fs;
    
    // Define a simple cube
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
    
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );
    
    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");
    
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);
    
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");
    
    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    let total_vertices = refiner.vertex_total_count();
    let flat_positions: Vec<f32> = vertex_positions
        .iter()
        .flat_map(|v| v.iter().copied())
        .collect();
    
    let mut all_vertices = Vec::with_capacity(total_vertices);
    for v in &vertex_positions {
        all_vertices.push(*v);
    }
    
    for level in 1..refiner.refinement_levels() {
        if let Some(refined) = primvar_refiner.interpolate(level, 3, &flat_positions) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }
    }
    
    // Export debug OBJ showing patch corners
    let mut obj_content = String::new();
    obj_content.push_str("# Debug: Patch corner positions\n");
    obj_content.push_str("# Original cube vertices marked with 'o'\n");
    obj_content.push_str("# Patch corners marked with 'o' in different colors\n\n");
    
    // Export original cube vertices
    obj_content.push_str("# Original cube vertices\n");
    for (i, v) in vertex_positions.iter().enumerate() {
        obj_content.push_str(&format!("v {} {} {} # orig {}\n", v[0], v[1], v[2], i));
    }
    
    // Export patch corner vertices
    obj_content.push_str("\n# Patch corners\n");
    let mut vertex_count = vertex_positions.len();
    
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            let num_patches = patch_table.patch_array_patches_len(array_idx);
            let control_verts_per_patch = desc.control_vertices_len();
            
            if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
                obj_content.push_str(&format!("\n# Patch array {} ({} patches)\n", array_idx, num_patches));
                
                for patch_idx in 0..num_patches.min(5) { // Only first 5 patches to avoid too much data
                    let start_idx = patch_idx * control_verts_per_patch;
                    
                    // Get the 4 corner control points (indices 0, 3, 12, 15 for a 4x4 patch)
                    if control_verts_per_patch == 16 {
                        let corners = [0, 3, 12, 15];
                        obj_content.push_str(&format!("# Patch {} corners:\n", patch_idx));
                        
                        for &corner in &corners {
                            let vert_idx = patch_vertices[start_idx + corner].0 as usize;
                            if vert_idx < all_vertices.len() {
                                let v = &all_vertices[vert_idx];
                                obj_content.push_str(&format!("v {} {} {} # patch {} corner {}\n", 
                                    v[0], v[1], v[2], patch_idx, corner));
                                vertex_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Create OBJ output path
    let output_path = test_utils::test_output_path("debug_patch_positions.obj");
    fs::create_dir_all(output_path.parent().unwrap()).ok();
    fs::write(&output_path, &obj_content).expect("Failed to write OBJ file");
    
    println!("Debug OBJ written to: {:?}", output_path);
    println!("Total patches: {}", patch_table.patch_arrays_len());
    println!("Vertex buffer size: {}", all_vertices.len());
}

#[cfg(feature = "truck")]
#[test]
fn test_export_simple_cube_to_obj() {
    use std::fs;
    
    // Define a simple cube (no creases)
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
    
    // Create OBJ content
    let mut obj_content = String::new();
    
    // Write header
    obj_content.push_str("# Simple cube original geometry (no creases)\n");
    obj_content.push_str("# Created by opensubdiv-petite test\n\n");
    
    // Write vertices
    obj_content.push_str("# Vertices\n");
    for (i, v) in vertex_positions.iter().enumerate() {
        obj_content.push_str(&format!("v {} {} {}  # vertex {}\n", v[0], v[1], v[2], i));
    }
    obj_content.push_str("\n");
    
    // Write faces (OBJ uses 1-based indexing)
    obj_content.push_str("# Faces\n");
    let face_names = ["front", "top", "back", "bottom", "left", "right"];
    for (face_idx, face_name) in face_names.iter().enumerate() {
        let start = face_idx * 4;
        obj_content.push_str(&format!("# {} face\n", face_name));
        obj_content.push_str(&format!("f {} {} {} {}\n", 
            face_vertex_indices[start] + 1,
            face_vertex_indices[start + 1] + 1,
            face_vertex_indices[start + 2] + 1,
            face_vertex_indices[start + 3] + 1
        ));
    }
    
    // Save to expected results directory
    let output_path = std::path::Path::new("tests/expected_results/simple_cube_original.obj");
    fs::write(&output_path, &obj_content).expect("Failed to write OBJ file");
    
    println!("Exported simple cube geometry to {:?}", output_path);
    println!("Vertices: {}", vertex_positions.len());
    println!("Faces: {}", face_vertex_counts.len());
}

#[cfg(feature = "truck")]
#[test]
fn test_export_creased_cube_to_obj() {
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
    
    // Create OBJ content
    let mut obj_content = String::new();
    
    // Write header
    obj_content.push_str("# Creased cube original geometry\n");
    obj_content.push_str("# Created by opensubdiv-petite test\n\n");
    
    // Write vertices
    obj_content.push_str("# Vertices\n");
    for (i, v) in vertex_positions.iter().enumerate() {
        obj_content.push_str(&format!("v {} {} {}  # vertex {}\n", v[0], v[1], v[2], i));
    }
    obj_content.push_str("\n");
    
    // Write faces (OBJ uses 1-based indexing)
    obj_content.push_str("# Faces\n");
    let face_names = ["front", "top", "back", "bottom", "left", "right"];
    for (face_idx, face_name) in face_names.iter().enumerate() {
        let start = face_idx * 4;
        obj_content.push_str(&format!("# {} face\n", face_name));
        obj_content.push_str(&format!("f {} {} {} {}\n", 
            face_vertex_indices[start] + 1,
            face_vertex_indices[start + 1] + 1,
            face_vertex_indices[start + 2] + 1,
            face_vertex_indices[start + 3] + 1
        ));
    }
    obj_content.push_str("\n");
    
    // Write edge information as comments
    obj_content.push_str("# Creased edges (vertex pairs with crease weight)\n");
    for i in (0..crease_indices.len()).step_by(2) {
        obj_content.push_str(&format!("# edge {}-{}: weight {}\n", 
            crease_indices[i], 
            crease_indices[i + 1], 
            crease_weights[i / 2]
        ));
    }
    
    // Save to expected results directory
    let output_path = std::path::Path::new("tests/expected_results/creased_cube_original.obj");
    fs::write(&output_path, &obj_content).expect("Failed to write OBJ file");
    
    println!("Exported original creased cube geometry to {:?}", output_path);
    println!("Vertices: {}", vertex_positions.len());
    println!("Faces: {}", face_vertex_counts.len());
    println!("Creased edges: {}", crease_weights.len());
}

#[cfg(feature = "truck")]
#[test]
fn test_single_quad_patch_generation() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        PatchTableOptions, EndCapType, UniformRefinementOptions,
        AdaptiveRefinementOptions,
    };
    
    // Non-planar quad - a saddle shape
    let vertex_positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.5],
        [0.0, 1.0, 0.5],
        [1.0, 1.0, 0.0],
    ];

    let face_vertex_counts = vec![4];
    let face_vertex_indices = vec![0, 1, 3, 2];

    println!("\n=== Testing patch generation at different refinement levels ===");
    
    // Test different refinement levels
    for level in 0..=3 {
        println!("\n--- Refinement level {} ---", level);
        
        // Create fresh refiner for each test
        let descriptor = TopologyDescriptor::new(
            vertex_positions.len(),
            &face_vertex_counts,
            &face_vertex_indices,
        );
        
        let refiner_options = TopologyRefinerOptions::default();
        let mut test_refiner = TopologyRefiner::new(descriptor, refiner_options)
            .expect("Failed to create topology refiner");
        
        if level > 0 {
            let mut uniform_options = UniformRefinementOptions::default();
            uniform_options.refinement_level = level;
            test_refiner.refine_uniform(uniform_options);
        }
        
        // Try to create patch table with different end cap types
        println!("  Testing different end cap types:");
        
        // Also test with generate_all_levels option
        println!("  With default options (no patches at base):");
        for end_cap_type in [EndCapType::None, EndCapType::BSplineBasis, EndCapType::GregoryBasis, EndCapType::LegacyGregory] {
            print!("    {:?}: ", end_cap_type);
            let patch_options = PatchTableOptions::new()
                .end_cap_type(end_cap_type);
                
            match PatchTable::new(&test_refiner, Some(patch_options)) {
                Ok(patch_table) => {
                    println!("{} patch arrays", patch_table.patch_arrays_len());
                    
                    for i in 0..patch_table.patch_arrays_len() {
                        if let Some(desc) = patch_table.patch_array_descriptor(i) {
                            println!("      Array {}: type={:?}, patches={}, cvs_per_patch={}", 
                                i, desc.patch_type(), 
                                patch_table.patch_array_patches_len(i), 
                                desc.control_vertices_len());
                        }
                    }
                }
                Err(e) => {
                    println!("Failed: {:?}", e);
                }
            }
        }
    }
    
    // Now test with adaptive refinement
    println!("\n--- Adaptive refinement ---");
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );
    
    let refiner_options = TopologyRefinerOptions::default();
    let mut adaptive_refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");
    
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    adaptive_refiner.refine_adaptive(adaptive_options, &[]);
    
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
        
    match PatchTable::new(&adaptive_refiner, Some(patch_options)) {
        Ok(patch_table) => {
            println!("  Patch table created successfully");
            println!("  Number of patch arrays: {}", patch_table.patch_arrays_len());
            
            for i in 0..patch_table.patch_arrays_len() {
                if let Some(desc) = patch_table.patch_array_descriptor(i) {
                    println!("  Array {}: type={:?}, patches={}, cvs_per_patch={}", 
                        i, desc.patch_type(), 
                        patch_table.patch_array_patches_len(i), 
                        desc.control_vertices_len());
                }
            }
        }
        Err(e) => {
            println!("  Failed to create patch table: {:?}", e);
        }
    }
}

#[cfg(feature = "truck")]
#[test]
fn test_debug_truck_conversion() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        AdaptiveRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use opensubdiv_petite::truck_integration::PatchTableExt;
    use truck_geometry::prelude::ParametricSurface;
    
    // Simple single quad for debugging
    let vertex_positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];

    let face_vertex_counts = vec![4];
    let face_vertex_indices = vec![0, 1, 3, 2];

    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement like creased cube test
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3; // Higher level for simpler geometry
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // Build vertex buffer
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
    
    println!("\n=== DEBUG: Patch Table Info ===");
    println!("Total vertices: {}", all_vertices.len());
    println!("Number of patch arrays: {}", patch_table.patch_arrays_len());
    
    // Debug patch arrays
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            println!("\nPatch array {}:", array_idx);
            println!("  Type: {:?}", desc.patch_type());
            println!("  Num patches: {}", patch_table.patch_array_patches_len(array_idx));
            println!("  Control verts per patch: {}", desc.control_vertices_len());
            
            if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
                let num_patches = patch_table.patch_array_patches_len(array_idx);
                for patch_idx in 0..num_patches.min(2) { // Print first 2 patches
                    println!("\n  Patch {}:", patch_idx);
                    let start = patch_idx * desc.control_vertices_len();
                    let end = start + desc.control_vertices_len();
                    
                    println!("    Control vertex indices:");
                    for (i, &idx) in patch_vertices[start..end].iter().enumerate() {
                        if i % 4 == 0 {
                            print!("      ");
                        }
                        print!("{:3} ", idx.0);
                        if (i + 1) % 4 == 0 {
                            println!();
                        }
                    }
                    
                    println!("    Control point positions:");
                    for (i, &idx) in patch_vertices[start..end].iter().enumerate() {
                        let vert_idx = idx.0 as usize;
                        if vert_idx < all_vertices.len() {
                            let v = &all_vertices[vert_idx];
                            println!("      {}: [{:.3}, {:.3}, {:.3}]", i, v[0], v[1], v[2]);
                        }
                    }
                }
            }
        }
    }
    
    // Try truck conversion
    println!("\n=== Attempting truck conversion ===");
    match patch_table.to_truck_surfaces(&all_vertices) {
        Ok(surfaces) => {
            println!("Successfully converted {} surfaces", surfaces.len());
            for (i, surface) in surfaces.iter().enumerate().take(2) {
                println!("\nSurface {}:", i);
                let (u_range, v_range) = surface.parameter_range();
                println!("  Parameter range: u={:?}, v={:?}", u_range, v_range);
                
                // Sample corners
                println!("  Corner points:");
                println!("    (0,0): {:?}", surface.subs(0.0, 0.0));
                println!("    (1,0): {:?}", surface.subs(1.0, 0.0));
                println!("    (0,1): {:?}", surface.subs(0.0, 1.0));
                println!("    (1,1): {:?}", surface.subs(1.0, 1.0));
            }
        }
        Err(e) => {
            println!("Conversion failed: {:?}", e);
        }
    }
}