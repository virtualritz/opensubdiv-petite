//! Test for exporting a single patch from simple cube to STEP format

mod utils;

#[cfg(feature = "truck")]
mod tests {
    use crate::utils::default_end_cap_type;
    use crate::utils::{assert_file_matches, test_output_path};
    use opensubdiv_petite::far::{
        PatchTable, PatchTableOptions, PrimvarRefiner, TopologyDescriptor, TopologyRefiner,
        TopologyRefinerOptions,
    };

    #[test]
    fn test_simple_cube_single_patch() -> anyhow::Result<()> {
        use opensubdiv_petite::truck::PatchTableExt;
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
        let face_indices_u32: Vec<u32> =
            face_vertices.iter().flatten().map(|&i| i as u32).collect();

        // Create topology descriptor
        let descriptor = TopologyDescriptor::new(
            vertex_positions.len(),
            &num_face_vertices, // vertices_per_face (counts)
            &face_indices_u32,  // vertex_indices_per_face (flattened indices)
        )?;

        // Create topology refiner
        let refiner_options = TopologyRefinerOptions::default();
        let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

        // Use adaptive refinement to get Regular patches
        use opensubdiv_petite::far::AdaptiveRefinementOptions;
        let mut adaptive_options = AdaptiveRefinementOptions::default();
        adaptive_options.isolation_level = 3;
        refiner.refine_adaptive(adaptive_options, &[]);

        // Build complete vertex buffer
        let primvar_refiner = PrimvarRefiner::new(&refiner)?;
        let total_vertices = refiner.vertex_count_all_levels();

        let mut all_vertices = Vec::with_capacity(total_vertices);
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

        // Create patch table
        let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());

        let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

        // Convert just the first patch to a B-spline surface
        let surfaces = patch_table.to_truck_surfaces(&all_vertices)?;

        if surfaces.is_empty() {
            panic!("No surfaces created!");
        }

        // Get the first surface
        let first_surface = surfaces.into_iter().next().unwrap();

        // Create a face from this single surface
        use truck_geometry::prelude::BSplineCurve;
        use truck_geometry::prelude::{KnotVec, ParametricSurface};
        use truck_modeling::{Curve, Edge, Face, Shell, Surface, Vertex, Wire};

        // Get the four corner points - for this knot configuration,
        // the valid surface is in the [1/3, 2/3] parameter range
        let p00 = first_surface.subs(1.0 / 3.0, 1.0 / 3.0);
        let p10 = first_surface.subs(2.0 / 3.0, 1.0 / 3.0);
        let p11 = first_surface.subs(2.0 / 3.0, 2.0 / 3.0);
        let p01 = first_surface.subs(1.0 / 3.0, 2.0 / 3.0);

        // Create vertices
        let v00 = Vertex::new(p00);
        let v10 = Vertex::new(p10);
        let v11 = Vertex::new(p11);
        let v01 = Vertex::new(p01);

        // Create edges with linear curves
        let e0 = Edge::new(
            &v00,
            &v10,
            Curve::BSplineCurve(BSplineCurve::new(KnotVec::bezier_knot(1), vec![p00, p10])),
        );
        let e1 = Edge::new(
            &v10,
            &v11,
            Curve::BSplineCurve(BSplineCurve::new(KnotVec::bezier_knot(1), vec![p10, p11])),
        );
        let e2 = Edge::new(
            &v11,
            &v01,
            Curve::BSplineCurve(BSplineCurve::new(KnotVec::bezier_knot(1), vec![p11, p01])),
        );
        let e3 = Edge::new(
            &v01,
            &v00,
            Curve::BSplineCurve(BSplineCurve::new(KnotVec::bezier_knot(1), vec![p01, p00])),
        );

        // Create wire and face
        let wire = Wire::from(vec![e0, e1, e2, e3]);
        let face = Face::new(vec![wire], Surface::BSplineSurface(first_surface));

        // Create a shell with just this one face
        let shell = Shell::from(vec![face]);

        // Compress and export the shell as STEP
        let compressed = shell.compress();

        // Write to STEP file
        let step_string = out::CompleteStepDisplay::new(
            out::StepModel::from(&compressed),
            out::StepHeaderDescriptor {
                file_name: "simple_cube_single_patch.step".to_owned(),
                ..Default::default()
            },
        )
        .to_string();

        // Write STEP file
        let step_path = test_output_path("simple_cube_single_patch.step");
        std::fs::write(&step_path, &step_string)?;

        // Compare or update expected results
        assert_file_matches(&step_path, "simple_cube_single_patch.step");
        Ok(())
    }
}
