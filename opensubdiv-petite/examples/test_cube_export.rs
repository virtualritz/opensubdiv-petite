use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions, PatchTable,
};

#[cfg(feature = "truck")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opensubdiv_petite::truck_integration::PatchTableExt;
    use truck_stepio::out::*;
    
    // Create a simple cube mesh
    let vertex_positions = vec![
        [-0.5, -0.5, -0.5], // 0
        [ 0.5, -0.5, -0.5], // 1
        [-0.5,  0.5, -0.5], // 2
        [ 0.5,  0.5, -0.5], // 3
        [-0.5,  0.5,  0.5], // 4
        [ 0.5,  0.5,  0.5], // 5
        [-0.5, -0.5,  0.5], // 6
        [ 0.5, -0.5,  0.5], // 7
    ];
    
    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 1, 3, 2,  // back
        2, 3, 5, 4,  // top
        4, 5, 7, 6,  // front
        6, 7, 1, 0,  // bottom
        0, 2, 4, 6,  // left
        1, 7, 5, 3,  // right
    ];
    
    let descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    );

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;
    
    let mut adaptive_options = AdaptiveRefinementOptions::default();
    adaptive_options.isolation_level = 3;
    refiner.refine_adaptive(adaptive_options, &[]);
    
    // Try with Gregory basis end cap
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::GregoryBasis)
        .use_inf_sharp_patch(false);
    
    let patch_table = PatchTable::new(&refiner, Some(patch_options))?;
    
    // Build vertex buffer
    let primvar_refiner = PrimvarRefiner::new(&refiner);
    let mut all_vertices = Vec::with_capacity(refiner.vertex_total_count());
    all_vertices.extend_from_slice(&vertex_positions);
    
    for level in 1..refiner.refinement_levels() {
        let src_data: Vec<f32> = all_vertices[(all_vertices.len() - refiner.level(level - 1).unwrap().vertex_count())..]
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
    
    // Add local points
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
    
    // Convert to shell - test both methods
    println!("Testing regular conversion...");
    match patch_table.to_truck_shell(&all_vertices) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string = CompleteStepDisplay::new(
                StepModel::from(&compressed),
                Default::default(),
            ).to_string();
            std::fs::write("cube_regular.step", step_string)?;
            println!("Wrote cube_regular.step");
        }
        Err(e) => println!("Regular conversion failed: {:?}", e),
    }
    
    println!("Testing gap-filling conversion...");
    match patch_table.to_truck_shell_with_gap_filling(&all_vertices) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string = CompleteStepDisplay::new(
                StepModel::from(&compressed),
                Default::default(),
            ).to_string();
            std::fs::write("cube_gap_filled.step", step_string)?;
            println!("Wrote cube_gap_filled.step");
        }
        Err(e) => println!("Gap-filling conversion failed: {:?}", e),
    }
    
    Ok(())
}

#[cfg(not(feature = "truck"))]
fn main() {
    println!("This example requires the 'truck' feature. Run with:");
    println!("  cargo run --example test_cube_export --features truck");
}