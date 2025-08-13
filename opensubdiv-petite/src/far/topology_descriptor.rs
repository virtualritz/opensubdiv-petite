//! A container holding references to raw topology data.
//!
//! ## Example
//! ```
//! # use opensubdiv_petite::far::TopologyDescriptor;
//! // The positions as a flat buffer. This is commonly used later, with a PrimvarRefiner.
//! let vertices = [1, 1, 1, 1, -1, -1, -1, 1, -1, -1, -1, 1];
//!
//! // Describe the basic topology of our tetrahedron.
//! let mut tetrahedron = TopologyDescriptor::new(
//!     vertices.len() / 3,
//!     // Four triangles.
//!     &[3; 4],
//!     // Vertex indices for each triangle.
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
use opensubdiv_petite_sys as sys;
use std::marker::PhantomData;

/// A `TopologyDescriptor` holds references to raw topology data as flat index
/// buffers.
///
/// This is used to construct a
/// [`TopologyRefiner`](crate::far::TopologyRefiner).
///
/// See the [module level documentation](crate::far::topology_descriptor) for
/// an example.
#[derive(Debug)]
pub struct TopologyDescriptor<'a> {
    pub(crate) descriptor: sys::OpenSubdiv_v3_6_1_Far_TopologyDescriptor,
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
    /// # Arguments
    ///
    /// * `vertices_len` - The number of vertices in the mesh.
    /// * `vertices_per_face` - A slice containing the number of vertices for
    ///   each face in the mesh. The length of this is the number of faces in
    ///   the mesh.
    /// * `vertex_indices_per_face` - A flat list of the vertex indices for each
    ///   face in the mesh.
    #[inline]
    pub fn new(
        vertices_len: usize,
        vertices_per_face: &'a [u32],
        vertex_indices_per_face: &'a [u32],
    ) -> crate::Result<TopologyDescriptor<'a>> {
        let mut descriptor = unsafe { sys::OpenSubdiv_v3_6_1_Far_TopologyDescriptor::new() };

        #[cfg(feature = "topology_validation")]
        {
            if vertex_indices_per_face.len() != vertices_per_face.iter().sum::<u32>() as _ {
                return Err(crate::Error::InvalidTopology(
                    "The number of vertex indices is not equal to the sum of face arities."
                        .to_string(),
                ));
            }
            for (i, &vertex_index) in vertex_indices_per_face.iter().enumerate() {
                if vertices_len <= (vertex_index as usize) {
                    return Err(crate::Error::InvalidTopology(format!(
                        "Vertex index[{}] = {} is out of range (should be < {}).",
                        i, vertex_index, vertices_len
                    )));
                }
            }
        }

        descriptor.numVertices = vertices_len.min(i32::MAX as usize) as i32;
        descriptor.numFaces = vertices_per_face.len().min(i32::MAX as usize) as i32;
        descriptor.numVertsPerFace = vertices_per_face.as_ptr() as _;
        descriptor.vertIndicesPerFace = vertex_indices_per_face.as_ptr() as _;

        Ok(TopologyDescriptor {
            descriptor,
            _marker: PhantomData,
        })
    }

    /// Add creases as vertex index pairs with corresponding sharpness.
    #[inline]
    pub fn creases(&mut self, creases: &'a [u32], sharpness: &'a [f32]) -> &mut Self {
        assert!(creases.len().is_multiple_of(2));
        assert!(creases.len() / 2 <= sharpness.len());

        #[cfg(feature = "topology_validation")]
        {
            for (i, &crease_vertex) in creases.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= crease_vertex {
                    // In builder pattern, we can't return Result, so we panic with a clear message
                    panic!(
                        "Crease index[{}] = {} is out of range (should be < {}).",
                        i, crease_vertex, self.descriptor.numVertices
                    );
                }
            }
        }

        self.descriptor.numCreases = sharpness.len().min(i32::MAX as usize) as i32;
        self.descriptor.creaseVertexIndexPairs = creases.as_ptr() as _;
        self.descriptor.creaseWeights = sharpness.as_ptr();
        self
    }

    /// Add corners as vertex indices with corresponding sharpness.
    #[inline]
    pub fn corners(&mut self, corners: &'a [u32], sharpness: &'a [f32]) -> &mut Self {
        assert!(corners.len() <= sharpness.len());

        #[cfg(feature = "topology_validation")]
        {
            for corner in corners.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= *corner.1 {
                    panic!(
                        "Corner index[{}] = {} is out of range (should be < {}).",
                        corner.0, *corner.1, self.descriptor.numVertices
                    );
                }
            }
        }

        self.descriptor.numCorners = sharpness.len().min(i32::MAX as usize) as i32;
        self.descriptor.cornerVertexIndices = corners.as_ptr() as _;
        self.descriptor.cornerWeights = sharpness.as_ptr();
        self
    }

    /// Add holes as face indices.
    #[inline]
    pub fn holes(&mut self, holes: &'a [u32]) -> &mut Self {
        #[cfg(feature = "topology_validation")]
        {
            for hole in holes.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= *hole.1 {
                    panic!(
                        "Hole index[{}] = {} is out of range (should be < {}).",
                        hole.0, *hole.1, self.descriptor.numVertices
                    );
                }
            }
        }

        self.descriptor.numHoles = holes.len().min(i32::MAX as usize) as i32;
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
