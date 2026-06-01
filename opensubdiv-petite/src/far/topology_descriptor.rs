//! A container holding references to raw topology data.
//!
//! ## Example
//! ```
//! # use opensubdiv_petite::far::TopologyDescriptor;
//! // The positions as a flat buffer. This is commonly used later, with a PrimvarRefiner.
//! let vertices = [1, 1, 1, 1, -1, -1, -1, 1, -1, -1, -1, 1];
//!
//! // Describe the basic topology of our tetrahedron.
//! let tetrahedron = TopologyDescriptor::new(
//!     vertices.len() / 3,
//!     // Four triangles.
//!     &[3; 4],
//!     // Vertex indices for each triangle.
//!     &[2, 1, 0, 3, 2, 0, 1, 3, 0, 2, 3, 1],
//! )
//! .expect("Failed to create topology descriptor");
//!
//! // Make all edges creased with sharpness 8.0.
//! tetrahedron
//!     .creases(&[0, 2, 0, 3, 1, 3, 0, 1, 2, 3, 1, 2], &[8.0; 6])
//!     .expect("Failed to add creases");
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
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct TopologyDescriptor<'a> {
    pub(crate) descriptor: sys::OpenSubdiv_v3_7_0_Far_TopologyDescriptor,
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
        let mut descriptor = unsafe { sys::OpenSubdiv_v3_7_0_Far_TopologyDescriptor::new() };

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
    pub fn creases(mut self, creases: &'a [u32], sharpness: &'a [f32]) -> crate::Result<Self> {
        if !creases.len().is_multiple_of(2) {
            return Err(crate::Error::InvalidTopology(
                "Crease index list must contain vertex index pairs.".to_string(),
            ));
        }
        let pair_count = creases.len() / 2;
        if sharpness.len() != pair_count {
            return Err(crate::Error::InvalidTopology(
                "Crease sharpness list length must match crease pair count.".to_string(),
            ));
        }

        #[cfg(feature = "topology_validation")]
        {
            for (i, &crease_vertex) in creases.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= crease_vertex {
                    return Err(crate::Error::InvalidTopology(format!(
                        "Crease index[{}] = {} is out of range (should be < {}).",
                        i, crease_vertex, self.descriptor.numVertices
                    )));
                }
            }
        }

        self.descriptor.numCreases = pair_count.min(i32::MAX as usize) as i32;
        self.descriptor.creaseVertexIndexPairs = creases.as_ptr() as _;
        self.descriptor.creaseWeights = sharpness.as_ptr();
        Ok(self)
    }

    /// Add corners as vertex indices with corresponding sharpness.
    #[inline]
    pub fn corners(mut self, corners: &'a [u32], sharpness: &'a [f32]) -> crate::Result<Self> {
        if corners.len() > sharpness.len() {
            return Err(crate::Error::InvalidTopology(
                "Corner sharpness list must be at least as long as corner index list.".to_string(),
            ));
        }

        #[cfg(feature = "topology_validation")]
        {
            for (i, &corner_vertex) in corners.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= corner_vertex {
                    return Err(crate::Error::InvalidTopology(format!(
                        "Corner index[{}] = {} is out of range (should be < {}).",
                        i, corner_vertex, self.descriptor.numVertices
                    )));
                }
            }
        }

        self.descriptor.numCorners = sharpness.len().min(i32::MAX as usize) as i32;
        self.descriptor.cornerVertexIndices = corners.as_ptr() as _;
        self.descriptor.cornerWeights = sharpness.as_ptr();
        Ok(self)
    }

    /// Add holes as face indices.
    #[inline]
    pub fn holes(mut self, holes: &'a [u32]) -> crate::Result<Self> {
        #[cfg(feature = "topology_validation")]
        {
            for (i, &hole_index) in holes.iter().enumerate() {
                if self.descriptor.numVertices as u32 <= hole_index {
                    return Err(crate::Error::InvalidTopology(format!(
                        "Hole index[{}] = {} is out of range (should be < {}).",
                        i, hole_index, self.descriptor.numVertices
                    )));
                }
            }
        }

        self.descriptor.numHoles = holes.len().min(i32::MAX as usize) as i32;
        self.descriptor.holeIndices = holes.as_ptr() as _;
        Ok(self)
    }

    /// Set if the topology describes faces with left handed (counter-clockwise)
    /// winding.
    #[inline]
    pub fn left_handed(mut self, left_handed: bool) -> Self {
        self.descriptor.isLeftHanded = left_handed;
        self
    }
}
