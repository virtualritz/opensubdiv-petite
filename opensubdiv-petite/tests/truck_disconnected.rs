//! Tests for exporting patches as disconnected surfaces to STEP format

mod test_utils;

#[cfg(feature = "truck")]
mod tests {
    use crate::test_utils::default_end_cap_type;
    use opensubdiv_petite::far::{
        EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner, TopologyDescriptor,
        TopologyRefiner, TopologyRefinerOptions, UniformRefinementOptions,
    };
    use opensubdiv_petite::Index;
    use std::path::PathBuf;

    fn test_output_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        path.push(filename);
        path
    }

    /// Build complete vertex buffer including all refinement levels
    fn build_vertex_buffer(refiner: &TopologyRefiner, base_vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
        println!("Building vertex buffer:");

        let num_levels = refiner.refinement_levels();
        println!("  Number of refinement levels: {}", num_levels);

        // Calculate total vertices needed
        let mut total_vertices = base_vertices.len();
        for level in 1..=num_levels {
            total_vertices += refiner.level(level).unwrap().vertices_len();
        }
        println!("  Total vertices across all levels: {}", total_vertices);

        // Allocate buffer for all vertices
        let mut all_vertices = Vec::with_capacity(total_vertices);
        all_vertices.extend_from_slice(base_vertices);
        println!("  Level 0: {} vertices", base_vertices.len());

        // Refine vertices level by level
        let mut level_start = 0;
        let mut prev_level_count = base_vertices.len();

        for level in 1..=num_levels {
            let level_obj = refiner.level(level).unwrap();
            let level_count = level_obj.vertices_len();

            println!(
                "  Level {}: {} vertices (interpolating from {} vertices at level {})",
                level,
                level_count,
                prev_level_count,
                level - 1
            );

            // Allocate vertices for this level
            let mut level_vertices = vec![[0.0f32; 3]; level_count];

            // Build flat source data from PREVIOUS level only
            let src_data: Vec<f32> = all_vertices[level_start..level_start + prev_level_count]
                .iter()
                .flat_map(|v| v.iter().copied())
                .collect();

            // Build flat destination data
            let mut dst_data = vec![0.0f32; level_count * 3];

            // Create a primvar refiner and interpolate
            let primvar_refiner = PrimvarRefiner::new(&refiner);
            primvar_refiner.interpolate(
                level,
                &src_data,
                &mut dst_data,
                0, // channel
                prev_level_count,
                level_count,
            );

            // Convert back to vertex array
            for (i, vertex) in level_vertices.iter_mut().enumerate() {
                vertex[0] = dst_data[i * 3];
                vertex[1] = dst_data[i * 3 + 1];
                vertex[2] = dst_data[i * 3 + 2];
            }

            println!("    Interpolated {} vertices", level_vertices.len());
            all_vertices.extend_from_slice(&level_vertices);

            level_start += prev_level_count;
            prev_level_count = level_count;
        }

        println!("  Final vertex buffer size: {}", all_vertices.len());
        all_vertices
    }

    #[test]
    #[ignore] // Currently needs truck_integration method to_truck_shells
    fn test_simple_cube_disconnected_patches() {
        use opensubdiv_petite::truck_integration::PatchTableExt;
        use truck_stepio::out;

        // Define simple cube vertices
        let vertex_positions = vec![
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, -0.5, -0.5],
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        ];

        // Define cube faces (quads)
        let face_vertices = vec![
            vec![0, 1, 5, 4], // Front
            vec![2, 3, 7, 6], // Back
            vec![0, 4, 7, 3], // Left
            vec![1, 2, 6, 5], // Right
            vec![0, 3, 2, 1], // Bottom
            vec![4, 5, 6, 7], // Top
        ];

        // Flatten face data
        let num_face_vertices = face_vertices
            .iter()
            .map(|f| f.len() as u32)
            .collect::<Vec<_>>();
        let face_indices = face_vertices
            .iter()
            .flatten()
            .map(|&i| Index::from(i as u32))
            .collect::<Vec<_>>();

        // Create topology descriptor
        let descriptor =
            TopologyDescriptor::new(vertex_positions.len(), face_indices, num_face_vertices);

        // Create topology refiner with uniform refinement
        let uniform_options = UniformRefinementOptions::new(3);
        let refiner_options = TopologyRefinerOptions::new_uniform(uniform_options);

        let refiner = TopologyRefiner::new(descriptor, refiner_options)
            .expect("Failed to create topology refiner");

        // Build complete vertex buffer
        let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

        // Create patch table
        let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());

        let patch_table =
            PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

        println!("Number of patches: {}", patch_table.patches_len());

        // Convert patches to individual shells
        let shells = patch_table
            .to_truck_shells(&all_vertices)
            .expect("Failed to convert to truck shells");

        println!("Created {} individual shells", shells.len());

        // Compress all shells
        let compressed_shells: Vec<_> = shells.iter().map(|shell| shell.compress()).collect();

        // Create the STEP export
        // We'll export the first shell and then append the others
        if compressed_shells.is_empty() {
            panic!("No shells to export");
        }

        // For now, just export the first few shells as a test
        let shells_to_export = compressed_shells.into_iter().take(10).collect::<Vec<_>>();

        // Create a combined model with multiple shells
        // Each shell will be a separate SHELL_BASED_SURFACE_MODEL in the STEP file
        let step_string = shells_to_export
            .iter()
            .enumerate()
            .map(|(i, shell)| {
                let model = out::StepModel::from(shell);
                if i == 0 {
                    // First shell includes the header
                    out::CompleteStepDisplay::new(
                        model,
                        out::StepHeaderDescriptor {
                            file_name: "simple_cube_disconnected.step".to_owned(),
                            ..Default::default()
                        },
                    )
                    .to_string()
                } else {
                    // Other shells just add their data
                    format!("# Shell {}\n{}", i, model)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Write STEP file
        let step_path = test_output_path("simple_cube_disconnected.step");
        std::fs::write(&step_path, &step_string).expect("Failed to write STEP file");

        println!("Successfully generated {}", step_path.display());
    }
}
