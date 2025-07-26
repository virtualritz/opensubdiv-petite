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
fn test_verify_bspline_knots() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        UniformRefinementOptions, PatchTableOptions, EndCapType,
    };
    use truck_geometry::prelude::{BSplineSurface, KnotVec, ParametricSurface};
    use truck_modeling::cgmath::Point3;
    
    // Create a single quad face
    let vertex_positions = vec![
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, 1.0, 0.0],
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
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options).unwrap();
    
    // Refine to level 2 to get a patch
    let uniform_options = UniformRefinementOptions {
        refinement_level: 2,
        ..Default::default()
    };
    refiner.refine_uniform(uniform_options);
    
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options)).unwrap();
    
    println!("Created {} patches", patch_table.patch_arrays_len());
    
    // Test creating B-spline with correct knots
    println!("\n=== Testing Knot Vectors ===");
    
    // Bezier knots (wrong for OpenSubdiv)
    let bezier_knots = KnotVec::bezier_knot(3);
    println!("Bezier knots (degree 3): {:?}", bezier_knots);
    
    // Uniform B-spline knots (correct for OpenSubdiv)
    let uniform_knots = KnotVec::uniform_knot(3, 4); // degree 3, 4 control points
    println!("Uniform B-spline knots: {:?}", uniform_knots);
    
    // Test surface evaluation with both knot types
    let test_control_points = vec![
        vec![
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(-0.33, -1.0, 0.0),
            Point3::new(0.33, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
        ],
        vec![
            Point3::new(-1.0, -0.33, 0.0),
            Point3::new(-0.33, -0.33, 0.1),
            Point3::new(0.33, -0.33, 0.1),
            Point3::new(1.0, -0.33, 0.0),
        ],
        vec![
            Point3::new(-1.0, 0.33, 0.0),
            Point3::new(-0.33, 0.33, 0.1),
            Point3::new(0.33, 0.33, 0.1),
            Point3::new(1.0, 0.33, 0.0),
        ],
        vec![
            Point3::new(-1.0, 1.0, 0.0),
            Point3::new(-0.33, 1.0, 0.0),
            Point3::new(0.33, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ],
    ];
    
    // Create surfaces with different knot vectors
    let bezier_surface = BSplineSurface::new(
        (bezier_knots.clone(), bezier_knots.clone()),
        test_control_points.clone()
    );
    
    let uniform_surface = BSplineSurface::new(
        (uniform_knots.clone(), uniform_knots.clone()),
        test_control_points.clone()
    );
    
    // Check parameter ranges
    println!("\nParameter ranges:");
    println!("Bezier surface: {:?}", bezier_surface.parameter_range());
    println!("Uniform B-spline surface: {:?}", uniform_surface.parameter_range());
    
    // Evaluate at center
    println!("\nEvaluating at (0.5, 0.5):");
    println!("Bezier surface: {:?}", bezier_surface.subs(0.5, 0.5));
    println!("Uniform B-spline surface: {:?}", uniform_surface.subs(0.5, 0.5));
    
    // For uniform B-splines, the valid parameter range is different
    // The parameter range is [k[d], k[n]] where d is degree and n is number of control points
    let u_start = uniform_knots[3];  // degree 3
    let u_end = uniform_knots[4];    // n = 4 control points
    println!("\nUniform B-spline valid range: [{}, {}]", u_start, u_end);
    
    // Evaluate corners with correct parameter range
    println!("\nCorner evaluations:");
    for (u, v) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
        println!("At ({}, {}):", u, v);
        println!("  Bezier: {:?}", bezier_surface.subs(u, v));
        println!("  Uniform: {:?}", uniform_surface.subs(u, v));
    }
    
    // Try evaluating at the actual valid range for uniform B-spline
    println!("\nUniform B-spline at valid parameter range:");
    println!("  At ({}, {}): {:?}", u_start, u_start, uniform_surface.subs(u_start, u_start));
    println!("  At ({}, {}): {:?}", u_end, u_start, uniform_surface.subs(u_end, u_start));
    println!("  At ({}, {}): {:?}", u_start, u_end, uniform_surface.subs(u_start, u_end));
    println!("  At ({}, {}): {:?}", u_end, u_end, uniform_surface.subs(u_end, u_end));
}

