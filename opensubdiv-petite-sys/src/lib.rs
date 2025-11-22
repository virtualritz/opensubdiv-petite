#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::useless_transmute)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(unsafe_op_in_unsafe_fn)]
//! You probably want the high level wrapper – [`opensubdiv-petite`](https://crates.io/crates/opensubdiv-petite).

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Manual type definitions for blocked types
pub type OpenSubdiv_v3_7_0_Far_PatchTable_StencilTablePtr = *mut std::ffi::c_void;

pub mod far;
pub use far::*;

pub mod sdc;

pub mod osd;

pub mod vtr;

pub mod bfr;

// Disabled test for now.
/*
#[test]
fn _it_works() {
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

    let scheme_type = sdc::Scheme::CatmullClark;

    let mut options = sdc::Options::default();
    options.set_vtx_bound_interp(
        sdc::options::VtxBoundaryInterpolation::EdgeOnly as u32,
    );
    let asu = unsafe { std::mem::transmute::<sdc::Options, u32>(options) };
    println!("sdc options bits: {}", asu);

    let descriptor = far::TopologyDescriptor::new(
        num_vertices,
        num_faces,
        verts_per_face.as_ptr(),
        vert_indices.as_ptr(),
    );

    let refiner = unsafe {
        TopologyRefinerFactory_TopologyDescriptor_Create(
            &descriptor as *const TopologyDescriptor,
            far::topology_refiner_factory::Options::new(scheme_type, options),
        )
    };

    // uniformly refine the topology up to max level
    unsafe {
        let opt = UniformOptions::new(2, false, false);
        let asu = std::mem::transmute::<UniformOptions, u32>(opt);
        println!("refine options bits: {}", asu);
        TopologyRefiner_RefineUniform(
            refiner,
            UniformOptions::new(2, false, false),
        );
    }

    let mut stencil_opts = far::stencil_table_factory::Options::default();
    stencil_opts.fields.set_generate_offsets(true);
    stencil_opts.fields.set_generate_intermediate_levels(false);
    let asu = unsafe {
        std::mem::transmute::<OptionsFields, u32>(stencil_opts.fields)
    };
    println!("stencil options bits: {}", asu);

    let stencil_table =
        unsafe { StencilTableFactory_Create(refiner, stencil_opts) };

    // get coarse verts and refined verts
    let n_coarse_verts = unsafe {
        let level = TopologyRefiner_GetLevel(refiner, 0);
        TopologyLevel_GetNumVertices(level)
    };

    let n_refined_verts = unsafe { StencilTable_GetNumStencils(stencil_table) };

    unsafe {
        TopologyRefiner_destroy(refiner);
    }

    // set up a buffer for primvar data
    let vbuffer = unsafe {
        osd::CpuVertexBuffer_Create(
            3,
            n_coarse_verts + n_refined_verts,
            std::ptr::null(),
        )
    };

    // execution phase (every frame)
    {
        // pack the control vertex data at the start of the vertex buffer and
        // update every time the control data changes
        unsafe {
            osd::CpuVertexBuffer_UpdateData(
                vbuffer,
                vertices.as_ptr(),
                0,
                n_coarse_verts,
                std::ptr::null(),
            );
        }

        let src_desc = osd::BufferDescriptor::new(0, 3, 3);
        let dst_desc = osd::BufferDescriptor::new(n_coarse_verts * 3, 3, 3);

        // launch the computation
        let res = unsafe {
            osd::cpu_evaluator::CpuEvaluator_EvalStencils(
                vbuffer,
                src_desc,
                vbuffer,
                dst_desc,
                stencil_table,
            )
        };
        println!("Res: {}", res);

        let refined_verts = unsafe {
            std::slice::from_raw_parts(
                osd::CpuVertexBuffer_BindCpuBuffer(vbuffer)
                    .offset(3 * n_coarse_verts as isize),
                n_refined_verts as usize * 3,
            )
        };

        println!("particle ");
        for i in 0..n_refined_verts as usize {
            println!(
                "-p {:.6} {:.6} {:.6}",
                refined_verts[i * 3],
                refined_verts[i * 3 + 1],
                refined_verts[i * 3 + 2]
            );
        }
        println!("-c 1;");
    }
}
*/
