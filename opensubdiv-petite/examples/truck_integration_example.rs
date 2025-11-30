//! Example demonstrating OpenSubdiv to truck CAD kernel integration.

use anyhow::Result;
use opensubdiv_petite::far::{
    EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner, TopologyDescriptor, TopologyRefiner,
    TopologyRefinerOptions, UniformRefinementOptions,
};
#[cfg(feature = "truck")]
use opensubdiv_petite::truck::{bfr_regular_surfaces, PatchTableExt};

#[cfg(feature = "truck")]
use truck_stepio::out::{CompleteStepDisplay, StepModel};

fn main() -> Result<()> {
    #[cfg(not(feature = "truck"))]
    {
        println!("This example requires the 'truck' feature.");
        println!("Run with: cargo run --example truck_integration_example --features truck");
        return Ok(());
    }

    #[cfg(feature = "truck")]
    {
        println!("OpenSubdiv to Truck Integration Example");
        println!("======================================\n");

        // Cube topology
        let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
        let face_vertex_indices = vec![
            0, 1, 3, 2, // bottom
            2, 3, 5, 4, // front
            4, 5, 7, 6, // top
            6, 7, 1, 0, // back
            0, 2, 4, 6, // left
            1, 7, 5, 3, // right
        ];

        let vertex_positions = vec![
            [-1.0, -1.0, -1.0], // 0
            [1.0, -1.0, -1.0],  // 1
            [-1.0, -1.0, 1.0],  // 2
            [1.0, -1.0, 1.0],   // 3
            [-1.0, 1.0, 1.0],   // 4
            [1.0, 1.0, 1.0],    // 5
            [-1.0, 1.0, -1.0],  // 6
            [1.0, 1.0, -1.0],   // 7
        ];

        // Topology setup
        let descriptor = TopologyDescriptor::new(
            vertex_positions.len(),
            &face_vertex_counts,
            &face_vertex_indices,
        )?;

        let refiner_options = TopologyRefinerOptions::default();
        let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

        // Uniform refinement to level 2
        let mut uniform_options = UniformRefinementOptions::default();
        uniform_options.refinement_level = 2;
        refiner.refine_uniform(uniform_options);
        println!("Refined mesh to level {}", uniform_options.refinement_level);

        // Build primvars across levels
        let primvar_refiner = PrimvarRefiner::new(&refiner)?;
        let mut all_vertices = Vec::with_capacity(refiner.vertex_count_all_levels());
        all_vertices.extend_from_slice(&vertex_positions);

        for level in 1..refiner.refinement_levels() {
            let prev_count = refiner
                .level(level - 1)
                .map(|l| l.vertex_count())
                .unwrap_or(0);
            let start = all_vertices.len() - prev_count;
            let src_data: Vec<f32> = all_vertices[start..start + prev_count]
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

        // Build patch table with B-spline end caps
        let patch_options = PatchTableOptions::new().end_cap_type(EndCapType::BSplineBasis);
        let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

        println!(
            "\nPatch Table created: {} arrays, {} patches",
            patch_table.patch_array_count(),
            patch_table.patch_count()
        );

        // BFR regular faces (keeps coarse quads)
        let bfr_surfaces = bfr_regular_surfaces(&refiner, &all_vertices, 0, 0)
            .map(|s| s.len())
            .unwrap_or(0);
        println!("BFR produced {} coarse B-spline surfaces", bfr_surfaces);

        // Convert all patches to B-spline surfaces (PatchTable + BFR mixed)
        let surfaces = patch_table.to_truck_surfaces_bfr_mixed(&refiner, &all_vertices, 0, 0)?;
        println!(
            "Converted {} patches to B-spline surfaces (mixed BFR/PatchTable)",
            surfaces.len()
        );

        // Build stitched shell and export STEP
        let shell = patch_table.to_truck_shell_stitched(&all_vertices)?;
        let compressed = shell.compress();
        let step_string =
            CompleteStepDisplay::new(StepModel::from(&compressed), Default::default()).to_string();
        std::fs::write("truck_integration.step", step_string)?;
        println!("Wrote truck_integration.step");

        println!("\nTruck integration example completed!");
    }

    Ok(())
}
