mod test_utils;
use test_utils::*;

use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
};
use opensubdiv_petite::obj_bspline_export::PatchTableObjExt;

/// Build complete vertex buffer including all refinement levels
fn build_vertex_buffer(refiner: &TopologyRefiner, base_vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let primvar_refiner = PrimvarRefiner::new(refiner);
    let total_vertices = refiner.vertex_total_count();

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    all_vertices.extend_from_slice(base_vertices);

    // For each refinement level, interpolate from the PREVIOUS level only
    let num_levels = refiner.refinement_levels();
    let mut level_start = 0;

    for level in 1..num_levels {
        let prev_level_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);

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

    all_vertices
}

#[test]
fn test_simple_plane_bspline_obj() {
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
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    println!(
        "Simple plane: {} patches, {} vertices",
        patch_table.patches_len(),
        all_vertices.len()
    );

    // Export to OBJ
    let output_path = test_output_path("simple_plane_bspline.obj");
    patch_table
        .export_obj_bspline_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export OBJ");

    // Compare or update expected results
    assert_file_matches(&output_path, "simple_plane_bspline.obj");
}

#[test]
fn test_simple_cube_bspline_obj() {
    // Simple cube vertices
    let vertex_positions = vec![
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 2, 3, 1, // front face (-z)
        2, 6, 7, 3, // top face (+y)
        6, 4, 5, 7, // back face (+z)
        4, 0, 1, 5, // bottom face (-y)
        4, 6, 2, 0, // left face (-x)
        1, 3, 7, 5, // right face (+x)
    ];

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
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    println!(
        "Simple cube: {} patches, {} vertices",
        patch_table.patches_len(),
        all_vertices.len()
    );

    // Export to OBJ
    let output_path = test_output_path("simple_cube_bspline.obj");
    patch_table
        .export_obj_bspline_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export OBJ");

    // Compare or update expected results
    assert_file_matches(&output_path, "simple_cube_bspline.obj");
}

#[test]
fn test_creased_cube_bspline_obj() {
    // Creased cube vertices
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

    let mut descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );
    descriptor.creases(&crease_indices, &crease_weights);

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    println!(
        "Creased cube: {} patches, {} vertices",
        patch_table.patches_len(),
        all_vertices.len()
    );

    // Export to OBJ
    let output_path = test_output_path("creased_cube_bspline.obj");
    patch_table
        .export_obj_bspline_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export OBJ");

    // Compare or update expected results
    assert_file_matches(&output_path, "creased_cube_bspline.obj");
}

#[test]
fn test_two_patches_bspline_obj() {
    // Simple cube - same as in truck.rs test
    let vertex_positions = vec![
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 2, 3, 1, // front face (-z)
        2, 6, 7, 3, // top face (+y)
        6, 4, 5, 7, // back face (+z)
        4, 0, 1, 5, // bottom face (-y)
        4, 6, 2, 0, // left face (-x)
        1, 3, 7, 5, // right face (+x)
    ];

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
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    println!(
        "Cube for two patches: {} patches, {} vertices",
        patch_table.patches_len(),
        all_vertices.len()
    );

    // Export only the first two patches
    let output_path = test_output_path("two_patches_bspline.obj");
    let mut file = std::fs::File::create(&output_path).unwrap();

    use std::io::Write;
    writeln!(
        file,
        "# OpenSubdiv B-spline Surface Export - First Two Patches Only"
    )
    .unwrap();
    writeln!(file, "# Generated by opensubdiv-petite").unwrap();
    writeln!(file, "#").unwrap();

    // Write all control points
    writeln!(file, "# Control points").unwrap();
    for (i, cp) in all_vertices.iter().enumerate() {
        writeln!(file, "v {} {} {}  # vertex {}", cp[0], cp[1], cp[2], i).unwrap();
    }
    writeln!(file).unwrap();

    // Export only first two patches
    let mut patch_count = 0;
    'outer: for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            if desc.patch_type() != opensubdiv_petite::far::PatchType::Regular {
                continue;
            }

            if let Some(cv_indices) = patch_table.patch_array_vertices(array_idx) {
                const REGULAR_PATCH_SIZE: usize = 16;

                for patch_idx in 0..patch_table.patch_array_patches_len(array_idx) {
                    if patch_count >= 2 {
                        break 'outer;
                    }

                    writeln!(
                        file,
                        "# Patch {} (array {}, local {})",
                        patch_count, array_idx, patch_idx
                    )
                    .unwrap();

                    // Write B-spline surface
                    writeln!(file, "cstype bspline").unwrap();
                    writeln!(file, "deg 3 3").unwrap();

                    let start = patch_idx * REGULAR_PATCH_SIZE;
                    let patch_cvs = &cv_indices[start..start + REGULAR_PATCH_SIZE];

                    write!(file, "surf 0.0 1.0 0.0 1.0").unwrap();
                    for i in 0..16 {
                        let cv_idx = patch_cvs[i].0 as usize + 1; // 1-based
                        write!(file, " {}", cv_idx).unwrap();
                    }
                    writeln!(file).unwrap();

                    writeln!(file, "parm u -3.0 -2.0 -1.0 0.0 1.0 2.0 3.0 4.0").unwrap();
                    writeln!(file, "parm v -3.0 -2.0 -1.0 0.0 1.0 2.0 3.0 4.0").unwrap();
                    writeln!(file, "end").unwrap();
                    writeln!(file).unwrap();

                    patch_count += 1;
                }
            }
        }
    }

    // Compare or update expected results
    assert_file_matches(&output_path, "two_patches_bspline.obj");
}
