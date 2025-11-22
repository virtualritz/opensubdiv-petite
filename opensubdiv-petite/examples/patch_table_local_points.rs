//! Demonstrates working with patch tables and local point stencil tables.
//!
//! This example shows how to:
//! - Create a patch table with adaptive refinement.
//! - Access local points generated for irregular patches.
//! - Use local point stencil tables to compute local point positions.
//!
//! Local points are additional vertices created by OpenSubdiv for patches
//! that cannot be represented with regular B-spline or Bezier patches.
//! These include patches near extraordinary vertices or boundaries.

use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple cube mesh
    let vertex_positions = vec![
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [-0.5, 0.5, -0.5],  // 2
        [0.5, 0.5, -0.5],   // 3
        [-0.5, 0.5, 0.5],   // 4
        [0.5, 0.5, 0.5],    // 5
        [-0.5, -0.5, 0.5],  // 6
        [0.5, -0.5, 0.5],   // 7
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
    )?;

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // Refine adaptively
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    println!("Refinement complete. Max level: {}", refiner.max_level());

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(EndCapType::BSplineBasis);

    let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

    println!(
        "Patch table created with {} patches",
        patch_table.patch_count()
    );

    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner)?;
    let mut all_vertices = Vec::with_capacity(refiner.vertex_count_all_levels());

    // Add base vertices
    all_vertices.extend_from_slice(&vertex_positions);
    println!("Added {} base vertices", vertex_positions.len());

    // Add refined vertices
    for level in 1..refiner.refinement_levels() {
        let prev_level = refiner.level(level - 1).unwrap();
        let prev_count = prev_level.vertex_count();
        let src_start = all_vertices.len() - prev_count;

        let src_data: Vec<f32> = all_vertices[src_start..]
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();

        println!(
            "Level {}: interpolating from {} vertices",
            level, prev_count
        );

        if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            println!("  Generated {} vertices", level_vertices.len());
            all_vertices.extend_from_slice(&level_vertices);
        }
    }

    println!("Total vertices after refinement: {}", all_vertices.len());

    // Check local points
    let num_local_points = patch_table.local_point_count();
    println!("Patch table has {} local points", num_local_points);

    if num_local_points > 0 {
        if let Some(stencil_table) = patch_table.local_point_stencil_table() {
            println!("Stencil table info:");
            println!("  Number of stencils: {}", stencil_table.len());

            // Get the control vertex count - this should match our refined vertex buffer
            let control_vertex_count = stencil_table.control_vertex_count();
            println!("  Control vertex count: {}", control_vertex_count);
            println!("  Current vertex buffer size: {}", all_vertices.len());

            // The control_vertex_count being 0 indicates the stencil table was created
            // for local points only and expects the refined vertices to be provided
            // separately
            if control_vertex_count == 0 {
                println!("INFO: Stencil table appears to be for local points only");
                println!(
                    "      It expects refined vertices as input (count: {})",
                    all_vertices.len()
                );

                // For local point stencils, the source should be the refined vertices
                // and the output will be the local points
                println!(
                    "\nApplying stencils to compute {} local points...",
                    num_local_points
                );

                // Flatten the vertex data for each dimension
                for dim in 0..3 {
                    let dim_name = ["X", "Y", "Z"][dim];
                    let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();
                    println!(
                        "  Processing dimension {} with {} source values",
                        dim_name,
                        src_dim.len()
                    );

                    // Apply stencils - this computes the local points from refined vertices
                    let dst_dim = stencil_table.update_values(&src_dim, None, None);
                    println!("    Generated {} local point values", dst_dim.len());

                    if dst_dim.len() != num_local_points {
                        println!(
                            "    WARNING: Expected {} local points but got {}",
                            num_local_points,
                            dst_dim.len()
                        );
                    }
                }

                println!("\nSuccessfully computed local points!");
            } else if control_vertex_count != all_vertices.len() {
                println!(
                    "ERROR: Stencil table expects {} control vertices but we have {}",
                    control_vertex_count,
                    all_vertices.len()
                );
                println!("This mismatch indicates a problem with vertex buffer construction!");
                return Err("Vertex count mismatch".into());
            } else {
                // Normal case - apply stencils
                println!("\nApplying stencils for dimension 0...");
                let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[0]).collect();
                println!("  Source array size: {}", src_dim.len());

                let dst_dim = stencil_table.update_values(&src_dim, None, None);
                println!("  Output array size: {}", dst_dim.len());

                println!("Successfully applied stencils!");
            }
        } else {
            println!("No local point stencil table available");
        }
    } else {
        println!("No local points in patch table");
    }

    Ok(())
}
