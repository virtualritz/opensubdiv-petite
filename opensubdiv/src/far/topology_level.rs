//! An interface for accessing data in a specific level of a refined topology
//! hierarchy.
use super::topology_refiner::TopologyRefiner;
use opensubdiv_sys as sys;
use std::convert::TryInto;

use sys::vtr::types::{Index, LocalIndex};

const INVALID_INDEX: u32 = u32::MAX; // aka: -1i32

/// Provides an interface to data in a specific level of a
/// topology hierarchy.  Instances of `TopologyLevel` are created and owned by a
/// [`TopologyRefiner`](crate::far::TopologyRefiner), which will return
/// const-references to them.  Such references are only valid during the
/// lifetime of the `TopologyRefiner` that created and returned them, and only
/// for a given refinement. I.e. if the `TopologyRefiner` is re-refined, any
/// references to `TopoologyLevel`s are invalidated.
// FIXME: We should really try and encode this in the type system - maybe
// TopologyRefiner could create and store a dummy Refinment struct on the
// TopologyRefiner, which this holds a reference to.
pub struct TopologyLevel<'a> {
    pub(crate) ptr: sys::far::TopologyLevelPtr,
    pub(crate) refiner: std::marker::PhantomData<&'a TopologyRefiner>,
}

/// ### Methods to Inspect the Overall Inventory of Components
///
/// All three main component types are indexed locally within each level.  For
/// some topological relationships – notably face-vertices, which is often
/// the only relationship of interest – the total number of entries is also
/// made available.
impl<'a> TopologyLevel<'a> {
    /// Return the number of vertices in this level.
    pub fn vertices_len(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumVertices(self.ptr) as _ }
    }

    /// Return the number of faces in this level.
    pub fn faces_len(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFaces(self.ptr) as _ }
    }

    /// Return the number of edges in this level.
    pub fn edges_len(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumEdges(self.ptr) as _ }
    }

    /// Returns the total number of face-vertices; i.e. the sum of all vertices
    /// for all faces.
    pub fn face_vertices_len(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFaceVertices(self.ptr) as _ }
    }

    /// Returns an iterator over the face vertices of this level.
    pub fn face_vertices_iter(&self) -> FaceVerticesIterator {
        FaceVerticesIterator {
            level: self,
            current: 0,
            num: self.face_vertices_len() as _,
        }
    }
}

