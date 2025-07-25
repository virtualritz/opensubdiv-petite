//! Example demonstrating patch table creation and usage

use opensubdiv_petite::far::{
    EndCapType, PatchTable, PatchTableOptions, PatchType, TopologyDescriptor, TopologyRefiner,
    TopologyRefinerOptions,
};

fn main() {
    println!("OpenSubdiv Patch Table Example");
    println!("==============================\n");

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
        -1.0, -1.0, -1.0, // 0
        1.0, -1.0, -1.0,  // 1
        -1.0, -1.0, 1.0,  // 2
        1.0, -1.0, 1.0,   // 3
        -1.0, 1.0, 1.0,   // 4
        1.0, 1.0, 1.0,    // 5
        -1.0, 1.0, -1.0,  // 6
        1.0, 1.0, -1.0,   // 7
    ];

    // Create topology descriptor
    let descriptor = TopologyDescriptor::new(
        face_vertex_counts.clone(),
        face_vertex_indices.clone(),
    )
    .expect("Failed to create topology descriptor");

    println!("Created topology descriptor for a cube:");
    println!("  {} vertices", vertex_positions.len() / 3);
    println!("  {} faces", face_vertex_counts.len());

    // Create topology refiner
    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
        .expect("Failed to create topology refiner");

    println!("\nCreated topology refiner");

    // Refine uniformly to level 2
    refiner.refine_uniform(2);
    println!("Refined uniformly to level 2");
    
    // Create patch table with B-spline end caps
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::BSplineBasis);
    
    println!("\nCreating patch table with B-spline end caps...");
    let patch_table = PatchTable::new(&refiner, Some(patch_options))
        .expect("Failed to create patch table");

    // Report patch table statistics
    println!("\nPatch Table Statistics:");
    println!("  Number of patch arrays: {}", patch_table.patch_arrays_len());
    println!("  Total number of patches: {}", patch_table.patches_len());
    println!("  Number of control vertices: {}", patch_table.control_vertices_len());
    println!("  Maximum valence: {}", patch_table.max_valence());

    // Iterate through patch arrays
    println!("\nPatch Arrays:");
    for i in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(i) {
            let num_patches = patch_table.patch_array_patches_len(i);
            println!(
                "  Array {}: {} patches of type {:?} ({} control vertices each)",
                i,
                num_patches,
                desc.patch_type(),
                desc.control_vertices_len()
            );

            // Check if these are regular B-spline patches
            if desc.is_regular() {
                println!("    -> These are regular bi-cubic B-spline patches");
            }
        }
    }

    // Access patch parameters for the first few patches
    println!("\nFirst few patch parameters:");
    for array_idx in 0..patch_table.patch_arrays_len().min(2) {
        for patch_idx in 0..patch_table.patch_array_patches_len(array_idx).min(3) {
            if let Some(param) = patch_table.patch_param(array_idx, patch_idx) {
                let (u, v) = param.uv();
                println!(
                    "  Patch [{}, {}]: UV=({:.3}, {:.3}), depth={}, boundary={}, transition={}",
                    array_idx,
                    patch_idx,
                    u,
                    v,
                    param.depth(),
                    param.boundary(),
                    param.transition()
                );
            }
        }
    }

    // Access control vertex indices
    if let Some(cv_table) = patch_table.control_vertices_table() {
        println!("\nControl vertex table has {} entries", cv_table.len());
        println!("First 16 control vertex indices: {:?}", &cv_table[..16.min(cv_table.len())]);
    }

    println!("\nPatch table example completed successfully!");
}