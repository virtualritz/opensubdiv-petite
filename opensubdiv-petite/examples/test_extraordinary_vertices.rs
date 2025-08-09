//! Test example for extraordinary vertex handling in STEP export

use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
};

#[cfg(feature = "truck")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opensubdiv_petite::truck_integration::PatchTableExt;

    // Create a simple cube mesh - corners have valence 3 (extraordinary vertices)
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
    );

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // Check vertex valences
    if let Some(level0) = refiner.level(0) {
        println!("Base level topology:");
        println!("  Vertices: {}", level0.vertex_count());
        for v in 0..level0.vertex_count() {
            if let Some(edges) = level0.vertex_edges(opensubdiv_petite::Index(v as _)) {
                println!("  Vertex {} has valence {}", v, edges.len());
            }
        }
    }

    // Refine adaptively
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    println!("\nRefinement complete. Max level: {}", refiner.max_level());

    // Create patch table - try different end cap types
    println!("\nTesting different end cap types:");

    for end_cap in [EndCapType::GregoryBasis, EndCapType::BSplineBasis] {
        println!("\n--- Testing {:?} ---", end_cap);

        let patch_options = PatchTableOptions::new()
            .end_cap_type(end_cap)
            .use_inf_sharp_patch(false);

        let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

        // Count patch types
        let mut regular_count = 0;
        let mut gregory_basis_count = 0;
        let mut gregory_triangle_count = 0;
        let mut other_count = 0;

        for array_idx in 0..patch_table.patch_arrays_len() {
            if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
                let count = patch_table.patch_array_patches_len(array_idx);
                match desc.patch_type() {
                    opensubdiv_petite::far::PatchType::Regular => regular_count += count,
                    opensubdiv_petite::far::PatchType::GregoryBasis => gregory_basis_count += count,
                    opensubdiv_petite::far::PatchType::GregoryTriangle => {
                        gregory_triangle_count += count
                    }
                    _ => other_count += count,
                }
            }
        }

        println!("Patch counts:");
        println!("  Regular: {}", regular_count);
        println!("  GregoryBasis: {}", gregory_basis_count);
        println!("  GregoryTriangle: {}", gregory_triangle_count);
        println!("  Other: {}", other_count);

        // Build vertex buffer
        let primvar_refiner = PrimvarRefiner::new(&refiner);
        let mut all_vertices = Vec::with_capacity(refiner.vertex_total_count());

        // Add base vertices
        all_vertices.extend_from_slice(&vertex_positions);

        // Add refined vertices
        for level in 1..refiner.refinement_levels() {
            let src_data: Vec<f32> = all_vertices
                [(all_vertices.len() - refiner.level(level - 1).unwrap().vertex_count())..]
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
        }

        // Add local points if any
        let num_local_points = patch_table.local_point_count();
        if num_local_points > 0 {
            println!("Computing {} local points", num_local_points);
            if let Some(stencil_table) = patch_table.local_point_stencil_table() {
                let mut local_points = Vec::with_capacity(num_local_points);

                for dim in 0..3 {
                    let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();
                    let dst_dim = stencil_table.update_values(&src_dim, None, None);

                    for (i, &val) in dst_dim.iter().enumerate() {
                        if dim == 0 {
                            local_points.push([val, 0.0, 0.0]);
                        } else {
                            local_points[i][dim] = val;
                        }
                    }
                }

                all_vertices.extend_from_slice(&local_points);
            }
        }

        println!("Total vertices: {}", all_vertices.len());

        // Try to convert to truck shell
        match patch_table.to_truck_shell(&all_vertices) {
            Ok(_shell) => println!("Successfully converted to shell"),
            Err(e) => println!("Failed to convert to shell: {:?}", e),
        }
    }

    Ok(())
}

#[cfg(not(feature = "truck"))]
fn main() {
    println!("This example requires the 'truck' feature. Run with:");
    println!("  cargo run --example test_extraordinary_vertices --features truck");
}