/// ### Methods to Inspect Topological Relationships for Individual Components
///
/// With three main component types (vertices, faces and edges), for each of the
/// three components the TopologyLevel stores the incident/adjacent components
/// of the other two types.  So there are six relationships available for
/// immediate inspection.  All are accessed by methods that return an array of
/// fixed size containing the indices of the incident components.
///
/// For some of the relations, i.e. those for which the incident components are
/// of higher order or 'contain' the component itself (e.g. a vertex has
/// incident faces that contain it), an additional 'local index' is available
/// that identifies the component within each of its neighbors.  For example, if
/// vertex V is the k'th vertex in some face F, then when F occurs in the set of
/// incident vertices of V, the local index corresponding to F will be k.  The
/// ordering of local indices matches the ordering of the incident component to
/// which it corresponds.
impl<'a> TopologyLevel<'a> {
    /// Access the vertices incident a given face
    pub fn face_vertices(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceVertices(self.ptr, f);
            if 0 == arr.size()
                || arr.begin().is_null()
                || self.faces_len() <= f as _
            {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the edges incident a given face
    pub fn face_edges(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceEdges(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the vertices incident a given edge
    pub fn edge_vertices(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeVertices(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the faces incident a given edge
    pub fn edge_faces(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeFaces(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the faces incident a given vertex
    pub fn vertex_faces(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexFaces(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the edges incident a given vertex
    pub fn vertex_edges(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexEdges(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the local indices of a vertex with respect to its incident faces
    pub fn vertex_face_local_indices(&self, f: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr =
                sys::far::TopologyLevel_GetVertexFaceLocalIndices(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the local indices of a vertex with respect to its incident edges
    pub fn vertex_edge_local_indices(&self, f: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr =
                sys::far::TopologyLevel_GetVertexEdgeLocalIndices(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Access the local indices of an edge with respect to its incident faces
    pub fn edge_face_local_indices(&self, f: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr =
                sys::far::TopologyLevel_GetEdgeFaceLocalIndices(self.ptr, f);
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Identify the edge matching the given vertex pair.
    #[inline]
    pub fn find_edge(&self, v0: Index, v1: Index) -> Option<Index> {
        let i = unsafe { sys::far::TopologyLevel_FindEdge(self.ptr, v0, v1) };
        if INVALID_INDEX == i {
            None
        } else {
            Some(i)
        }
    }
}

/// An iterator over the face vertices of this [`TopologyLevel`].
pub struct FaceVerticesIterator<'a> {
    level: &'a TopologyLevel<'a>,
    num: u32,
    current: u32,
}

impl<'a> Iterator for FaceVerticesIterator<'a> {
    type Item = &'a [Index];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.num {
            None
        } else {
            self.current += 1;
            self.level.face_vertices(self.current - 1)
        }
    }
}

/// Methods to inspect other topological properties of individual components
impl<'a> TopologyLevel<'a> {
    /// Return if the edge is non-manifold.
    #[inline]
    pub fn is_edge_non_manifold(&self, e: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsEdgeNonManifold(self.ptr, e) }
    }

    /// Returns `true` if the vertex is non-manifold.
    #[inline]
    pub fn is_vertex_non_manifold(&self, v: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsVertexNonManifold(self.ptr, v) }
    }

    /// Returns`true` if the edge is a boundary.
    #[inline]
    pub fn is_edge_boundary(&self, e: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsEdgeBoundary(self.ptr, e) }
    }

    /// Return`true` if the vertex is a boundary.
    #[inline]
    pub fn is_vertex_boundary(&self, v: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsVertexBoundary(self.ptr, v) }
    }
}

/// Methods to inspect face-varying data.
///
/// Face-varying data is organized into topologically independent channels,
/// each with an integer identifier.  Access to face-varying data generally
/// requires the specification of a channel, though with a single channel
/// being a common situation the first/only channel will be assumed if
/// unspecified.
///
/// A face-varying channel is composed of a set of values that may be shared
/// by faces meeting at a common vertex.  Just as there are sets of vertices
/// that are associated with faces by index (ranging from 0 to
/// num-vertices - 1), face-varying values are also referenced by index
/// (ranging from 0 to num-values -1).
///
/// The face-varying values associated with a face are accessed similarly to
/// the way in which vertices associated with the face are accessed -- an
/// array of fixed size containing the indices for each corner is provided
/// for inspection, iteration, etc.
///
/// When the face-varying topology around a vertex "matches", it has the
/// same limit properties and so results in the same limit surface when
/// collections of adjacent vertices match.  Like other references to
/// "topology", this includes consideration of sharpness.  So it may be
/// that face-varying values are assigned around a vertex on a boundary in
/// a way that appears to match, but the face-varying interpolation option
/// requires sharpening of that vertex in face-varying space -- the
/// difference in the topology of the resulting limit surfaces leading to
/// the query returning false for the match.  The edge case is simpler in
/// that it only considers continuity across the edge, not the entire
/// neighborhood around each end vertex.
impl<'a> TopologyLevel<'a> {
    /// Return the number of face-varying channels (should be same for all
    /// levels).
    #[inline]
    pub fn face_varying_channels_len(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFVarChannels(self.ptr) as _ }
    }

    /// Return the total number of face-varying values in a particular channel.
    /// (the upper bound of a face-varying value index).
    #[inline]
    pub fn face_varying_values_len(&self, channel: usize) -> usize {
        unsafe {
            sys::far::TopologyLevel_GetNumFVarValues(
                self.ptr,
                channel.try_into().unwrap(),
            ) as _
        }
    }

    /// Access the face-varying values associated with a particular face.
    #[inline]
    pub fn face_varying_values_on_face(
        &self,
        f: Index,
        channel: usize,
    ) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceFVarValues(
                self.ptr,
                f,
                channel as _,
            );
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin(),
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Return `true` if face-varying topology around a vertex matches.
    #[inline]
    pub fn vertex_face_varying_topology_matches(
        &self,
        v: Index,
        channel: usize,
    ) -> bool {
        unsafe {
            sys::far::TopologyLevel_DoesVertexFVarTopologyMatch(
                self.ptr,
                v,
                channel.try_into().unwrap(),
            )
        }
    }

    /// Return `true` if face-varying topology across the edge only matches.
    #[inline]
    pub fn edge_face_varying_topology_matches(
        &self,
        e: Index,
        channel: u32,
    ) -> bool {
        unsafe {
            sys::far::TopologyLevel_DoesEdgeFVarTopologyMatch(
                self.ptr,
                e,
                channel.try_into().unwrap(),
            )
        }
    }

    /// Return `true` if face-varying topology around a face matches.
    #[inline]
    pub fn face_varying_topology_on_face_matches(
        &self,
        f: Index,
        channel: u32,
    ) -> bool {
        unsafe {
            sys::far::TopologyLevel_DoesFaceFVarTopologyMatch(
                self.ptr,
                f,
                channel.try_into().unwrap(),
            )
        }
    }
}

/// Methods to identify parent or child components in adjoining levels of
/// refinement.
impl<'a> TopologyLevel<'a> {
    /// Access the child faces (in the next level) of a given face.
    #[inline]
    pub fn face_child_faces(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceChildFaces(self.ptr, f);
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin(),
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Access the child edges (in the next level) of a given face.
    #[inline]
    pub fn face_child_edges(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceChildEdges(self.ptr, f);
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin(),
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Access the child edges (in the next level) of a given edge.
    #[inline]
    pub fn edge_child_edges(&self, e: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeChildEdges(self.ptr, e);
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin(),
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Return the child vertex (in the next level) of a given face.
    #[inline]
    pub fn face_child_vertex(&self, f: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetFaceChildVertex(self.ptr, f) }
    }

    /// Return the child vertex (in the next level) of a given edge.
    #[inline]
    pub fn edge_child_vertex(&self, e: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetEdgeChildVertex(self.ptr, e) }
    }

    /// Return the child vertex (in the next level) of a given vertex.
    #[inline]
    pub fn vertex_child_vertex(&self, v: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetVertexChildVertex(self.ptr, v) }
    }

    /// Return the parent face (in the previous level) of a given face.
    #[inline]
    pub fn face_parent_face(&self, f: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetFaceParentFace(self.ptr, f) }
    }
}
