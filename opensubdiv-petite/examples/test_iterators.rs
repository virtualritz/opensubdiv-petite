//! Test example for iterator functionality.

use opensubdiv_petite::far::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple cube mesh.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, // face 0
        2, 3, 5, 4, // face 1  
        4, 5, 7, 6, // face 2
        6, 7, 1, 0, // face 3
        1, 7, 5, 3, // face 4
        6, 0, 2, 4, // face 5
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let mut refiner = TopologyRefiner::new(descriptor, TopologyRefinerOptions::default())?;
    
    // Refine uniformly to level 2.
    let mut options = UniformRefinementOptions::default();
    options.refinement_level = 2;
    refiner.refine_uniform(options);
    
    // Get refined level.
    let level_2 = refiner.level(2).unwrap();
    
    println!("Testing sequential iterator:");
    let seq_count = level_2
        .face_vertices_iter()
        .map(|face| face.len())
        .sum::<usize>();
    println!("  Sequential iterator counted {} vertices", seq_count);
    
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        
        println!("\nTesting parallel iterator:");
        let par_count: usize = level_2
            .face_vertices_par_iter()
            .map(|face| face.len())
            .sum();
        println!("  Parallel iterator counted {} vertices", par_count);
        
        assert_eq!(seq_count, par_count, "Counts should match!");
        println!("\n✓ Both iterators produce the same result!");
    }
    
    #[cfg(not(feature = "rayon"))]
    println!("\n(Parallel iterator not available - rayon feature not enabled)");
    
    Ok(())
}