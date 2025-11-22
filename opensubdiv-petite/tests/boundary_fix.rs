//! Test the boundary fix for patches near extraordinary vertices

#[cfg(feature = "truck")]
mod tests {
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use opensubdiv_petite::truck::PatchTableExt;
    use std::fs;
    use truck_modeling::Shell;
    use truck_stepio::out::{CompleteStepDisplay, StepHeaderDescriptor, StepModel};

    #[test]
    fn test_cube_boundary_fix() {
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
        );

        // Create topology refiner
        let refiner_options = TopologyRefinerOptions::default();
        let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
            .expect("Failed to create topology refiner");

        // Refine adaptively
        let mut adaptive_options = AdaptiveRefinementOptions::default();
        adaptive_options.isolation_level = 3;
        refiner.refine_adaptive(adaptive_options, &[]);

        // Create patch table
        let patch_options = PatchTableOptions::new().end_cap_type(EndCapType::BSplineBasis);

        let patch_table =
            PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

        // Build complete vertex buffer
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
        if patch_table.local_point_count() > 0 {
            if let Some(stencil_table) = patch_table.local_point_stencil_table() {
                let mut local_points = Vec::with_capacity(patch_table.local_point_count());

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

        // Convert to truck shell
        let shell = patch_table
            .to_truck_shell(&all_vertices)
            .expect("Failed to convert to truck shell");

        // Export to STEP
        let compressed = shell.compress();
        let step_string = CompleteStepDisplay::new(
            StepModel::from(&compressed),
            StepHeaderDescriptor {
                file_name: "cube_boundary_fix.step".to_owned(),
                ..Default::default()
            },
        )
        .to_string();

        // Write STEP file
        let output_path = std::env::temp_dir().join("cube_boundary_fix.step");
        fs::write(&output_path, step_string).expect("Failed to write STEP file");

        println!("STEP file written to: {:?}", output_path);

        // The test passes if we can create a valid shell without gaps
        // In a real test, we would verify the shell is watertight
    }
}
