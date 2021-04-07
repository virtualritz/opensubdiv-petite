use opensubdiv::far;

fn main() {
    let vertices = [
        -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
        0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];
    let num_vertices = vertices.len() / 3;

    let verts_per_face = [4, 4, 4, 4, 4, 4];

    let vert_indices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let creases = [0, 1, 1, 3, 3, 2, 2, 0];

    let crease_weights = [10., 10., 10., 10.];

    // populate a descriptor with our raw data
    let mut refiner = far::TopologyRefiner::new(
        far::TopologyDescriptor::new(
            num_vertices as _,
            &verts_per_face,
            &vert_indices,
        )
        .creases(&creases, &crease_weights)
        .clone(),
        far::topology_refiner::Options::new()
            .scheme(far::Scheme::CatmullClark)
            .boundary_interpolation(far::BoundaryInterpolation::EdgeOnly)
            .clone(),
    )
    .expect("Could not create TopologyRefiner");

    let max_level = 2;
    // uniformly refine up to 'max level' of 2
    refiner.refine_uniform(
        far::topology_refiner::UniformRefinementOptions::default()
            .refinement_level(max_level)
            .clone(),
    );

    // interpolate vertex primvar data
    let primvar_refiner = far::PrimvarRefiner::new(&refiner);

    let mut refined_verts = Vec::with_capacity(max_level as _);

    refined_verts.push(vertices.to_vec());

    for level in 1..=max_level {
        refined_verts.push(
            primvar_refiner
                .interpolate(
                    level,
                    3, // Each element is a 3-tuple
                    refined_verts[(level - 1) as usize].as_slice(),
                )
                .unwrap(),
        );
    }

    // output an OBJ of the highest level
    let last_level = refiner.level(max_level).unwrap();

    println!("o subdivision_cube");

    // print vertex positions
    for v in refined_verts.last().unwrap().chunks(3) {
        println!("v {} {} {}", v[0], v[1], v[2]);
    }

    // for f in 0..nfaces {
    //     let face_vert_indices =
    // last_level.face_vertices(Index(f)).unwrap();
    for face_vert_indices in last_level.face_vertices_iter() {
        // all refined cat-clark faces should be quads
        assert!(face_vert_indices.len() == 4);
        print!("f ");
        for fv in face_vert_indices {
            print!("{} ", fv + 1);
        }
        print!("\n");
    }
}
