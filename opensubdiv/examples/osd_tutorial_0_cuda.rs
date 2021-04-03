#![cfg(not(target_os = "macos"))]

use opensubdiv::{far, osd, sdc};

fn main() {
    let vertices = [
        -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
        0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];
    let num_vertices = 8;
    let num_faces = 6;

    let verts_per_face = [4, 4, 4, 4, 4, 4];

    let vert_indices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    // populate a descriptor with our raw data
    let descriptor = far::TopologyDescriptor::new(
        num_vertices,
        num_faces,
        &verts_per_face,
        &vert_indices,
    );

    // instantiate a TopologyRefiner from the descriptor
    let mut refiner = far::topology_refiner_factory::create(
        descriptor,
        far::topology_refiner_factory::Options::new(
            sdc::Scheme::CatmullClark,
            sdc::OptionsBuilder::new()
                .vtx_boundary_interpolation(
                    sdc::VtxBoundaryInterpolation::EdgeOnly,
                )
                .build(),
        ),
    )
    .expect("Could not create TopologyRefiner");

    refiner.refine_uniform(far::uniform_options().refinement_level(2).build());

    let stencil_table = far::stencil_table_factory::create(
        &refiner,
        far::stencil_table_factory::options()
            .generate_offsets(true)
            .generate_intermediate_levels(false)
            .build(),
    );

    let n_coarse_verts = refiner.level(0).unwrap().num_vertices();
    let n_refined_verts = stencil_table.num_stencils();

    // set up a buffer for primvar data
    let mut src_buffer = osd::CudaVertexBuffer::new(3, n_coarse_verts);
    let mut dst_buffer = osd::CudaVertexBuffer::new(3, n_refined_verts);

    let cuda_stencil_table = osd::CudaStencilTable::new(&stencil_table);

    // execution phase (every frame)
    {
        // pack the control vertices at the start of the buffer
        src_buffer.update_data(&vertices, 0, n_coarse_verts);

        let src_desc = osd::BufferDescriptor::new(0, 3, 3);
        let dst_desc = osd::BufferDescriptor::new(0, 3, 3);

        // launch the computation
        osd::cuda_evaluator::eval_stencils(
            &src_buffer,
            src_desc,
            &mut dst_buffer,
            dst_desc,
            &cuda_stencil_table,
        )
        .expect("eval_stencils failed");

        // print the result as a MEL command to draw vertices as points
        let refined_verts = dst_buffer.bind_cuda_buffer();
        println!("particle");
        for v in refined_verts.chunks(3) {
            let x = v[0];
            let y = v[1];
            let z = v[2];
            println!("-p {} {} {}", x, y, z);
        }
        println!("-c 1;");
    }
}
