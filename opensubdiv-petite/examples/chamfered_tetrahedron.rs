//! Generate a subdivided chamfered tetrahedron and print it as a Wavefront OBJ.
//!
//! The base mesh is a chamfered tetrahedron (`cT` in Conway notation): four
//! triangles and six hexagons, with semi-sharp creases along the chamfer
//! edges. It is refined with Catmull--Clark, so every refined face is a quad
//! and is written out as a quad `f` line -- OBJ supports n-gons directly, so no
//! triangulation is needed.
//!
//! Run with:
//! ```text
//! cargo run --example chamfered_tetrahedron > chamfered_tetrahedron.obj
//! ```
use opensubdiv_petite::far;

// Uniformly refine up to this level.
const MAX_LEVEL: usize = 3;

fn main() {
    // Vertex positions of the chamfered tetrahedron (16 vertices).
    let vertices = [
        0.57735025f32,
        0.57735025,
        0.57735025,
        0.57735025,
        -0.57735025,
        -0.57735025,
        -0.57735025,
        0.57735025,
        -0.57735025,
        -0.57735025,
        -0.57735025,
        0.57735025,
        -0.2566001,
        0.5132003,
        -0.5132003,
        0.5132003,
        -0.2566001,
        -0.5132003,
        0.5132003,
        0.5132003,
        0.2566001,
        -0.5132003,
        -0.2566001,
        0.5132003,
        -0.5132003,
        0.5132003,
        -0.2566001,
        0.2566001,
        0.5132003,
        0.5132003,
        0.5132003,
        -0.5132003,
        -0.2566001,
        -0.2566001,
        -0.5132003,
        0.5132003,
        0.5132003,
        0.2566001,
        0.5132003,
        -0.5132003,
        0.2566001,
        -0.5132003,
        -0.5132003,
        -0.5132003,
        0.2566001,
        0.2566001,
        -0.5132003,
        -0.5132003,
    ];

    let num_vertices = vertices.len() / 3;

    // Four triangles followed by six hexagons.
    let verts_per_face = [3u32, 3, 3, 3, 6, 6, 6, 6, 6, 6];

    let vert_indices = [
        4u32, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 9, 8, 2, 4, 6, 0, 12, 11, 3, 7, 9, 1, 15,
        14, 3, 11, 10, 0, 6, 5, 1, 10, 12, 2, 8, 7, 3, 14, 13, 1, 5, 4, 2, 13, 15,
    ];

    // Semi-sharp creases along the chamfer edges, as vertex-index pairs.
    let creases = [
        4u32, 5, 5, 6, 7, 8, 8, 9, 10, 11, 11, 12, 13, 14, 14, 15, 0, 9, 2, 4, 4, 6, 0, 12, 3, 7,
        7, 9, 1, 15, 3, 11, 0, 6, 1, 10, 10, 12, 2, 8, 3, 14, 1, 5, 2, 13, 13, 15,
    ];

    let crease_weights = [2.0f32; 24];

    // Build the descriptor, then chain in the creases (the consuming builder
    // returns the descriptor; assign the result).
    let mut descriptor =
        far::TopologyDescriptor::new(num_vertices as _, &verts_per_face, &vert_indices)
            .expect("Could not create TopologyDescriptor");
    descriptor = descriptor
        .creases(&creases, &crease_weights)
        .expect("Failed to add creases");

    let mut refiner = far::TopologyRefiner::new(
        descriptor,
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: Some(far::BoundaryInterpolation::EdgeOnly),
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: MAX_LEVEL,
        ..Default::default()
    });

    // Interpolate vertex positions through each refinement level.
    let primvar_refiner =
        far::PrimvarRefiner::new(&refiner).expect("Could not create PrimvarRefiner");

    let mut refined_vertices = vertices.to_vec();
    for level in 1..=MAX_LEVEL {
        refined_vertices = primvar_refiner
            .interpolate(level, 3, &refined_vertices)
            .expect("refinement_level exceeds the refiner's max level");
    }

    // Emit the highest level as OBJ.
    let last_level = refiner.level(MAX_LEVEL).unwrap();

    println!("o chamfered_tetrahedron");

    for v in refined_vertices.chunks(3) {
        println!("v {} {} {}", v[0], v[1], v[2]);
    }

    for face_vert_indices in last_level.face_vertices_iter() {
        print!("f");
        for fv in face_vert_indices {
            // OBJ vertex indices are 1-based.
            print!(" {}", fv.0 + 1);
        }
        println!();
    }
}
