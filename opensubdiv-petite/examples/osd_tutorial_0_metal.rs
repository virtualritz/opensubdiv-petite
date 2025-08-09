#![cfg(target_os = "macos")]

use opensubdiv_petite::{far, osd};

fn main() {
    println!("Metal example for OpenSubdiv");
    println!("Note: This example requires Metal framework setup which is beyond the scope of this basic example.");
    println!("You would need to:");
    println!("1. Initialize a Metal device");
    println!("2. Create command queue and command buffers");
    println!("3. Pass those to the Metal evaluator");

    let vertices = [
        -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
        -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];
    let num_vertices = vertices.len() / 3;

    let verts_per_face = [4, 4, 4, 4, 4, 4];

    let vert_indices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    // populate a descriptor with our raw data
    let descriptor =
        far::TopologyDescriptor::new(num_vertices as _, &verts_per_face, &vert_indices);

    // instantiate a TopologyRefiner from the descriptor
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
        refinement_level: 2,
        ..Default::default()
    });

    let stencil_table = far::StencilTable::new(
        &refiner,
        far::StencilTableOptions {
            generate_offsets: true,
            generate_intermediate_levels: false,
            ..Default::default()
        },
    );

    let n_coarse_verts = refiner.level(0).unwrap().vertex_count();
    let n_refined_verts = stencil_table.len();

    println!("Created topology with {} coarse vertices", n_coarse_verts);
    println!("Refined to {} vertices", n_refined_verts);

    // Metal implementation would go here
    // You would need to:
    // 1. Get Metal device: let device = ...;
    // 2. Create vertex buffers:
    //    let src_buffer = osd::MetalVertexBuffer::new(3, n_coarse_verts, device);
    //    let dst_buffer = osd::MetalVertexBuffer::new(3, n_refined_verts, device);
    // 3. Create stencil table:
    //    let metal_stencil_table = osd::MetalStencilTable::new(&stencil_table, device);
    // 4. Update data and evaluate:
    //    src_buffer.update_data(&vertices, 0, n_coarse_verts, command_buffer);
    //    osd::metal_evaluator::evaluate_stencils(
    //        &src_buffer, src_desc, &mut dst_buffer, dst_desc,
    //        &metal_stencil_table, command_buffer, compute_encoder
    //    ).expect("eval_stencils failed");
}
