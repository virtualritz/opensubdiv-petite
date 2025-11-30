mod utils;

#[cfg(feature = "truck")]
use utils::*;

#[cfg(feature = "truck")]
#[test]
fn test_simple_plane_surfaces_only() -> anyhow::Result<()> {
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use opensubdiv_petite::truck::PatchTableExt;
    use truck_modeling::*;
    use truck_stepio::out;

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
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // Use adaptive refinement
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table
    let patch_options = PatchTableOptions::new().end_cap_type(default_end_cap_type());
    let patch_table =
        PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner)?;
    let total_vertices = refiner.vertex_total_count();

    let mut all_vertices = Vec::with_capacity(total_vertices);

    // Add base level vertices
    all_vertices.extend_from_slice(&vertex_positions);

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

    // Convert patches to B-spline surfaces directly
    let surfaces = patch_table.to_truck_surfaces(&all_vertices)?;

    println!("Generated {} B-spline surfaces", surfaces.len());

    // Create a simple shell containing just the surfaces as trimmed surfaces
    // Each surface will be a face with its natural boundary
    let mut faces = Vec::new();
    for (i, surface) in surfaces.into_iter().enumerate() {
        if i < 3 {
            // Debug: print control points of first few surfaces
            println!("Surface {} control points:", i);
            use truck_geometry::prelude::ParametricSurface;
            for row in 0..4 {
                for col in 0..4 {
                    let cp = surface.control_point(row, col);
                    println!(
                        "  [{},{}] = ({:.3}, {:.3}, {:.3})",
                        row, col, cp.x, cp.y, cp.z
                    );
                }
            }
        }

        // Create a trimmed surface with the natural boundary of the B-spline
        // This uses the parameter domain [0,1] x [0,1]
        let face = Face::try_new(vec![], Surface::BSplineSurface(surface))?;
        faces.push(face);
    }

    let shell = Shell::from(faces);
    let compressed = shell.compress();

    // Write to STEP file
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            file_name: "simple_plane_surfaces_only.step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();

    // Write STEP file to test output directory
    let step_path = test_output_path("simple_plane_surfaces_only.step");
    std::fs::write(&step_path, &step_string)?;

    println!("Wrote STEP file to: {}", step_path.display());
    Ok(())
}