#[cfg(feature = "truck")]
#[test]
fn test_osd_patch_to_bspline_fix() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        UniformRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use truck_geometry::prelude::{BSplineSurface, KnotVec, ParametricSurface};
    use truck_modeling::cgmath::Point3;
    
    // Create a simple cube
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
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options).unwrap();
    
    // Use adaptive refinement like in the original test
    use opensubdiv_petite::far::AdaptiveRefinementOptions;
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);
    
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options)).unwrap();
    
    // Get refined vertices
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
    
    println!("Total vertices: {}", all_vertices.len());
    println!("Number of patch arrays: {}", patch_table.patch_arrays_len());
    
    for i in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(i) {
            println!("Patch array {}: type={:?}, num_patches={}, num_control_vertices={}", 
                i, desc.patch_type(), patch_table.patch_array_patches_len(i), desc.control_vertices_len());
        }
    }
    
    // Try to convert the first patch manually with corrected knots
    if patch_table.patch_arrays_len() > 0 {
        if let Some(desc) = patch_table.patch_array_descriptor(0) {
            if let Some(patch_vertices) = patch_table.patch_array_vertices(0) {
                if desc.control_vertices_len() == 16 {
                    // Get the 16 control points
                    let mut control_points = vec![vec![Point3::new(0.0, 0.0, 0.0); 4]; 4];
                    
                    for i in 0..16 {
                        let row = i / 4;
                        let col = i % 4;
                        let vert_idx = patch_vertices[i].0 as usize;
                        if vert_idx < all_vertices.len() {
                            let v = &all_vertices[vert_idx];
                            control_points[row][col] = Point3::new(v[0] as f64, v[1] as f64, v[2] as f64);
                        }
                    }
                    
                    println!("\nFirst patch control points:");
                    for (i, row) in control_points.iter().enumerate() {
                        println!("Row {}: {:?}", i, row);
                    }
                    
                    // Also print the raw indices
                    println!("\nFirst patch vertex indices:");
                    for i in 0..16 {
                        let vert_idx = patch_vertices[i].0;
                        println!("Control point {}: vertex index {}", i, vert_idx);
                    }
                    
                    // Create B-spline with uniform knots - but maybe we need clamped uniform knots?
                    // Clamped uniform B-spline has multiplicity at ends to interpolate corners
                    let clamped_knots = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
                    let uniform_knots = KnotVec::uniform_knot(3, 4);
                    
                    println!("\nTesting different knot vectors:");
                    println!("Clamped (Bezier): {:?}", clamped_knots);
                    println!("Uniform: {:?}", uniform_knots);
                    
                    let clamped_surface = BSplineSurface::new(
                        (clamped_knots.clone(), clamped_knots.clone()),
                        control_points.clone()
                    );
                    
                    let uniform_surface = BSplineSurface::new(
                        (uniform_knots.clone(), uniform_knots.clone()),
                        control_points.clone()
                    );
                    
                    // Evaluate at parametric corners
                    println!("\nEvaluating surfaces at corners:");
                    for (u, v) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                        println!("At ({}, {}):", u, v);
                        println!("  Clamped: {:?}", clamped_surface.subs(u, v));
                        println!("  Uniform: {:?}", uniform_surface.subs(u, v));
                    }
                    
                    // OpenSubdiv might be expecting the patches to be evaluated over [0,1]
                    // even though they're uniform B-splines. This suggests we need to 
                    // reparameterize or use a different knot vector.
                    
                    // Let's also check if this is a boundary patch or interior patch
                    // by looking at the vertex positions
                    println!("\nAnalyzing patch location:");
                    let center = Point3::new(
                        (control_points[1][1].x + control_points[1][2].x + 
                         control_points[2][1].x + control_points[2][2].x) / 4.0,
                        (control_points[1][1].y + control_points[1][2].y + 
                         control_points[2][1].y + control_points[2][2].y) / 4.0,
                        (control_points[1][1].z + control_points[1][2].z + 
                         control_points[2][1].z + control_points[2][2].z) / 4.0,
                    );
                    println!("Approximate patch center: {:?}", center);
                    
                    // Test evaluation with the actual OpenSubdiv control points
                    println!("\nEvaluating patch with OSD control points:");
                    let test_surface = BSplineSurface::new(
                        (clamped_knots.clone(), clamped_knots.clone()),
                        control_points.clone()
                    );
                    
                    // Evaluate at several points
                    for v in 0..5 {
                        for u in 0..5 {
                            let u_param = u as f64 / 4.0;
                            let v_param = v as f64 / 4.0;
                            let pt = test_surface.subs(u_param, v_param);
                            println!("  ({:.2}, {:.2}): [{:.3}, {:.3}, {:.3}]", 
                                u_param, v_param, pt.x, pt.y, pt.z);
                        }
                    }
                    
                    // Export this patch as OBJ for visualization
                    use std::fs;
                    let mut obj_content = String::new();
                    obj_content.push_str("# First OpenSubdiv patch control points\n");
                    obj_content.push_str("# Control points arranged in 4x4 grid\n\n");
                    
                    // Write control points
                    for (i, row) in control_points.iter().enumerate() {
                        for (j, pt) in row.iter().enumerate() {
                            obj_content.push_str(&format!("v {} {} {} # row {} col {}\n", 
                                pt.x, pt.y, pt.z, i, j));
                        }
                    }
                    
                    // Write control polygon
                    obj_content.push_str("\n# Control polygon rows\n");
                    for i in 0..4 {
                        for j in 0..3 {
                            let v1 = i * 4 + j + 1;
                            let v2 = i * 4 + j + 2;
                            obj_content.push_str(&format!("l {} {}\n", v1, v2));
                        }
                    }
                    
                    obj_content.push_str("\n# Control polygon columns\n");
                    for j in 0..4 {
                        for i in 0..3 {
                            let v1 = i * 4 + j + 1;
                            let v2 = (i + 1) * 4 + j + 1;
                            obj_content.push_str(&format!("l {} {}\n", v1, v2));
                        }
                    }
                    
                    let output_path = test_utils::test_output_path("first_patch_control_points.obj");
                    fs::create_dir_all(output_path.parent().unwrap()).ok();
                    fs::write(&output_path, &obj_content).expect("Failed to write OBJ file");
                    println!("Wrote control points to {:?}", output_path);
                    
                    // Test uniform B-spline with correct parameter mapping
                    println!("\n=== Testing Uniform B-spline with parameter mapping ===");
                    
                    // For uniform B-splines, the valid parameter range is from knot[degree] to knot[n]
                    // where n = number of control points. For cubic (degree 3) with 4 control points:
                    // Knot vector: [0, 0, 0, 0, 0.25, 0.5, 0.75, 1, 1, 1, 1]
                    // Valid range: from knot[3]=0 to knot[4]=0.25
                    // But truck's uniform_knot normalizes differently
                    
                    let uniform_surface = BSplineSurface::new(
                        (uniform_knots.clone(), uniform_knots.clone()),
                        control_points.clone()
                    );
                    
                    // Get the actual parameter range
                    let (u_range, v_range) = uniform_surface.parameter_range();
                    println!("Uniform B-spline parameter range: u={:?}, v={:?}", u_range, v_range);
                    
                    // Evaluate at the actual valid range
                    use std::ops::Bound;
                    let u_min = match u_range.0 {
                        Bound::Included(v) => v,
                        _ => 0.0,
                    };
                    let u_max = match u_range.1 {
                        Bound::Included(v) => v,
                        _ => 1.0,
                    };
                    
                    println!("Evaluating uniform B-spline at valid range:");
                    for v in 0..5 {
                        for u in 0..5 {
                            let u_param = u_min + (u as f64 / 4.0) * (u_max - u_min);
                            let v_param = u_min + (v as f64 / 4.0) * (u_max - u_min);
                            let pt = uniform_surface.subs(u_param, v_param);
                            println!("  ({:.3}, {:.3}): [{:.3}, {:.3}, {:.3}]", 
                                u_param, v_param, pt.x, pt.y, pt.z);
                        }
                    }
                    
                    // Check multiple patches to see if they connect
                    println!("\n=== Checking patch connectivity ===");
                    let num_patches_to_check = 3.min(patch_table.patch_array_patches_len(0));
                    
                    for patch_idx in 0..num_patches_to_check {
                        println!("\nPatch {}:", patch_idx);
                        let start_idx = patch_idx * 16;
                        
                        // Get corner control points
                        let corners = [0, 3, 12, 15]; // corners of 4x4 grid
                        let mut corner_points = Vec::new();
                        
                        for &corner in &corners {
                            let vert_idx = patch_vertices[start_idx + corner].0 as usize;
                            if vert_idx < all_vertices.len() {
                                let v = &all_vertices[vert_idx];
                                corner_points.push(Point3::new(v[0] as f64, v[1] as f64, v[2] as f64));
                                println!("  Corner {}: vertex {} = [{:.3}, {:.3}, {:.3}]", 
                                    corner, vert_idx, v[0], v[1], v[2]);
                            }
                        }
                        
                        // Create surface and evaluate corners
                        if corner_points.len() == 4 {
                            // Build full control point grid for this patch
                            let mut patch_cps = vec![vec![Point3::new(0.0, 0.0, 0.0); 4]; 4];
                            for i in 0..16 {
                                let row = i / 4;
                                let col = i % 4;
                                let vert_idx = patch_vertices[start_idx + i].0 as usize;
                                if vert_idx < all_vertices.len() {
                                    let v = &all_vertices[vert_idx];
                                    patch_cps[row][col] = Point3::new(v[0] as f64, v[1] as f64, v[2] as f64);
                                }
                            }
                            
                            let patch_surface = BSplineSurface::new(
                                (clamped_knots.clone(), clamped_knots.clone()),
                                patch_cps
                            );
                            
                            println!("  Evaluated corners:");
                            println!("    (0,0): {:?}", patch_surface.subs(0.0, 0.0));
                            println!("    (1,0): {:?}", patch_surface.subs(1.0, 0.0));
                            println!("    (0,1): {:?}", patch_surface.subs(0.0, 1.0));
                            println!("    (1,1): {:?}", patch_surface.subs(1.0, 1.0));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "truck")]
#[test]
fn test_opensubdiv_knot_vectors() {
    use truck_geometry::prelude::{BSplineSurface, KnotVec, ParametricSurface};
    use truck_modeling::cgmath::Point3;
    
    // Test control points from a simple surface
    let control_points = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.5),
            Point3::new(2.0, 1.0, 0.5),
            Point3::new(3.0, 1.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(1.0, 2.0, 0.5),
            Point3::new(2.0, 2.0, 0.5),
            Point3::new(3.0, 2.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 3.0, 0.0),
            Point3::new(1.0, 3.0, 0.0),
            Point3::new(2.0, 3.0, 0.0),
            Point3::new(3.0, 3.0, 0.0),
        ],
    ];
    
    // Test different knot vectors
    println!("Testing different knot vectors for OpenSubdiv B-spline patches:\n");
    
    // 1. Bezier/Clamped knots (what we currently use)
    let bezier_knots = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    println!("1. Bezier/Clamped knots: {:?}", bezier_knots);
    
    // 2. Standard uniform B-spline knots
    let uniform_knots = KnotVec::uniform_knot(3, 4);
    println!("2. Uniform knots: {:?}", uniform_knots);
    
    // 3. Non-uniform knots that might match OpenSubdiv
    // OpenSubdiv might use knots that give the standard B-spline basis
    // but normalized to [0,1] parameter range
    let osd_knots_1 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 1.0/3.0, 2.0/3.0, 1.0, 1.0, 1.0, 1.0]);
    println!("3. Possible OSD knots 1: {:?}", osd_knots_1);
    
    // 4. Another possibility - interior knots at 0.5
    let osd_knots_2 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0]);
    println!("4. Possible OSD knots 2: {:?}", osd_knots_2);
    
    // Create surfaces with each knot vector
    let surfaces = vec![
        ("Bezier", BSplineSurface::new((bezier_knots.clone(), bezier_knots.clone()), control_points.clone())),
        ("Uniform", BSplineSurface::new((uniform_knots.clone(), uniform_knots.clone()), control_points.clone())),
        ("OSD1", BSplineSurface::new((osd_knots_1.clone(), osd_knots_1.clone()), control_points.clone())),
        ("OSD2", BSplineSurface::new((osd_knots_2.clone(), osd_knots_2.clone()), control_points.clone())),
    ];
    
    // Evaluate at key points
    println!("\nEvaluating at parameter values:");
    let test_params = vec![(0.0, 0.0), (0.5, 0.0), (1.0, 0.0), (0.5, 0.5), (1.0, 1.0)];
    
    for (u, v) in test_params {
        println!("\nAt ({}, {}):", u, v);
        for (name, surface) in &surfaces {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| surface.subs(u, v))) {
                Ok(point) => println!("  {}: {:?}", name, point),
                Err(_) => println!("  {}: <evaluation failed>", name),
            }
        }
    }
    
    // Check parameter ranges
    println!("\nParameter ranges:");
    for (name, surface) in &surfaces {
        println!("  {}: {:?}", name, surface.parameter_range());
    }
}

