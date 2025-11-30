//! Demonstrates STEP export of a cube with infinitely sharp creases.
//!
//! OpenSubdiv considers sharpness >= 10.0 as "infinite". Unlike semi-sharp
//! creases (which decay by 1.0 per subdivision level), infinite creases
//! act as topological boundaries and don't require deep isolation.
//!
//! Key insight: `isolation_level = 1` is sufficient for infinite creases
//! because they don't decay - the edge stays perfectly sharp forever.

use anyhow::Result;
use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
};
use opensubdiv_petite::truck::PatchTableExt;
use truck_stepio::out::*;

/// Export a cube with all 12 edges creased at the given sharpness.
fn export_infinite_crease_cube(sharpness: f32, filename: &str) -> Result<()> {
    println!("\n=== Exporting cube with sharpness {sharpness} to {filename} ===");

    // Unit cube vertices.
    let vertices = vec![
        [-0.5, -0.5, -0.5], // 0: back-bottom-left
        [0.5, -0.5, -0.5],  // 1: back-bottom-right
        [-0.5, 0.5, -0.5],  // 2: back-top-left
        [0.5, 0.5, -0.5],   // 3: back-top-right
        [-0.5, 0.5, 0.5],   // 4: front-top-left
        [0.5, 0.5, 0.5],    // 5: front-top-right
        [-0.5, -0.5, 0.5],  // 6: front-bottom-left
        [0.5, -0.5, 0.5],   // 7: front-bottom-right
    ];

    // Six quad faces.
    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 1, 3, 2, // back
        2, 3, 5, 4, // top
        4, 5, 7, 6, // front
        6, 7, 1, 0, // bottom
        0, 2, 4, 6, // left
        1, 7, 5, 3, // right
    ];

    // Crease only three edges sharing vertex 0 (same as creased_cube_export.rs).
    let crease_indices: Vec<u32> = vec![
        0, 1, // edge 0-1
        0, 2, // edge 0-2
        0, 6, // edge 0-6
    ];
    let crease_sharpness = vec![sharpness; 3];

    let mut descriptor =
        TopologyDescriptor::new(vertices.len(), &face_vertex_counts, &face_vertex_indices)?;
    descriptor.creases(&crease_indices, &crease_sharpness);

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // AIDEV-NOTE: Isolation level for infinite vs semi-sharp creases.
    // - Semi-sharp (sharpness < 10.0): needs ceil(sharpness) + 1 levels because
    //   sharpness decays by 1.0 per subdivision level.
    // - Infinite (sharpness >= 10.0): only needs 1 level because infinite creases
    //   act as topological boundaries and don't decay.
    let is_infinite = sharpness >= 10.0;
    let isolation_level = if is_infinite {
        1 // One refinement converts all faces to quads; crease is a boundary.
    } else {
        (sharpness.ceil() as usize + 1).max(1)
    };

    println!(
        "  Sharpness: {sharpness}, is_infinite: {is_infinite}, isolation_level: {isolation_level}"
    );

    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = isolation_level;
    refiner.refine_adaptive(adaptive_options, &[]);

    // Create patch table with inf_sharp_patch enabled for infinite creases.
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::GregoryBasis)
        .use_inf_sharp_patch(is_infinite);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

    println!(
        "  Patch arrays: {}, total patches: {}",
        patch_table.patch_array_count(),
        (0..patch_table.patch_array_count())
            .map(|i| patch_table.patch_array_patch_count(i))
            .sum::<usize>()
    );
    for array_idx in 0..patch_table.patch_array_count() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            println!(
                "    Array {}: type {:?}, patches {}",
                array_idx,
                desc.patch_type(),
                patch_table.patch_array_patch_count(array_idx)
            );
        }
    }

    // Build vertex buffer with refined positions.
    let primvar_refiner = PrimvarRefiner::new(&refiner)?;
    let mut all_vertices = Vec::with_capacity(refiner.vertex_count_all_levels());
    all_vertices.extend_from_slice(&vertices);

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

    // Add local points for Gregory patches.
    let num_local_points = patch_table.local_point_count();
    if num_local_points > 0 {
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

    println!("  Total vertices: {}", all_vertices.len());

    // Export to STEP via truck.
    match patch_table.to_truck_shell(&all_vertices) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write(filename, step_string)?;
            println!("  Wrote {filename}");
        }
        Err(e) => {
            eprintln!("  Export failed: {:?}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("Infinite Crease Cube Export Example");
    println!("====================================");
    println!();
    println!("OpenSubdiv treats sharpness >= 10.0 as 'infinite'.");
    println!("Infinite creases act as topological boundaries and only need");
    println!("isolation_level = 1 (not ceil(sharpness) + 1).");

    // Test with different infinite sharpness values.
    // All should produce identical sharp-edged cubes.
    export_infinite_crease_cube(10.0, "infinite_cube_10.step")?;
    export_infinite_crease_cube(11.0, "infinite_cube_11.step")?;
    export_infinite_crease_cube(100.0, "infinite_cube_100.step")?;

    println!();
    println!("All three STEP files should show identical sharp-edged cubes.");
    println!("Open them in a CAD viewer to verify the edges are perfectly sharp.");

    Ok(())
}
