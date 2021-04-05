use opensubdiv::far;

#[derive(Copy, Clone)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

struct VertexSlice<'a> {
    slice: &'a [Vertex],
}

struct VertexSliceMut<'a> {
    slice: &'a mut [Vertex],
}

impl<'a> far::PrimvarBufferSrc for VertexSlice<'a> {
    const LEN_ELEMENTS: u32 = 3;

    fn as_f32(&self) -> &[f32] {
        unsafe {
            std::slice::from_raw_parts(
                self.slice.as_ptr() as *const f32,
                self.slice.len() * 3,
            )
        }
    }
}

impl<'a> far::PrimvarBufferDst for VertexSliceMut<'a> {
    const LEN_ELEMENTS: u32 = 3;

    fn as_f32_mut(&mut self) -> &mut [f32] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.slice.as_mut_ptr() as *mut f32,
                self.slice.len() * 3,
            )
        }
    }
}

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

    // populate a descriptor with our raw data
    let mut refiner = far::TopologyDescriptor::new(
        num_vertices as _,
        &verts_per_face,
        &vert_indices,
    )
    .into_refiner(
        far::topology_refiner::Options::new()
            .with_scheme(far::Scheme::CatmullClark)
            .with_boundary_interpolation(far::BoundaryInterpolation::EdgeOnly)
            .finalize(),
    )
    .expect("Could not create TopologyRefiner");

    let max_level = 2;
    // uniformly refine up to 'max level' of 2
    refiner.refine_uniform(
        far::topology_refiner::UniformRefinementOptions::default()
            .refinement_level(max_level)
            .finalize(),
    );

    // initialize coarse mesh positions
    let vbuffer = vertices
        .chunks(3)
        .map(|v| Vertex {
            x: v[0],
            y: v[1],
            z: v[2],
        })
        .collect::<Vec<_>>();

    // interpolate vertex primvar data
    let primvar_refiner = far::PrimvarRefiner::new(&refiner);

    let mut refined_verts = Vec::with_capacity(max_level as _);

    refined_verts.push(vbuffer);
    for level in 1..=max_level {
        let mut dst_vec = vec![
            Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0
            };
            refiner.level(level).unwrap().len_vertices() as _
        ];

        let src = unsafe {
            VertexSlice {
                slice: refined_verts
                    .get_unchecked(level as usize - 1)
                    .as_slice(),
            }
        };

        let mut dst = VertexSliceMut {
            slice: dst_vec.as_mut_slice(),
        };

        primvar_refiner.interpolate(level, &src, &mut dst);

        refined_verts.push(dst_vec);
    }

    // output an OBJ of the highest level
    let last_level = refiner.level(max_level).unwrap();

    // print vertex positions
    for v in refined_verts.last().unwrap().iter() {
        println!("v {} {} {}", v.x, v.y, v.z);
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