#[cfg(feature = "truck")]
#[test]
fn test_patch_extraction_order() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        AdaptiveRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    
    // Create a single quad to understand patch extraction
    let vertex_positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],  
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    
    let face_vertex_counts = vec![4];
    let face_vertex_indices = vec![0, 1, 3, 2]; // Counter-clockwise
    
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );
    
    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options).unwrap();
    
    // Use uniform refinement to ensure we get a patch
    use opensubdiv_petite::far::UniformRefinementOptions;
    let uniform_options = UniformRefinementOptions {
        refinement_level: 3,
        ..Default::default()
    };
    refiner.refine_uniform(uniform_options);
    
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    let patch_table = PatchTable::new(&refiner, Some(patch_options)).unwrap();
    
    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    let flat_positions: Vec<f32> = vertex_positions
        .iter()
        .flat_map(|v| v.iter().copied())
        .collect();
    
    let mut all_vertices = Vec::with_capacity(refiner.vertex_total_count());
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
    
    println!("Single quad test:");
    println!("Total vertices: {}", all_vertices.len());
    println!("Number of patch arrays: {}", patch_table.patch_arrays_len());
    
    // Print all vertices
    println!("\nAll vertices:");
    for (i, v) in all_vertices.iter().enumerate() {
        println!("  Vertex {}: [{:.3}, {:.3}, {:.3}]", i, v[0], v[1], v[2]);
        if i >= 20 { 
            println!("  ... ({} more vertices)", all_vertices.len() - i - 1);
            break;
        }
    }
    
    // Check if we have regular patches
    for i in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(i) {
            let num_patches = patch_table.patch_array_patches_len(i);
            println!("\nPatch array {}: type={:?}, num_patches={}", i, desc.patch_type(), num_patches);
            
            if desc.control_vertices_len() == 16 && num_patches > 0 {
                if let Some(patch_vertices) = patch_table.patch_array_vertices(i) {
                    // Print first patch
                    println!("\nFirst patch control point indices:");
                    for j in 0..16 {
                        let idx = patch_vertices[j].0 as usize;
                        if idx < all_vertices.len() {
                            let v = &all_vertices[idx];
                            println!("  CP{}: vertex {} = [{:.3}, {:.3}, {:.3}]", j, idx, v[0], v[1], v[2]);
                        }
                    }
                }
            }
        }
    }
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