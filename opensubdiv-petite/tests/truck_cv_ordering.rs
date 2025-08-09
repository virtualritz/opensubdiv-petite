mod test_utils;

#[cfg(feature = "truck")]
#[test]
fn test_simple_plane_cv_ordering() {
    use opensubdiv_petite::far::{
        EndCapType, PatchTable, PatchTableOptions, TopologyDescriptor, TopologyRefiner,
        TopologyRefinerOptions, UniformRefinementOptions,
    };

    // Create a 3x3 quad mesh (4x4 vertices)
    // The center patch should have CVs that match these vertex positions exactly
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
    use opensubdiv_petite::far::AdaptiveRefinementOptions;
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 2;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new();
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer with all refinement levels
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

    // Add refined vertices from each level
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

    println!("\n=== SIMPLE PLANE CV ORDERING TEST ===");
    println!("Created 3x3 quad mesh with 4x4 vertices");
    println!("Original vertex positions:");
    for y in (0..4).rev() {
        print!("  Row {}: ", y);
        for x in 0..4 {
            let idx = y * 4 + x;
            print!(
                "[{:.0},{:.0}] ",
                vertex_positions[idx][0], vertex_positions[idx][1]
            );
        }
        println!();
    }
    println!(
        "\nNumber of patches generated: {}",
        patch_table.patches_len()
    );

    // Print CV indices for each patch
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(patch_vertices) = patch_table.patch_array_vertices(array_idx) {
            let num_patches = patch_table.patch_array_patches_len(array_idx);

            for patch_idx in 0..num_patches {
                println!("\nPatch {}:", patch_idx);
                let start = patch_idx * 16;

                // Print in grid format with actual indices
                println!("  CV indices (as stored in array):");
                for i in 0..16 {
                    if i % 4 == 0 {
                        print!("    ");
                    }
                    print!("{:3} ", patch_vertices[start + i].0);
                    if i % 4 == 3 {
                        println!();
                    }
                }

                // Print in logical 4x4 grid (bottom-left origin)
                println!("\n  Logical grid (bottom-left origin):");
                for row in (0..4).rev() {
                    print!("    Row {}: ", row);
                    for col in 0..4 {
                        let i = row * 4 + col;
                        print!("{:3} ", patch_vertices[start + i].0);
                    }
                    println!();
                }

                // Print actual CV positions for first patch
                if patch_idx == 0 {
                    println!("\n  Center patch CV positions:");
                    for row in (0..4).rev() {
                        print!("    Row {}: ", row);
                        for col in 0..4 {
                            let i = row * 4 + col;
                            let cv_idx = patch_vertices[start + i].0 as usize;
                            if cv_idx < all_vertices.len() {
                                let v = &all_vertices[cv_idx];
                                print!("[{:.0},{:.0}] ", v[0], v[1]);
                            }
                        }
                        println!();
                    }
                }
            }
        }
    }

    // Check for shared vertices between patches
    if patch_table.patches_len() >= 2 {
        if let Some(patch_vertices) = patch_table.patch_array_vertices(0) {
            println!("\n=== CHECKING SHARED VERTICES ===");

            // Get CVs for first two patches
            let patch0_cvs: Vec<u32> = (0..16).map(|i| patch_vertices[i].0).collect();
            let patch1_cvs: Vec<u32> = (16..32).map(|i| patch_vertices[i].0).collect();

            println!(
                "\nPatch 0 right edge CVs (col 3): {:?}",
                vec![patch0_cvs[3], patch0_cvs[7], patch0_cvs[11], patch0_cvs[15]]
            );
            println!(
                "Patch 1 left edge CVs (col 0): {:?}",
                vec![patch1_cvs[0], patch1_cvs[4], patch1_cvs[8], patch1_cvs[12]]
            );

            // Check if they share vertices
            let shared = patch0_cvs[3] == patch1_cvs[0]
                && patch0_cvs[7] == patch1_cvs[4]
                && patch0_cvs[11] == patch1_cvs[8]
                && patch0_cvs[15] == patch1_cvs[12];

            println!("\nPatches share edge vertices: {}", shared);
        }
    }

    println!("\n=== END SIMPLE PLANE TEST ===\n");
}
