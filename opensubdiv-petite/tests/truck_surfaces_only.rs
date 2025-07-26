//! Test for exporting B-spline surfaces directly (without face topology)

#[cfg(feature = "truck")]
mod tests {
    use opensubdiv_petite::far::{
        PatchTable, TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
        UniformRefinementOptions, PatchTableOptions, EndCapType, PrimvarRefiner,
    };
    use opensubdiv_petite::Index;
    use std::path::PathBuf;
    
    fn test_output_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        path.push(filename);
        path
    }
    
    /// Build complete vertex buffer including all refinement levels
    fn build_vertex_buffer(
        refiner: &TopologyRefiner,
        base_vertices: &[[f32; 3]],
    ) -> Vec<[f32; 3]> {
        println!("Building vertex buffer:");
        
        let num_levels = refiner.refinement_levels();
        println!("  Number of refinement levels: {}", num_levels);
        
        // Calculate total vertices needed
        let mut total_vertices = base_vertices.len();
        for level in 1..=num_levels {
            total_vertices += refiner.level(level).unwrap().vertex_count();
        }
        println!("  Total vertices across all levels: {}", total_vertices);
        
        // Allocate buffer for all vertices
        let mut all_vertices = Vec::with_capacity(total_vertices);
        all_vertices.extend_from_slice(base_vertices);
        println!("  Level 0: {} vertices", base_vertices.len());
        
        // Refine vertices level by level
        let mut level_start = 0;
        let mut prev_level_count = base_vertices.len();
        
        for level in 1..=num_levels {
            let level_obj = refiner.level(level).unwrap();
            let level_count = level_obj.vertex_count();
            
            println!("  Level {}: {} vertices (interpolating from {} vertices at level {})", 
                     level, level_count, prev_level_count, level - 1);
            
            // Allocate vertices for this level
            let mut level_vertices = vec![[0.0f32; 3]; level_count];
            
            // Build flat source data from PREVIOUS level only
            let src_data: Vec<f32> = all_vertices[level_start..level_start + prev_level_count]
                .iter()
                .flat_map(|v| v.iter().copied())
                .collect();
            
            
            // Create a primvar refiner and interpolate
            let primvar_refiner = PrimvarRefiner::new(&refiner);
            if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
                // Convert back to vertex array
                for (i, vertex) in level_vertices.iter_mut().enumerate() {
                    vertex[0] = refined[i * 3];
                    vertex[1] = refined[i * 3 + 1];
                    vertex[2] = refined[i * 3 + 2];
                }
            }
            
            println!("    Interpolated {} vertices", level_vertices.len());
            all_vertices.extend_from_slice(&level_vertices);
            
            level_start += prev_level_count;
            prev_level_count = level_count;
        }
        
        println!("  Final vertex buffer size: {}", all_vertices.len());
        all_vertices
    }
    
    #[test] 
    fn test_export_surfaces_only() {
        use opensubdiv_petite::truck_integration::PatchTableExt;
        use truck_geometry::prelude::ParametricSurface;
        
        // Simple plane - 3x3 grid of vertices
        let vertex_positions = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [2.0, 1.0, 0.0], [3.0, 1.0, 0.0],
            [0.0, 2.0, 0.0], [1.0, 2.0, 0.0], [2.0, 2.0, 0.0], [3.0, 2.0, 0.0],
            [0.0, 3.0, 0.0], [1.0, 3.0, 0.0], [2.0, 3.0, 0.0], [3.0, 3.0, 0.0],
        ];
        
        // Define plane faces (3x3 quads)
        let face_vertices = vec![
            vec![0, 1, 5, 4],   vec![1, 2, 6, 5],   vec![2, 3, 7, 6],
            vec![4, 5, 9, 8],   vec![5, 6, 10, 9],  vec![6, 7, 11, 10],
            vec![8, 9, 13, 12], vec![9, 10, 14, 13], vec![10, 11, 15, 14],
        ];
        
        // Flatten face data
        let num_face_vertices = face_vertices.iter().map(|f| f.len() as u32).collect::<Vec<_>>();
        let face_indices = face_vertices.iter()
            .flatten()
            .map(|&i| Index::from(i as u32))
            .collect::<Vec<_>>();
        
        // Create topology descriptor
        let face_indices_u32: Vec<u32> = face_indices.into_iter().map(|idx| idx.into()).collect();
        let descriptor = TopologyDescriptor::new(
            vertex_positions.len(),
            &face_indices_u32,
            &num_face_vertices,
        );
        
        // Create topology refiner
        let refiner_options = TopologyRefinerOptions::default();
        let mut refiner = TopologyRefiner::new(descriptor, refiner_options)
            .expect("Failed to create topology refiner");
        
        // Refine uniformly
        refiner.refine_uniform(UniformRefinementOptions { 
            refinement_level: 1,
            order_vertices_from_faces_first: false,
            full_topology_in_last_level: true,
        });
        
        // Build complete vertex buffer
        let all_vertices = build_vertex_buffer(&refiner, &vertex_positions);
        
        // Create patch table
        let patch_options = PatchTableOptions::new()
            .end_cap_type(EndCapType::BSplineBasis);
        
        let patch_table = PatchTable::new(&refiner, Some(patch_options))
            .expect("Failed to create patch table");
        
        println!("Number of patches: {}", patch_table.patches_len());
        
        // Convert patches to B-spline surfaces
        let surfaces = patch_table.to_truck_surfaces(&all_vertices)
            .expect("Failed to convert to truck surfaces");
        
        println!("Created {} B-spline surfaces", surfaces.len());
        
        // For debugging, print the first surface's control points
        if let Some(first_surface) = surfaces.first() {
            println!("\nFirst surface control points:");
            let control_points = first_surface.control_points();
            for (i, row) in control_points.iter().enumerate() {
                println!("  Row {}: ", i);
                for (j, pt) in row.iter().enumerate() {
                    println!("    [{},{}] = ({:.3}, {:.3}, {:.3})", i, j, pt.x, pt.y, pt.z);
                }
            }
            
            // Sample the surface at a few points
            println!("\nSurface evaluation at parameter points:");
            for u in [0.0, 0.5, 1.0] {
                for v in [0.0, 0.5, 1.0] {
                    let pt = first_surface.subs(u, v);
                    println!("  S({:.1}, {:.1}) = ({:.3}, {:.3}, {:.3})", u, v, pt.x, pt.y, pt.z);
                }
            }
        }
        
        // For now, just ensure we got surfaces
        assert!(!surfaces.is_empty(), "No surfaces were created");
        println!("\nTest passed: {} surfaces created successfully", surfaces.len());
    }
}