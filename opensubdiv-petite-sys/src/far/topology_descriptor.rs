#[repr(C)]
pub struct FVarChannel {
    num_values: i32,
    value_indices: *const i32,
}

#[repr(C)]
pub struct TopologyDescriptor {
    num_vertices: i32,
    num_faces: i32,

    num_verts_per_face: *const i32,
    vert_indices_per_face: *const i32,

    num_creases: i32,
    crease_vertex_index_pairs: *const i32,
    crease_weights: *const f32,

    num_corners: i32,
    corner_vertex_indices: *const i32,
    corner_weights: *const f32,

    num_holes: i32,
    hole_indices: *const i32,

    is_left_handed: bool,

    num_fvar_channels: i32,
    fvar_channels: *const FVarChannel,
}

impl Default for TopologyDescriptor {
    fn default() -> TopologyDescriptor {
        TopologyDescriptor {
            num_vertices: 0,
            num_faces: 0,
            num_verts_per_face: std::ptr::null(),
            vert_indices_per_face: std::ptr::null(),
            num_creases: 0,
            crease_vertex_index_pairs: std::ptr::null(),
            crease_weights: std::ptr::null(),
            num_corners: 0,
            corner_vertex_indices: std::ptr::null(),
            corner_weights: std::ptr::null(),
            num_holes: 0,
            hole_indices: std::ptr::null(),
            is_left_handed: false,
            num_fvar_channels: 0,
            fvar_channels: std::ptr::null(),
        }
    }
}

impl TopologyDescriptor {
    pub fn new(
        num_vertices: i32,
        num_faces: i32,
        num_verts_per_face: *const i32,
        vert_indices_per_face: *const i32,
    ) -> TopologyDescriptor {
        TopologyDescriptor {
            num_vertices,
            num_faces,
            num_verts_per_face,
            vert_indices_per_face,
            ..Default::default()
        }
    }
}
