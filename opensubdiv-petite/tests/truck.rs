mod utils;

#[cfg(feature = "truck")]
use utils::*;

#[cfg(feature = "truck")]
#[test]
fn test_truck_integration_compiles() {
    use opensubdiv_petite::truck::TruckError;

    // Just verify the module compiles and types are accessible
    let _error: TruckError = TruckError::InvalidControlPoints;

    // This test passes if it compiles.
}

#[cfg(feature = "truck")]
#[test]
fn test_simple_plane_to_step() {
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use truck_stepio::out;

    // Create a 3x3 quad mesh (4x4 vertices)
    let mut vertex_positions = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            vertex_positions.push([x as f32, y as f32, 0.0]);
        }
    }

    // Create 3x3 quads
    let mut face_vertex_counts = Vec::new();
    let mut face_vertex_indices = Vec::new();

    for y in 0..3 {
        for x in 0..3 {
            face_vertex_counts.push(4);
            let base = y * 4 + x;
            face_vertex_indices.push(base);
            face_vertex_indices.push(base + 1);
            face_vertex_indices.push(base + 5);
            face_vertex_indices.push(base + 4);
        }
    }

    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    )
    .expect("Failed to create topology descriptor");

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let adaptive_options = AdaptiveRefinementOptions {
        isolation_level: 3,
        ..Default::default()
    };
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner).expect("Failed to create primvar refiner");
    let total_vertices = refiner.vertex_count_all_levels();

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    all_vertices.extend_from_slice(&vertex_positions);

    // For each refinement level, interpolate from the PREVIOUS level only
    let num_levels = refiner.refinement_levels();
    let mut level_start = 0;

    for level in 1..num_levels {
        let prev_level_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);
        let _level_verts = refiner.level(level).map(|l| l.vertex_count()).unwrap_or(0);

        // Get vertices from PREVIOUS level only
        let src_data: Vec<f32> = all_vertices[level_start..level_start + prev_level_count]
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();

        if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }

        level_start += prev_level_count;
    }

    // Check if patch table has local points that need to be appended
    let num_local_points = patch_table.local_point_count();

    // If there are local points, we need to evaluate them using the stencil table
    if num_local_points > 0 {
        if let Some(stencil_table) = patch_table.local_point_stencil_table() {
            // Apply stencils to compute local points (3 floats per point)
            let mut local_points = Vec::with_capacity(num_local_points);

            for dim in 0..3 {
                // Extract just this dimension from source vertices
                let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();

                // Apply stencils for this dimension
                let dst_dim = stencil_table.update_values(&src_dim, None, None);

                // Store results
                for (i, &val) in dst_dim.iter().enumerate() {
                    if dim == 0 {
                        local_points.push([val, 0.0, 0.0]);
                    } else {
                        local_points[i][dim] = val;
                    }
                }
            }

            // Append local points to the existing vertex buffer
            all_vertices.extend_from_slice(&local_points);
        }
    }

    // Convert patches to truck shell
    use opensubdiv_petite::truck::PatchTableExt;

    let shell = patch_table
        .to_truck_shell(&all_vertices)
        .expect("Failed to convert to truck shell");

    // Compress and export the shell as STEP
    let compressed = shell.compress();

    // Write to STEP file
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            file_name: "simple_plane.step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();

    // Write STEP file to test output directory
    let step_path = test_output_path("simple_plane.step");
    std::fs::write(&step_path, &step_string).expect("Failed to write STEP file");

    // Compare or update expected results
    assert_file_matches(&step_path, "simple_plane.step");
}

#[cfg(feature = "truck")]
#[test]
fn test_simple_cube_to_step() {
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use truck_stepio::out;

    // Simple cube vertices
    let vertex_positions = vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 1, 3, 2, // back
        2, 3, 5, 4, // top
        4, 5, 7, 6, // front
        6, 7, 1, 0, // bottom
        0, 2, 4, 6, // left
        1, 7, 5, 3, // right
    ];

    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    )
    .expect("Failed to create topology descriptor");

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let adaptive_options = AdaptiveRefinementOptions {
        isolation_level: 3,
        ..Default::default()
    };
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner).expect("Failed to create primvar refiner");
    let total_vertices = refiner.vertex_count_all_levels();

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    all_vertices.extend_from_slice(&vertex_positions);

    // For each refinement level, interpolate from the PREVIOUS level only
    let num_levels = refiner.refinement_levels();
    let mut level_start = 0;

    for level in 1..num_levels {
        let prev_level_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);
        let _level_verts = refiner.level(level).map(|l| l.vertex_count()).unwrap_or(0);

        // Get vertices from PREVIOUS level only
        let src_data: Vec<f32> = all_vertices[level_start..level_start + prev_level_count]
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();

        if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }

        level_start += prev_level_count;
    }

    // Check if patch table has local points that need to be appended
    let num_local_points = patch_table.local_point_count();

    // If there are local points, we need to evaluate them using the stencil table
    if num_local_points > 0 {
        if let Some(stencil_table) = patch_table.local_point_stencil_table() {
            // Apply stencils to compute local points (3 floats per point)
            let mut local_points = Vec::with_capacity(num_local_points);

            for dim in 0..3 {
                // Extract just this dimension from source vertices
                let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();

                // Apply stencils for this dimension
                let dst_dim = stencil_table.update_values(&src_dim, None, None);

                // Store results
                for (i, &val) in dst_dim.iter().enumerate() {
                    if dim == 0 {
                        local_points.push([val, 0.0, 0.0]);
                    } else {
                        local_points[i][dim] = val;
                    }
                }
            }

            // Append local points to the existing vertex buffer
            all_vertices.extend_from_slice(&local_points);
        }
    }

    // Convert patches to truck shell
    use opensubdiv_petite::truck::PatchTableExt;

    let shell = patch_table
        .to_truck_shell(&all_vertices)
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

    // Write STEP file to test output directory
    let step_path = test_output_path("simple_cube.step");
    std::fs::write(&step_path, &step_string).expect("Failed to write STEP file");

    // Compare or update expected results
    assert_file_matches(&step_path, "simple_cube.step");
}

