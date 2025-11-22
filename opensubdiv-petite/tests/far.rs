//! Tests for the far module.

use anyhow::Result;
use opensubdiv_petite::far::*;
use opensubdiv_petite::Index;

#[test]
fn topology_descriptor_creation() -> Result<()> {
    // Test basic cube topology.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;

    // Descriptor is moved, not cloned
    let _moved = descriptor;
    Ok(())
}

#[test]
fn topology_descriptor_with_creases() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let crease_vertices = [0, 1, 1, 3, 3, 2, 2, 0];
    let crease_weights = [10.0, 10.0, 10.0, 10.0];

    let mut descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    descriptor.creases(&crease_vertices, &crease_weights);

    // Test that descriptor can be used (moved)
    let _moved = descriptor;
    Ok(())
}

#[test]
fn topology_refiner_options_default() {
    let options = TopologyRefinerOptions::default();
    assert!(matches!(options.scheme, Scheme::CatmullClark));
    assert!(options.boundary_interpolation.is_none());
    assert!(matches!(
        options.face_varying_linear_interpolation,
        Some(FaceVaryingLinearInterpolation::All)
    ));
    assert!(matches!(options.creasing_method, CreasingMethod::Uniform));
    assert!(matches!(
        options.triangle_subdivision,
        TriangleSubdivision::CatmullClark
    ));
}

#[test]
fn topology_refiner_options_custom() {
    let options = TopologyRefinerOptions {
        scheme: Scheme::Loop,
        boundary_interpolation: Some(BoundaryInterpolation::EdgeOnly),
        face_varying_linear_interpolation: None,
        creasing_method: CreasingMethod::Chaikin,
        triangle_subdivision: TriangleSubdivision::Smooth,
    };

    assert!(matches!(options.scheme, Scheme::Loop));
    assert!(matches!(
        options.boundary_interpolation,
        Some(BoundaryInterpolation::EdgeOnly)
    ));
    assert!(options.face_varying_linear_interpolation.is_none());
    assert!(matches!(options.creasing_method, CreasingMethod::Chaikin));
    assert!(matches!(
        options.triangle_subdivision,
        TriangleSubdivision::Smooth
    ));
}

#[test]
fn topology_refiner_creation() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let refiner = TopologyRefiner::new(descriptor, options)?;

    // Check initial state.
    assert_eq!(refiner.refinement_levels(), 1);
    assert_eq!(refiner.vertex_count_all_levels(), 8);
    assert_eq!(refiner.face_count_all_levels(), 6);
    assert_eq!(refiner.edge_count_all_levels(), 12);
    // A newly created refiner hasn't had uniform refinement applied yet
    // The is_uniform() state is implementation-defined until refinement is applied
    assert!(!refiner.has_holes());
    Ok(())
}

#[test]
fn topology_refiner_uniform_refinement() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let mut refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    // Refine uniformly to level 2.
    refiner.refine_uniform(UniformRefinementOptions {
        refinement_level: 2,
        ..Default::default()
    });

    assert_eq!(refiner.refinement_levels(), 3); // Base + 2 refined levels.
    assert!(refiner.is_uniform());
    assert_eq!(refiner.max_level(), 2);

    // Check we can access all levels.
    assert!(refiner.level(0).is_some());
    assert!(refiner.level(1).is_some());
    assert!(refiner.level(2).is_some());
    assert!(refiner.level(3).is_none());
    Ok(())
}

#[test]
fn topology_level_access() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    let level0 = refiner.level(0).expect("Level 0 should exist");

    // Test count methods.
    assert_eq!(level0.vertex_count(), 8);
    assert_eq!(level0.face_count(), 6);
    assert_eq!(level0.edge_count(), 12);
    assert_eq!(level0.face_vertex_count(), 24); // 6 faces * 4 vertices each.
    Ok(())
}

#[test]
fn topology_level_face_vertices() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    let level0 = refiner.level(0).expect("Level 0 should exist");

    // Check face vertices.
    let face0_verts = level0
        .face_vertices(Index::from(0u32))
        .expect("Face 0 should have vertices");
    assert_eq!(face0_verts.len(), 4);
    assert_eq!(face0_verts[0].0, 0);
    assert_eq!(face0_verts[1].0, 1);
    assert_eq!(face0_verts[2].0, 3);
    assert_eq!(face0_verts[3].0, 2);

    // Test invalid face index.
    assert!(level0.face_vertices(Index::from(99u32)).is_none());
    Ok(())
}

