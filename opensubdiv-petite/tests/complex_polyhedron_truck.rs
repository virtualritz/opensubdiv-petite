//! Test for complex polyhedron operations with STEP export via truck.

mod test_utils;

#[cfg(feature = "truck")]
mod tests {
    use crate::test_utils::*;
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use std::f32::consts::PI;

    /// Create vertices for an icosahedron.
    fn create_icosahedron_vertices() -> Vec<[f32; 3]> {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
        let a = 1.0;
        let b = 1.0 / phi;

        // Normalize to unit sphere
        let scale = 1.0 / (a * a + b * b).sqrt();
        let a = a * scale;
        let b = b * scale;

        vec![
            // Vertices of the icosahedron
            [0.0, b, -a],
            [b, a, 0.0],
            [-b, a, 0.0],
            [0.0, b, a],
            [0.0, -b, a],
            [-a, 0.0, b],
            [0.0, -b, -a],
            [a, 0.0, -b],
            [a, 0.0, b],
            [-a, 0.0, -b],
            [b, -a, 0.0],
            [-b, -a, 0.0],
        ]
    }

    /// Create faces for an icosahedron.
    fn create_icosahedron_faces() -> (Vec<u32>, Vec<u32>) {
        let face_vertex_counts = vec![3; 20]; // 20 triangular faces

        let face_indices = vec![
            0, 1, 2, 0, 2, 9, 9, 2, 5, 5, 2, 3, 2, 1, 3, 3, 1, 8, 1, 0, 7, 0, 9, 7, 9, 5, 11, 5, 3,
            4, 3, 8, 4, 8, 1, 7, 4, 8, 10, 8, 7, 10, 7, 9, 6, 9, 11, 6, 11, 5, 4, 11, 4, 10, 6, 11,
            10, 6, 10, 7,
        ];

        (face_vertex_counts, face_indices)
    }

    /// Apply a twist operation to vertices.
    fn apply_twist(vertices: &mut [[f32; 3]], angle: f32) {
        for vertex in vertices.iter_mut() {
            let y = vertex[1];
            let twist_angle = angle * y;
            let cos_a = twist_angle.cos();
            let sin_a = twist_angle.sin();

            let x = vertex[0];
            let z = vertex[2];

            vertex[0] = x * cos_a - z * sin_a;
            vertex[2] = x * sin_a + z * cos_a;
        }
    }

    /// Apply a bulge operation to vertices.
    fn apply_bulge(vertices: &mut [[f32; 3]], factor: f32) {
        for vertex in vertices.iter_mut() {
            let scale = 1.0 + factor * (1.0 - vertex[1].abs());

            vertex[0] *= scale;
            vertex[2] *= scale;
        }
    }

    /// Apply a shear operation to vertices.
    fn apply_shear(vertices: &mut [[f32; 3]], shear_x: f32, shear_z: f32) {
        for vertex in vertices.iter_mut() {
            let y = vertex[1];
            vertex[0] += shear_x * y;
            vertex[2] += shear_z * y;
        }
    }