#[cfg(feature = "truck")]
#[test]
fn test_creased_cube_to_step() {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use truck_stepio::out;

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
        0, 1, 3, 2, // front
        2, 3, 5, 4, // top
        4, 5, 7, 6, // back
        6, 7, 1, 0, // bottom
        0, 2, 4, 6, // left
        1, 7, 5, 3, // right
    ];

    // Define creases
    let crease_indices = vec![
        0, 1, // bottom front edge
        1, 3, // right front edge
        3, 2, // top front edge
        2, 0, // left front edge
    ];
    let crease_weights = vec![2.0, 2.0, 2.0, 2.0];

    // Create topology descriptor
    let mut descriptor = TopologyDescriptor::new(
        vertex_positions.len(), // Number of vertices, not faces
        &face_vertex_counts,
        &face_vertex_indices,
    )
    .expect("Failed to create topology descriptor");
    descriptor.creases(&crease_indices, &crease_weights);

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();

    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement to generate B-spline patches
    // Based on OpenSubdiv docs, adaptive refinement isolates irregular features
    // and generates B-spline patches for regular regions
    use opensubdiv_petite::far::AdaptiveRefinementOptions;
    let adaptive_options = AdaptiveRefinementOptions {
        isolation_level: 2,
        ..Default::default()
    }; // Refine to isolate irregular vertices

    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table with B-spline patches for higher-order surfaces
    use opensubdiv_petite::far::PatchTableOptions;
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build complete vertex buffer including all refinement levels
    use opensubdiv_petite::far::PrimvarRefiner;
    let primvar_refiner = PrimvarRefiner::new(&refiner).expect("Failed to create primvar refiner");
    let total_vertices = refiner.vertex_count_all_levels();

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    all_vertices.extend_from_slice(&vertex_positions);

    // For each refinement level, interpolate from the PREVIOUS level only
    let num_levels = refiner.refinement_levels();
    let mut level_start = 0;

    for level in 1..num_levels {
        let prev_level_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);
        let _level_verts = refiner.level(level).map(|l| l.vertex_count()).unwrap_or(0);

        // Get vertices from PREVIOUS level only
        let src_data: Vec<f32> = all_vertices[level_start..level_start + prev_level_count]
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();

        if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }

        level_start += prev_level_count;
    }

    // Check if patch table has local points that need to be appended
    let num_local_points = patch_table.local_point_count();

    // If there are local points, we need to evaluate them using the stencil table
    if num_local_points > 0 {
        if let Some(stencil_table) = patch_table.local_point_stencil_table() {
            // Apply stencils to compute local points (3 floats per point)
            let mut local_points = Vec::with_capacity(num_local_points);

            for dim in 0..3 {
                // Extract just this dimension from source vertices
                let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();

                // Apply stencils for this dimension
                let dst_dim = stencil_table.update_values(&src_dim, None, None);

                // Store results
                for (i, &val) in dst_dim.iter().enumerate() {
                    if dim == 0 {
                        local_points.push([val, 0.0, 0.0]);
                    } else {
                        local_points[i][dim] = val;
                    }
                }
            }

            // Append local points to the existing vertex buffer
            all_vertices.extend_from_slice(&local_points);
        }
    }

    // Convert patches to truck shell
    use opensubdiv_petite::truck::PatchTableExt;

    let shell = patch_table
        .to_truck_shell(&all_vertices)
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

    // Write STEP file to test output directory
    let step_path = test_output_path("creased_cube.step");
    std::fs::write(&step_path, &step_string).expect("Failed to write STEP file");

    // Compare or update expected results
    assert_file_matches(&step_path, "creased_cube.step");
}
