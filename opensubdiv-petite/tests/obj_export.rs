mod test_utils;

use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, PatchTable, PatchTableOptions, PrimvarRefiner, TopologyDescriptor,
    TopologyRefiner, TopologyRefinerOptions,
};
use std::fs::File;
use std::io::Write;

/// Build complete vertex buffer including all refinement levels
fn build_vertex_buffer(refiner: &TopologyRefiner, base_vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let primvar_refiner = PrimvarRefiner::new(refiner);
    let total_vertices = refiner.vertex_total_count();

    println!("Building vertex buffer:");
    println!("  Total vertices across all levels: {}", total_vertices);
    println!(
        "  Number of refinement levels: {}",
        refiner.refinement_levels()
    );

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    println!("  Level 0: {} vertices", base_vertices.len());
    all_vertices.extend_from_slice(base_vertices);

    // For each refinement level, interpolate from the PREVIOUS level only
    let num_levels = refiner.refinement_levels();
    let mut level_start = 0;

    for level in 1..num_levels {
        let prev_level_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);
        let level_verts = refiner.level(level).map(|l| l.vertex_count()).unwrap_or(0);
        println!(
            "  Level {}: {} vertices (interpolating from {} vertices at level {})",
            level,
            level_verts,
            prev_level_count,
            level - 1
        );

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
            println!("    Interpolated {} vertices", level_vertices.len());
            all_vertices.extend_from_slice(&level_vertices);
        }

        level_start += prev_level_count;
    }

    println!("  Final vertex buffer size: {}", all_vertices.len());
    all_vertices
}

/// Export patch control cages to OBJ format for visual inspection
fn export_patch_cages_to_obj(
    filename: &str,
    patch_table: &PatchTable,
    all_vertices: &[[f32; 3]],
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;

    writeln!(file, "# OpenSubdiv Patch Control Cages")?;
    writeln!(file, "# Number of patches: {}", patch_table.patches_len())?;
    writeln!(file, "#")?;

    let mut vertex_offset = 1; // OBJ uses 1-based indexing
    let mut patch_global_idx = 0;

    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
            let num_patches = patch_table.patch_array_patches_len(array_idx);

            for patch_idx in 0..num_patches {
                writeln!(
                    file,
                    "# Patch {} (array {}, local {})",
                    patch_global_idx, array_idx, patch_idx
                )?;

                let start = patch_idx * 16; // 16 CVs per regular patch

                // Write vertices for this patch
                for i in 0..16 {
                    let array_idx = start + i;
                    if array_idx < patch_vertices.len() {
                        let cv_idx = patch_vertices[array_idx].0 as usize;
                        if cv_idx < all_vertices.len() {
                            let v = &all_vertices[cv_idx];
                            writeln!(file, "v {} {} {}", v[0], v[1], v[2])?;
                        } else {
                            writeln!(file, "v 0 0 0  # ERROR: CV index {} out of bounds", cv_idx)?;
                        }
                    } else {
                        writeln!(file, "v 0 0 0  # ERROR: patch vertex index out of bounds")?;
                    }
                }

                // Write faces - connect control points as quads
                // Create a 3x3 grid of quads from the 4x4 control points
                for row in 0..3 {
                    for col in 0..3 {
                        let base = row * 4 + col;
                        let v1 = vertex_offset + base;
                        let v2 = vertex_offset + base + 1;
                        let v3 = vertex_offset + base + 5;
                        let v4 = vertex_offset + base + 4;
                        writeln!(file, "f {} {} {} {}", v1, v2, v3, v4)?;
                    }
                }

                writeln!(file)?; // Empty line between patches
                vertex_offset += 16;
                patch_global_idx += 1;
            }
        }
    }

    Ok(())
}

#[test]
fn test_export_simple_plane_patches() {
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
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options =
        PatchTableOptions::new().end_cap_type(opensubdiv_petite::far::EndCapType::BSplineBasis);
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    // Export to OBJ
    let output_path = test_utils::test_output_path("simple_plane_patches.obj");
    println!("Writing OBJ to: {:?}", output_path);
    export_patch_cages_to_obj(output_path.to_str().unwrap(), &patch_table, &all_vertices)
        .expect("Failed to export OBJ");

    // Compare or update expected results
    test_utils::assert_file_matches(&output_path, "simple_plane_patches.obj");
}

#[test]
fn test_export_simple_cube_patches() {
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
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options =
        PatchTableOptions::new().end_cap_type(opensubdiv_petite::far::EndCapType::BSplineBasis);
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    // Export to OBJ
    let output_path = test_utils::test_output_path("simple_cube_patches.obj");
    println!("Writing OBJ to: {:?}", output_path);
    export_patch_cages_to_obj(output_path.to_str().unwrap(), &patch_table, &all_vertices)
        .expect("Failed to export OBJ");

    // Compare or update expected results
    test_utils::assert_file_matches(&output_path, "simple_cube_patches.obj");
}

#[test]
fn test_export_creased_cube_patches() {
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
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options =
        PatchTableOptions::new().end_cap_type(opensubdiv_petite::far::EndCapType::BSplineBasis);
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

    // Export to OBJ
    let output_path = test_utils::test_output_path("creased_cube_patches.obj");
    println!("Writing OBJ to: {:?}", output_path);
    export_patch_cages_to_obj(output_path.to_str().unwrap(), &patch_table, &all_vertices)
        .expect("Failed to export OBJ");

    println!("Number of patches: {}", patch_table.patches_len());

    // Compare or update expected results
    test_utils::assert_file_matches(&output_path, "creased_cube_patches.obj");
}
