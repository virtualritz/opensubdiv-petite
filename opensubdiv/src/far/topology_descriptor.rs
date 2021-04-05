use opensubdiv_sys as sys;
use std::{convert::TryInto, marker::PhantomData};

/// A container holding references to raw topology data.
///
/// `TopologyDescriptor` contains references to raw topology data as flat index
/// buffers.  This is used to construct a [`TopologyRefiner`].
#[derive(Clone, Copy, Debug)]
pub struct TopologyDescriptor<'a> {
    pub(crate) descriptor: sys::OpenSubdiv_v3_4_4_Far_TopologyDescriptor,
    // _marker needs to be invariant in 'a.
    // See "Making a struct outlive a parameter given to a method of
    // that struct": https://stackoverflow.com/questions/62374326/
    _marker: PhantomData<*mut &'a ()>,
}

impl<'a> TopologyDescriptor<'a> {
    /// Describes a mesh topology including creases, corners, holes and
    /// handedness.  This can be used as a builder to create a new
    /// [TopologyRefiner] by calling
    /// [`into_refiner()`](TopologyDescriptor::into_refiner()).
    ///
    /// ## Parameters
    /// * `len_vertices` - The number of vertices in the mesh.
    /// * `len_verts_per_face` - A slice containing the number of vertices for
    ///   each face in the mesh. The length of this is the number of faces in
    ///   the mesh.
    /// * `vert_indices_per_face` - A flat list of the vertex indices for each
    ///   face in the mesh.
    #[inline]
    pub fn new(
        len_vertices: u32,
        len_verts_per_face: &'a [u32],
        vert_indices_per_face: &'a [u32],
    ) -> TopologyDescriptor<'a> {
        let mut descriptor =
            unsafe { sys::OpenSubdiv_v3_4_4_Far_TopologyDescriptor::new() };

        descriptor.numVertices = len_vertices.try_into().unwrap();
        descriptor.numFaces = len_verts_per_face.len().try_into().unwrap();
        descriptor.numVertsPerFace = len_verts_per_face.as_ptr() as _;
        descriptor.vertIndicesPerFace = vert_indices_per_face.as_ptr() as _;

        TopologyDescriptor {
            descriptor,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn creases(
        &mut self,
        creases: &'a [u32],
        weights: &'a [f32],
    ) -> &mut Self {
        debug_assert!(0 == creases.len() % 2);
        debug_assert!(weights.len() == creases.len() / 2);

        self.descriptor.numCreases = weights.len().try_into().unwrap();
        self.descriptor.creaseVertexIndexPairs = creases.as_ptr() as _;
        self.descriptor.creaseWeights = weights.as_ptr();
        self
    }

    #[inline]
    pub fn corners(
        &mut self,
        corners: &'a [u32],
        weights: &'a [f32],
    ) -> &mut Self {
        debug_assert!(weights.len() == corners.len());

        self.descriptor.numCorners = weights.len().try_into().unwrap();
        self.descriptor.cornerVertexIndices = corners.as_ptr() as _;
        self.descriptor.cornerWeights = weights.as_ptr();
        self
    }

    #[inline]
    pub fn holes(&mut self, holes: &'a [u32]) -> &mut Self {
        self.descriptor.numHoles = holes.len().try_into().unwrap();
        self.descriptor.holeIndices = holes.as_ptr() as _;
        self
    }

    #[inline]
    pub fn left_handed(&mut self, left_handed: bool) -> &mut Self {
        self.descriptor.isLeftHanded = left_handed;
        self
    }
}
