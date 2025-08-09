//! Geometry generator binary
//! 
//! Generates subdivision surface geometry from a chamfered tetrahedron base mesh.

use opensubdiv_petite::{far, tri_mesh_buffers};
use std::fs::File;
use std::io::Write;

// Uniformly refine up to 'max level' of 3.
static MAX_LEVEL: usize = 3;

fn main() {
    let mesh_data = generate_chamfered_tetrahedron();
    
    // Write the mesh data to stdout or a file
    println!("Generated mesh with:");
    println!("  {} vertices", mesh_data.vertices.len() / 3);
    println!("  {} triangles", mesh_data.indices.len() / 3);
    
    // Write to OBJ file
    if let Err(e) = write_obj_file("chamfered_tetrahedron.obj", &mesh_data) {
        eprintln!("Error writing OBJ file: {}", e);
    } else {
        println!("Wrote mesh to chamfered_tetrahedron.obj");
    }
}

/// Mesh data structure
struct MeshData {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
}

/// Generate a subdivided chamfered tetrahedron mesh
fn generate_chamfered_tetrahedron() -> MeshData {
    // Topology for a chamfered tetrahedron.
    // cT – in Conway notation.
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

    assert!(0 == vertices.len() % 3);

    let face_arities = [3u32, 3, 3, 3, 6, 6, 6, 6, 6, 6];

    let face_vertices = [
        4u32, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 9, 8, 2, 4, 6, 0, 12, 11, 3, 7, 9, 1, 15,
        14, 3, 11, 10, 0, 6, 5, 1, 10, 12, 2, 8, 7, 3, 14, 13, 1, 5, 4, 2, 13, 15,
    ];

    let creases = [
        4u32, 5, 5, 6, 7, 8, 8, 9, 10, 11, 11, 12, 13, 14, 14, 15, 0, 9, 2, 4, 4, 6, 0, 12, 3, 7,
        7, 9, 1, 15, 3, 11, 0, 6, 1, 10, 10, 12, 2, 8, 3, 14, 1, 5, 2, 13, 13, 15,
    ];

    let crease_weights = [2.0; 24];

    // Create a refiner (a subdivider) from a topology descriptor.
    let mut refiner = far::TopologyRefiner::new(
        // Populate the descriptor with our raw data.
        far::TopologyDescriptor::new(vertices.len() / 3, &face_arities, &face_vertices)
            .creases(&creases, &crease_weights)
            // NOTE: Removing .left_handed(true) to fix the inside-out normals issue
            // The original code had left-handed winding which was causing inverted normals
            .clone(),
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: far::BoundaryInterpolation::EdgeOnly,
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    // Refine (subdivide) the topology uniformly MAX_LEVEL times.
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: MAX_LEVEL,
        ..Default::default()
    });

    // Interpolate vertex primvar data.
    let primvar_refiner = far::PrimvarRefiner::new(&refiner);

    let mut refined_vertices = vertices.to_vec();

    // Subdivide MAX_LEVEL times.
    // Note how the refined_vertices from the previous refinement step become
    // the base for the next.
    for level in 1..=MAX_LEVEL {
        refined_vertices = primvar_refiner
            .interpolate(
                level,
                3, // Each element is a 3-tuple.
                &refined_vertices,
            )
            .unwrap();
    }

    // Convert the subdivision mesh (all quads by now) into disconnected triangles.
    let (indices, points, normals) = tri_mesh_buffers::to_triangle_mesh_buffers(
        &refined_vertices,
        refiner.level(MAX_LEVEL).unwrap().face_vertices_iter(),
    );

    MeshData {
        vertices: points,
        normals,
        indices,
    }
}

/// Write mesh data to an OBJ file
fn write_obj_file(filename: &str, mesh: &MeshData) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    
    writeln!(file, "# Generated chamfered tetrahedron")?;
    writeln!(file, "# {} vertices, {} triangles", mesh.vertices.len() / 3, mesh.indices.len() / 3)?;
    writeln!(file)?;
    
    // Write vertices
    for i in (0..mesh.vertices.len()).step_by(3) {
        writeln!(file, "v {} {} {}", mesh.vertices[i], mesh.vertices[i + 1], mesh.vertices[i + 2])?;
    }
    writeln!(file)?;
    
    // Write normals
    for i in (0..mesh.normals.len()).step_by(3) {
        writeln!(file, "vn {} {} {}", mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2])?;
    }
    writeln!(file)?;
    
    // Write faces (OBJ uses 1-based indexing)
    for i in (0..mesh.indices.len()).step_by(3) {
        let v0 = mesh.indices[i] as usize + 1;
        let v1 = mesh.indices[i + 1] as usize + 1;
        let v2 = mesh.indices[i + 2] as usize + 1;
        writeln!(file, "f {}//{} {}//{} {}//{}", v0, v0, v1, v1, v2, v2)?;
    }
    
    Ok(())
}