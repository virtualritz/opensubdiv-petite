use anyhow::Result;
use opensubdiv_petite::{far, osd};

fn main() -> Result<()> {
    let vertices = [
        -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
        -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];
    let num_vertices = vertices.len() / 3;

    let verts_per_face = [4, 4, 4, 4, 4, 4];

    let vert_indices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    // instantiate a TopologyRefiner from a descriptor
    let mut refiner = far::TopologyRefiner::new(
        far::TopologyDescriptor::new(num_vertices as _, &verts_per_face, &vert_indices)?,
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: Some(far::BoundaryInterpolation::EdgeOnly),
            ..Default::default()
        },
    )?;

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
    )?;

    let n_coarse_verts = refiner
        .level(0)
        .ok_or(anyhow::anyhow!("No level 0"))?
        .vertex_count();
    let n_refined_verts = stencil_table.len();

    // set up a buffer for primvar data
    let mut src_buffer = osd::CpuVertexBuffer::new(3, n_coarse_verts)?;
    let mut dst_buffer = osd::CpuVertexBuffer::new(3, n_refined_verts)?;

    // execution phase (every frame)
    {
        // pack the control vertices at the start of the buffer
        src_buffer.update_data(&vertices, 0, n_coarse_verts);

        let src_desc = osd::BufferDescriptor::new(0, 3, 3)?;
        let dst_desc = osd::BufferDescriptor::new(0, 3, 3)?;

        // launch the computation
        osd::cpu_evaluator::evaluate_stencils(
            &src_buffer,
            src_desc,
            &mut dst_buffer,
            dst_desc,
            &stencil_table,
        )?;

        // print the result as a MEL command to draw vertices as points
        let refined_verts = dst_buffer.bind_cpu_buffer()?;
        println!("particle");
        for v in refined_verts.chunks(3) {
            println!("-p {} {} {}", v[0], v[1], v[2]);
        }
        println!("-c 1;");
    }

    Ok(())
}
