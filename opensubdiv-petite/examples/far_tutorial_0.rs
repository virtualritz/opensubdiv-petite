use opensubdiv_petite::far;

fn main() {
    // Geomtry for a cube control polyhedron.
    let vertices = [
        -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
        -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];

    let num_vertices = vertices.len() / 3;

    let verts_per_face = [4; 6];

    let vert_indices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let creases = [0, 1, 1, 3, 3, 2, 2, 0];

    let crease_weights = [10., 10., 10., 10.];

    // Create a refiner from a descriptor.
    let mut refiner = far::TopologyRefiner::new(
        // Populate the descriptor with our raw data.
        far::TopologyDescriptor::new(num_vertices as _, &verts_per_face, &vert_indices)
            .creases(&creases, &crease_weights)
            .clone(),
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: Some(far::BoundaryInterpolation::EdgeOnly),
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    // Uniformly refine up to 'max level' of 2.
    let max_level = 2;

    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: max_level,
        ..Default::default()
    });

    // Interpolate vertex primvar data.
    let primvar_refiner = far::PrimvarRefiner::new(&refiner);

    // Create a vector holding all the subdivison levels.
    let mut refined_verts = Vec::with_capacity(max_level as _);

    refined_verts.push(vertices.to_vec());

    for level in 1..=max_level {
        println!("{}", level);
        refined_verts.push(
            primvar_refiner
                .interpolate(
                    level,
                    3, // Each element is a 3-tuple.
                    refined_verts[(level - 1) as usize].as_slice(),
                )
                .unwrap(),
        );
    }

    // Output an OBJ of the highest level.
    let last_level = refiner.level(max_level).unwrap();

    println!("o subdivision_cube");

    // Print vertex positions.
    for v in refined_verts.last().unwrap().chunks(3) {
        println!("v {} {} {}", v[0], v[1], v[2]);
    }

    for face_vert_indices in last_level.face_vertices_iter() {
        // All refined cat-clark faces should be quads.
        assert!(4 == face_vert_indices.len());
        print!("f");
        for fv in face_vert_indices {
            print!(" {}", fv.0 + 1);
        }
        println!();
    }
}
