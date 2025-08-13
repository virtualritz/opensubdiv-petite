//! Example demonstrating OpenSubdiv to truck CAD kernel integration

use opensubdiv_petite::far::{
    EndCapType, PatchTable, PatchTableOptions, TopologyDescriptor, TopologyRefiner,
    TopologyRefinerOptions,
};

#[cfg(feature = "truck_integration")]
use opensubdiv_petite::truck_integration::PatchTableExt;
#[cfg(feature = "truck_integration")]
use std::convert::TryInto;

fn main() {
    #[cfg(not(feature = "truck_integration"))]
    {
        println!("This example requires the 'truck_integration' feature.");
        println!(
            "Run with: cargo run --example truck_integration_example --features truck_integration"
        );
        return;
    }

    #[cfg(feature = "truck_integration")]
    {
        println!("OpenSubdiv to Truck Integration Example");
        println!("======================================\n");

        // Create a simple cube topology
        let face_vertex_counts = vec![4, 4, 4, 4, 4, 4]; // 6 faces with 4 vertices each
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

        // Create topology descriptor
        let descriptor =
            TopologyDescriptor::new(face_vertex_counts.clone(), face_vertex_indices.clone())
                .expect("Failed to create topology descriptor");

        // Create topology refiner
        let refiner_options = TopologyRefinerOptions::default();
        let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
            .expect("Failed to create topology refiner");

        // Refine uniformly to level 2 for smooth surfaces
        refiner.refine_uniform(2);
        println!("Refined mesh to level 2");

        // Get refined vertex positions
        let level = refiner.level(2).expect("Failed to get refinement level");
        let num_vertices = level.vertex_count();

        // For this example, we'll use placeholder positions
        // In a real application, you would use PrimvarRefiner to refine the positions
        let mut refined_positions = vec![[0.0f32; 3]; num_vertices];

        // Copy base level positions
        for (i, pos) in vertex_positions.iter().enumerate() {
            if i < refined_positions.len() {
                refined_positions[i] = *pos;
            }
        }

        // Create patch table with B-spline end caps
        let patch_options = PatchTableOptions::new().end_cap_type(EndCapType::BSplineBasis);

        let patch_table =
            PatchTable::new(&refiner, Some(patch_options)).expect("Failed to create patch table");

        println!("\nPatch Table created:");
        println!("  {} patch arrays", patch_table.patch_arrays_len());
        println!("  {} total patches", patch_table.patches_len());

        // Convert patches to truck surfaces using the trait-based API
        use truck_geometry::prelude::*;
        use truck_modeling::*;

        // Convert all patches to B-spline surfaces
        let surfaces_result: Result<Vec<BSplineSurface<Point3>>, _> = patch_table
            .with_control_points(&refined_positions)
            .try_into();

        match surfaces_result {
            Ok(surfaces) => {
                println!(
                    "\nSuccessfully converted {} patches to B-spline surfaces",
                    surfaces.len()
                );

                // Print information about each surface
                for (i, surface) in surfaces.iter().enumerate() {
                    let (u_range, v_range) = surface.parameter_range();
                    println!(
                        "  Surface {}: u=[{:.2}, {:.2}], v=[{:.2}, {:.2}]",
                        i, u_range.start, u_range.end, v_range.start, v_range.end
                    );
                }

                // Example: Convert a single patch
                if patch_table.patches_len() > 0 {
                    let single_surface: Result<BSplineSurface<Point3>, _> =
                        patch_table.patch(0, &refined_positions).try_into();

                    if let Ok(surface) = single_surface {
                        println!("\nSuccessfully converted single patch to B-spline surface");
                        let point = surface.subs(0.5, 0.5);
                        println!(
                            "  Point at (0.5, 0.5): ({:.3}, {:.3}, {:.3})",
                            point.x, point.y, point.z
                        );
                    }
                }
            }
            Err(e) => {
                println!("Error converting patches: {}", e);
            }
        }

        // Try to create a shell (collection of connected surfaces)
        let shell_result: Result<Shell<Point3, Curve, Surface>, _> = patch_table
            .with_control_points(&refined_positions)
            .try_into();

        match shell_result {
            Ok(shell) => {
                println!(
                    "\nSuccessfully created a shell with {} faces",
                    shell.face_iter().count()
                );

                // The shell can now be used for:
                // - Boolean operations
                // - STEP export
                // - Further CAD operations

                // Example: Export to STEP format (requires additional truck
                // modules) let step_string =
                // truck_stepio::out::shell_to_string(&shell);
            }
            Err(e) => {
                println!("Error creating shell: {}", e);
            }
        }

        println!("\nTruck integration example completed!");
    }
}
