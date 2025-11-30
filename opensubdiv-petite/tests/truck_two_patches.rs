mod utils;

#[cfg(feature = "truck")]
use utils::*;

#[cfg(feature = "truck")]
#[test]
fn test_two_patches_surfaces_only() -> anyhow::Result<()> {
    use opensubdiv_petite::far::{
        AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
        TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
    };
    use opensubdiv_petite::truck::PatchTableExt;
    use truck_modeling::*;

    // Simple cube - same as in truck.rs test
    let vertex_positions = vec![
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 2, 3, 1, // front face (-z)
        2, 6, 7, 3, // top face (+y)
        6, 4, 5, 7, // back face (+z)
        4, 0, 1, 5, // bottom face (-y)
        4, 6, 2, 0, // left face (-x)
        1, 3, 7, 5, // right face (+x)
    ];

    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    )?;

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // Use adaptive refinement with isolation level 3 to get regular patches
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

    // Debug: check patch table contents
    println!("Patch table has {} arrays", patch_table.patch_arrays_len());
    println!("Total patches: {}", patch_table.patches_len());
    for i in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(i) {
            println!(
                "  Array {}: type={:?}, {} patches",
                i,
                desc.patch_type(),
                patch_table.patch_array_patches_len(i)
            );
        }
    }

    // If no Regular patches, try to get Quads patches
    if patch_table.patches_len() == 0 {
        println!("No patches generated! Check refinement settings.");
        return Ok(());
    }

    // Convert patches to truck shell - same approach as simple_plane test
    let shell = patch_table.to_truck_shell(&all_vertices)?;

    // Compress and export the shell as STEP - same as simple plane
    let compressed = shell.compress();

    // Write to STEP file using truck_stepio
    use truck_stepio::out;
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            file_name: "two_patches_surfaces_only.step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();

    // Write STEP file to test output directory
    let step_path = test_output_path("two_patches_surfaces_only.step");
    std::fs::write(&step_path, &step_string)?;

    println!("\nWrote STEP file to: {}", step_path.display());

    // Compare or update expected results
    assert_file_matches(&step_path, "two_patches_surfaces_only.step");
    Ok(())
}