    /// Build complete vertex buffer including all refinement levels.
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
    fn test_complex_polyhedron_to_step() {
        use opensubdiv_petite::truck_integration::PatchTableExt;
        use truck_stepio::out;

        // Create icosahedron base geometry
        let mut vertex_positions = create_icosahedron_vertices();
        let (face_vertex_counts, face_vertex_indices) = create_icosahedron_faces();

        // Apply three operations to create a complex shape
        // 1. Twist operation
        apply_twist(&mut vertex_positions, PI / 4.0);

        // 2. Bulge operation
        apply_bulge(&mut vertex_positions, 0.3);

        // 3. Shear operation
        apply_shear(&mut vertex_positions, 0.2, -0.1);

        // Scale up the model
        for vertex in vertex_positions.iter_mut() {
            vertex[0] *= 2.0;
            vertex[1] *= 2.0;
            vertex[2] *= 2.0;
        }

        // Define creases with weight 4 as requested
        // We'll add creases along some edges to create sharp features
        let crease_indices = vec![
            0, 1, // Edge from vertex 0 to 1
            1, 8, // Edge from vertex 1 to 8
            8, 4, // Edge from vertex 8 to 4
            4, 5, // Edge from vertex 4 to 5
            5, 9, // Edge from vertex 5 to 9
            9, 0, // Edge from vertex 9 to 0 (closing a loop)
            // Add another crease loop
            2, 3, // Edge from vertex 2 to 3
            3, 4, // Edge from vertex 3 to 4
            4, 11, // Edge from vertex 4 to 11
            11, 2, // Edge from vertex 11 to 2 (closing another loop)
        ];
        let crease_weights = vec![4.0; crease_indices.len() / 2];

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
        let mut adaptive_options = AdaptiveRefinementOptions::default();
        adaptive_options.isolation_level = 3;
        refiner.refine_adaptive(adaptive_options, &[]);

        // Create patch table
        let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
        let patch_table =
            PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

        println!(
            "Complex polyhedron: {} patches from {} base vertices and {} faces",
            patch_table.patches_len(),
            vertex_positions.len(),
            face_vertex_counts.len()
        );

        // Build vertex buffer
        let mut all_vertices = build_vertex_buffer(&refiner, &vertex_positions);

        println!("Total vertices after refinement: {}", all_vertices.len());

        // Check if patch table has local points that need to be appended
        let num_local_points = patch_table.local_point_count();
        
        // If there are local points, we need to evaluate them using the stencil table
        if num_local_points > 0 {
            if let Some(stencil_table) = patch_table.local_point_stencil_table() {
                // Apply stencils to compute local points (3 floats per point)
                let mut local_points = Vec::with_capacity(num_local_points);
                
                for dim in 0..3 {
                    // Extract just this dimension from source vertices
                    let src_dim: Vec<f32> = all_vertices
                        .iter()
                        .map(|v| v[dim])
                        .collect();
                        
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
                
                println!("Added {} local points, total vertices: {}", num_local_points, all_vertices.len());
            }
        }

        // Convert patches to truck shell
        let shell = patch_table
            .to_truck_shell(&all_vertices)
            .expect("Failed to convert to truck shell");

        // Compress and export the shell as STEP
        let compressed = shell.compress();

        // Write to STEP file
        let step_string = out::CompleteStepDisplay::new(
            out::StepModel::from(&compressed),
            out::StepHeaderDescriptor {
                file_name: "complex_polyhedron_crease4.step".to_owned(),
                ..Default::default()
            },
        )
        .to_string();

        // Write STEP file to test output directory
        let step_path = test_output_path("complex_polyhedron_crease4.step");
        std::fs::write(&step_path, &step_string).expect("Failed to write STEP file");

        println!("Successfully generated {}", step_path.display());
        
        // Compare with expected file
        assert_file_matches(&step_path, "complex_polyhedron_crease4.step");

        // Also export an OBJ file for visualization comparison
        let obj_path = test_output_path("complex_polyhedron_crease4.obj");
        let mut obj_file = std::fs::File::create(&obj_path).expect("Failed to create OBJ file");

        use std::io::Write;
        writeln!(
            obj_file,
            "# Complex polyhedron with twist, bulge, and shear operations"
        )
        .unwrap();
        writeln!(
            obj_file,
            "# {} vertices, {} faces",
            all_vertices.len(),
            patch_table.patches_len()
        )
        .unwrap();
        writeln!(obj_file, "# Creases with weight 4.0").unwrap();
        writeln!(obj_file).unwrap();

        // Write vertices
        for vertex in &all_vertices {
            writeln!(obj_file, "v {} {} {}", vertex[0], vertex[1], vertex[2]).unwrap();
        }

        println!("Also generated OBJ file: {}", obj_path.display());
    }
}