#[test]
fn topology_level_relationships() -> Result<()> {
    let vertices_per_face = [3, 3]; // Two triangles sharing an edge.
    let face_vertices = [0, 1, 2, 1, 3, 2];

    let descriptor = TopologyDescriptor::new(4, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    let level0 = refiner.level(0).expect("Level 0 should exist");

    // Find the shared edge between vertices 1 and 2.
    let edge = level0.find_edge(Index::from(1u32), Index::from(2u32));
    assert!(edge.is_some());

    // Test edge vertices.
    let edge_verts = level0
        .edge_vertices(edge.unwrap())
        .expect("Edge should have vertices");
    assert_eq!(edge_verts.len(), 2);

    // Test edge faces.
    let edge_faces = level0
        .edge_faces(edge.unwrap())
        .expect("Edge should have adjacent faces");
    assert_eq!(edge_faces.len(), 2); // Shared by two triangles.
    Ok(())
}

#[test]
fn primvar_refiner() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    // 3D positions for cube vertices.
    let positions = [
        -0.5, -0.5, 0.5, // 0
        0.5, -0.5, 0.5, // 1
        -0.5, 0.5, 0.5, // 2
        0.5, 0.5, 0.5, // 3
        -0.5, 0.5, -0.5, // 4
        0.5, 0.5, -0.5, // 5
        -0.5, -0.5, -0.5, // 6
        0.5, -0.5, -0.5, // 7
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let mut refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    refiner.refine_uniform(UniformRefinementOptions {
        refinement_level: 1,
        ..Default::default()
    });

    let primvar_refiner = PrimvarRefiner::new(&refiner)?;

    // Interpolate positions to level 1.
    let refined_positions = primvar_refiner
        .interpolate(1, 3, &positions)
        .expect("Failed to interpolate primvars");

    // Level 1 should have more vertices than level 0.
    let level1_vertex_count = refiner.level(1).unwrap().vertex_count();
    assert!(level1_vertex_count > 8);
    assert_eq!(refined_positions.len(), level1_vertex_count * 3);
    Ok(())
}

#[test]
fn stencil_table() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let mut refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    refiner.refine_uniform(UniformRefinementOptions {
        refinement_level: 2,
        ..Default::default()
    });

    let stencil_options = StencilTableOptions {
        generate_offsets: true,
        generate_intermediate_levels: false,
        ..Default::default()
    };

    let stencil_table = StencilTable::new(&refiner, stencil_options)?;

    // Debug output
    let num_levels = refiner.refinement_levels();
    let level0_verts = refiner.level(0).unwrap().vertex_count();
    let level1_verts = refiner.level(1).unwrap().vertex_count();
    let level2_verts = refiner.level(2).unwrap().vertex_count();
    let total_verts = refiner.vertex_count_all_levels();

    println!("Debug StencilTable test:");
    println!("  Refinement levels: {num_levels}");
    println!("  Level 0 vertices: {level0_verts}");
    println!("  Level 1 vertices: {level1_verts}");
    println!("  Level 2 vertices: {level2_verts}");
    println!("  Total vertices: {total_verts}");
    println!("  StencilTable options:");
    println!("    generate_offsets: {}", stencil_options.generate_offsets);
    println!(
        "    generate_intermediate_levels: {}",
        stencil_options.generate_intermediate_levels
    );
    println!(
        "    generate_control_vertices: {}",
        stencil_options.generate_control_vertices
    );

    // Stencil table should have stencils for refined vertices.
    // The exact number depends on the refinement level and topology.
    let stencil_count = stencil_table.len();
    println!("  Stencil count: {}", stencil_count);

    // Try with default options too
    let default_stencil_table = StencilTable::new(&refiner, StencilTableOptions::default())?;
    let default_count = default_stencil_table.len();
    println!("  Default stencil count: {}", default_count);

    assert!(
        stencil_count > 0,
        "StencilTable should contain stencils after refinement"
    );
    assert_eq!(
        stencil_count, 98,
        "StencilTable should have 98 stencils for level 2 vertices only"
    );
    Ok(())
}

#[test]
fn uniform_refinement_options_default() {
    let options = UniformRefinementOptions::default();
    assert_eq!(options.refinement_level, 4);
    assert!(options.order_vertices_from_faces_first);
    assert!(options.full_topology_in_last_level);
}

#[test]
fn adaptive_refinement_options_default() {
    let options = AdaptiveRefinementOptions::default();
    assert_eq!(options.isolation_level, 4);
    assert_eq!(options.secondary_level, 15);
    assert!(!options.single_crease_patch);
    assert!(!options.infintely_sharp_patch);
    assert!(!options.consider_face_varying_channels);
    assert!(!options.order_vertices_from_faces_first);
}

