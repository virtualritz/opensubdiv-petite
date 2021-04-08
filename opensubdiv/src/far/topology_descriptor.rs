//! A container holding references to raw topology data.
//!
//! ## Example
//! ```
//! # use opensubdiv::far::TopologyDescriptor;
//! // The positions. This is commonly used later, with a PrimvarRefiner.
//! let vertices = [1, 1, 1, 1, -1, -1, -1, 1, -1, -1 - 1, 1];
//!
//! // Describe the basic topology of our tetrahedron.
//! let mut tetrahedron = TopologyDescriptor::new(
//!     (vertices.len() / 3) as _,
//!     &[3; 4],
//!     &[2, 1, 0, 3, 2, 0, 1, 3, 0, 2, 3, 1],
//! );
//!
//! // Make all edges creased with sharpness 8.0.
//! tetrahedron.creases(&[0, 2, 0, 3, 1, 3, 0, 1, 2, 3, 1, 2], &[8.0; 6]);
//! ```
//!
//! ## Semi-Sharp Creases
//! Just as some types of parametric surfaces support additional shaping
//! controls to affect creasing along the boundaries between surface elements,
//! *OpenSubdiv* provides additional sharpness values associated with
//! edges and vertices to achieve similar results over arbitrary topology.
//!
//! Setting sharpness values to a maximum value (10 in this case – a number
//! chosen for historical reasons) effectively modifies the subdivision rules so
//! that the boundaries between the piecewise smooth surfaces are infinitely
//! sharp or discontinuous.
//!
//! But since real world surfaces never really have infinitely sharp edges,
//! especially when viewed sufficiently close, it is often preferable to set the
//! sharpness lower than this value, making the crease "semi-sharp".  A constant
//! weight value assigned to a sequence of edges connected edges therefore
//! enables the creation of features akin to fillets and blends without adding
//! extra rows of vertices (though that technique still has its merits):
//!
//! Sharpness values range from 0–10, with a value of 0 (or less) having no
//! effect on the surface and a value of 10 (or more) making the feature
//! completely sharp.
//!
//! It should be noted that infinitely sharp creases are really tangent
//! discontinuities in the surface, implying that the geometric normals are also
//! discontinuous there.  Therefore, displacing along the normal will likely
//! tear apart the surface along the crease.  If you really want to displace a
//! surface at a crease, it may be better to make the crease semi-sharp.
use opensubdiv_sys as sys;
use std::{convert::TryInto, marker::PhantomData};

/// A `TopologyDescriptor` holds references to raw topology data as flat index
/// buffers.
///
/// This is used to construct a
/// [`TopologyRefiner`](crate::far::TopologyRefiner).
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
    /// handedness.  This is fed into a
    /// [`TopologyRefiner`](crate::far::TopologyRefiner).
    ///
    /// ## Parameters
    /// * `vertices_len` - The number of vertices in the mesh.
    /// * `vertices_per_face - A slice containing the number of vertices for
    ///   each face in the mesh. The length of this is the number of faces in
    ///   the mesh.
    /// * `vertex_indices_per_face` - A flat list of the vertex indices for each
    ///   face in the mesh.
    #[inline]
    pub fn new(
        vertices_len: usize,
        vertices_per_face: &'a [u32],
        vertex_indices_per_face: &'a [u32],
    ) -> TopologyDescriptor<'a> {
        let mut descriptor =
            unsafe { sys::OpenSubdiv_v3_4_4_Far_TopologyDescriptor::new() };

        descriptor.numVertices = vertices_len.try_into().unwrap();
        descriptor.numFaces = vertices_per_face.len().try_into().unwrap();
        descriptor.numVertsPerFace = vertices_per_face.as_ptr() as _;
        descriptor.vertIndicesPerFace = vertex_indices_per_face.as_ptr() as _;

        TopologyDescriptor {
            descriptor,
            _marker: PhantomData,
        }
    }

    /// Add creases as vertex index pairs with corresponding sharpness.
    #[inline]
    pub fn creases(
        &mut self,
        creases: &'a [u32],
        sharpness: &'a [f32],
    ) -> &mut Self {
        debug_assert!(0 == creases.len() % 2);
        debug_assert!(creases.len() / 2 <= sharpness.len());

        self.descriptor.numCreases = sharpness.len().try_into().unwrap();
        self.descriptor.creaseVertexIndexPairs = creases.as_ptr() as _;
        self.descriptor.creaseWeights = sharpness.as_ptr();
        self
    }

    /// Add corners as vertex indices with corresponding sharpness.
    #[inline]
    pub fn corners(
        &mut self,
        corners: &'a [u32],
        sharpness: &'a [f32],
    ) -> &mut Self {
        debug_assert!(corners.len() <= sharpness.len());

        self.descriptor.numCorners = sharpness.len().try_into().unwrap();
        self.descriptor.cornerVertexIndices = corners.as_ptr() as _;
        self.descriptor.cornerWeights = sharpness.as_ptr();
        self
    }

    /// Add holes as face indices.
    #[inline]
    pub fn holes(&mut self, holes: &'a [u32]) -> &mut Self {
        self.descriptor.numHoles = holes.len().try_into().unwrap();
        self.descriptor.holeIndices = holes.as_ptr() as _;
        self
    }

    /// Set if the topology describes faces with left handed (counter-clockwise)
    /// winding.
    #[inline]
    pub fn left_handed(&mut self, left_handed: bool) -> &mut Self {
        self.descriptor.isLeftHanded = left_handed;
        self
    }
}
