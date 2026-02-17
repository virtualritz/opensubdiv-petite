mod utils;
use utils::*;

use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, PatchTable, PatchTableOptions, PrimvarRefiner, TopologyDescriptor,
    TopologyRefiner, TopologyRefinerOptions,
};
use opensubdiv_petite::iges_export::PatchTableIgesExt;

/// Build complete vertex buffer including all refinement levels
fn build_vertex_buffer(
    refiner: &TopologyRefiner,
    base_vertices: &[[f32; 3]],
) -> Result<Vec<[f32; 3]>, Box<dyn std::error::Error>> {
    let primvar_refiner = PrimvarRefiner::new(refiner)?;
    let total_vertices = refiner.vertex_count_all_levels();

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

    Ok(all_vertices)
}

#[test]
fn test_simple_plane_iges() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;

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
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions)?;

    println!(
        "Simple plane: {} patches, {} vertices",
        patch_table.patch_count(),
        all_vertices.len()
    );

    // Export to IGES
    let output_path = test_output_path("simple_plane.igs");
    patch_table
        .export_iges_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export IGES");

    // Compare or update expected results
    assert_file_matches(&output_path, "simple_plane.igs");
    Ok(())
}

#[test]
#[ignore = "IGES export needs to be reimplemented through truck for proper control vertex handling"]
fn test_simple_cube_iges() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;

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
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions)?;

    println!(
        "Simple cube: {} patches, {} vertices",
        patch_table.patch_count(),
        all_vertices.len()
    );

    // Export to IGES
    let output_path = test_output_path("simple_cube.igs");
    patch_table
        .export_iges_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export IGES");

    // Compare or update expected results
    assert_file_matches(&output_path, "simple_cube.igs");
    Ok(())
}

#[test]
#[ignore = "IGES export needs to be reimplemented through truck for proper control vertex handling"]
fn test_creased_cube_iges() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;
    descriptor.creases(&crease_indices, &crease_weights);

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
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions)?;

    println!(
        "Creased cube: {} patches, {} vertices",
        patch_table.patch_count(),
        all_vertices.len()
    );

    // Export to IGES
    let output_path = test_output_path("creased_cube.igs");
    patch_table
        .export_iges_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export IGES");

    // Compare or update expected results
    assert_file_matches(&output_path, "creased_cube.igs");
    Ok(())
}

#[test]
#[ignore = "IGES export needs to be reimplemented through truck for proper control vertex handling"]
fn test_two_patches_iges() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;

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
    let all_vertices = build_vertex_buffer(&refiner, &vertex_positions)?;

    println!(
        "Cube for two patches: {} patches, {} vertices",
        patch_table.patch_count(),
        all_vertices.len()
    );

    // Export only the first two patches by modifying the export
    let output_path = test_output_path("two_patches.igs");

    // For simplicity, we export all patches since IGES viewers can handle multiple
    // surfaces.
    patch_table
        .export_iges_file(output_path.to_str().unwrap(), &all_vertices)
        .expect("Failed to export IGES");

    // Compare or update expected results
    assert_file_matches(&output_path, "two_patches.igs");
    Ok(())
}