#[test]
fn face_vertices_iter() -> Result<()> {
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

    // Refine uniformly to level 1.
    let mut options = UniformRefinementOptions::default();
    options.refinement_level = 1;
    refiner.refine_uniform(options);

    // Get the base level.
    let level_0 = refiner.level(0).unwrap();

    // Test the iterator.
    let mut face_count = 0;
    let mut total_vertices = 0;

    for face_verts in level_0.face_vertices_iter() {
        assert_eq!(face_verts.len(), 4, "Cube faces should have 4 vertices");
        face_count += 1;
        total_vertices += face_verts.len();
    }

    assert_eq!(face_count, 6, "Cube should have 6 faces");
    assert_eq!(total_vertices, 24, "Total vertices in faces should be 24");

    // Verify iterator produces same results as direct access.
    let mut iter_results = Vec::new();
    for face_verts in level_0.face_vertices_iter() {
        iter_results.push(face_verts.to_vec());
    }

    for (i, face_verts) in iter_results.iter().enumerate() {
        let direct_verts = level_0.face_vertices(i.into()).unwrap();
        assert_eq!(
            face_verts, direct_verts,
            "Iterator should match direct access"
        );
    }

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]
fn face_vertices_par_iter() -> Result<()> {
    use rayon::prelude::*;

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

    // Refine uniformly to level 2 for more faces.
    let mut options = UniformRefinementOptions::default();
    options.refinement_level = 2;
    refiner.refine_uniform(options);

    // Get refined level.
    let level_2 = refiner.level(2).unwrap();

    // Test parallel iterator.
    let par_face_count: usize = level_2
        .face_vertices_par_iter()
        .map(|face_verts| {
            assert_eq!(face_verts.len(), 4, "All faces should have 4 vertices");
            1
        })
        .sum();

    // Should have 6 * 4^2 = 96 faces at level 2.
    assert_eq!(par_face_count, 96, "Level 2 should have 96 faces");

    // Verify parallel iterator produces same results as sequential.
    let mut seq_results: Vec<Vec<Index>> =
        level_2.face_vertices_iter().map(|v| v.to_vec()).collect();

    let mut par_results: Vec<Vec<Index>> = level_2
        .face_vertices_par_iter()
        .map(|v| v.to_vec())
        .collect();

    // Sort both since parallel order might differ.
    seq_results.sort();
    par_results.sort();

    assert_eq!(
        seq_results, par_results,
        "Parallel and sequential iterators should produce same faces"
    );

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]
fn face_vertices_par_iter_performance() -> Result<()> {
    use rayon::prelude::*;
    use std::time::Instant;

    // Create a larger mesh for performance testing.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let mut refiner = TopologyRefiner::new(descriptor, TopologyRefinerOptions::default())?;

    // Refine to level 4 for many faces.
    let mut options = UniformRefinementOptions::default();
    options.refinement_level = 4;
    refiner.refine_uniform(options);

    let level_4 = refiner.level(4).unwrap();

    // Time sequential iteration.
    let start = Instant::now();
    let seq_sum: usize = level_4.face_vertices_iter().map(|face| face.len()).sum();
    let seq_time = start.elapsed();

    // Time parallel iteration.
    let start = Instant::now();
    let par_sum: usize = level_4
        .face_vertices_par_iter()
        .map(|face| face.len())
        .sum();
    let par_time = start.elapsed();

    assert_eq!(
        seq_sum, par_sum,
        "Sequential and parallel should compute same sum"
    );

    // Print timing info (won't fail test, just informational).
    println!("Sequential time: {:?}", seq_time);
    println!("Parallel time: {:?}", par_time);
    println!(
        "Speedup: {:.2}x",
        seq_time.as_secs_f64() / par_time.as_secs_f64()
    );

    Ok(())
}

#[test]
fn deprecated_method_wrappers() -> Result<()> {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = TopologyRefinerOptions::default();

    let refiner =
        TopologyRefiner::new(descriptor, options).expect("Failed to create TopologyRefiner");

    // Test deprecated methods still work.
    #[allow(deprecated)]
    {
        assert_eq!(
            refiner.vertices_total_len(),
            refiner.vertex_count_all_levels()
        );
        assert_eq!(refiner.edges_total_len(), refiner.edge_count_all_levels());
        assert_eq!(refiner.faces_total_len(), refiner.face_count_all_levels());
        assert_eq!(
            refiner.face_vertices_total_len(),
            refiner.face_vertex_count_all_levels()
        );
    }

    let level0 = refiner.level(0).expect("Level 0 should exist");

    #[allow(deprecated)]
    {
        assert_eq!(level0.vertices_len(), level0.vertex_count());
        assert_eq!(level0.faces_len(), level0.face_count());
        assert_eq!(level0.edges_len(), level0.edge_count());
        assert_eq!(level0.face_vertices_len(), level0.face_vertex_count());
    }

    Ok(())
}
