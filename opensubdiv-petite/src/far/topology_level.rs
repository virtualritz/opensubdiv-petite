//! An interface for accessing data in a specific level of a refined topology
//! hierarchy.
use super::topology_refiner::TopologyRefiner;
use opensubdiv_petite_sys as sys;

use crate::Index;
use sys::vtr::types::LocalIndex;

const INVALID_INDEX: u32 = u32::MAX; // aka: -1i32

/// Provides an interface to data in a specific level of a
/// topology hierarchy.  Instances of `TopologyLevel` are created and owned by a
/// [`TopologyRefiner`], which will return
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

// SAFETY: TopologyLevel is a read-only view into the underlying C++ object
// which is immutable after creation. The TopologyRefiner ensures the data
// remains valid for the lifetime 'a.
unsafe impl<'a> Send for TopologyLevel<'a> {}
unsafe impl<'a> Sync for TopologyLevel<'a> {}

/// ### Methods to Inspect the Overall Inventory of Components
///
/// All three main component types are indexed locally within each level.  For
/// some topological relationships – notably face-vertices, which is often
/// the only relationship of interest – the total number of entries is also
/// made available.
impl<'a> TopologyLevel<'a> {
    /// Returns the number of vertices in this level.
    pub fn vertex_count(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumVertices(self.ptr) as _ }
    }

    /// Return the number of vertices in this level.
    #[deprecated(since = "0.3.0", note = "Use `vertex_count` instead")]
    #[inline]
    pub fn vertices_len(&self) -> usize {
        self.vertex_count()
    }

    /// Returns the number of faces in this level.
    pub fn face_count(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFaces(self.ptr) as _ }
    }

    /// Return the number of faces in this level.
    #[deprecated(since = "0.3.0", note = "Use `face_count` instead")]
    #[inline]
    pub fn faces_len(&self) -> usize {
        self.face_count()
    }

    /// Returns the number of edges in this level.
    pub fn edge_count(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumEdges(self.ptr) as _ }
    }

    /// Return the number of edges in this level.
    #[deprecated(since = "0.3.0", note = "Use `edge_count` instead")]
    #[inline]
    pub fn edges_len(&self) -> usize {
        self.edge_count()
    }

    /// Returns the total number of face-vertices -- the sum of all vertices
    /// for all faces.
    pub fn face_vertex_count(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFaceVertices(self.ptr) as _ }
    }

    /// Returns the total number of face-vertices; i.e. the sum of all vertices
    /// for all faces.
    #[deprecated(since = "0.3.0", note = "Use `face_vertex_count` instead")]
    #[inline]
    pub fn face_vertices_len(&self) -> usize {
        self.face_vertex_count()
    }

    /// Returns an iterator over the face vertices of this level.
    pub fn face_vertices_iter(&self) -> FaceVerticesIter<'_> {
        FaceVerticesIter {
            level: self,
            current: 0,
            num: self.face_count() as _,
        }
    }

    /// Returns a parallel iterator over the face vertices of this level.
    ///
    /// This method is only available when the `rayon` feature is enabled.
    #[cfg(feature = "rayon")]
    pub fn face_vertices_par_iter(&self) -> FaceVerticesParIter<'_> {
        FaceVerticesParIter::new(self)
    }
}

/// ### Methods to Inspect Topological Relationships for Individual Components
///
/// With three main component types (*vertices*, *faces* and *edges*), for each
/// of the three components the `TopologyLevel` stores the incident/adjacent
/// components of the other two types.  So there are six relationships available
/// for immediate inspection.  All are accessed by methods that return an array
/// of fixed size containing the indices of the incident components.
///
/// For some of the relations, i.e. those for which the incident components are
/// of higher order or 'contain' the component itself (e.g. a vertex has
/// incident faces that contain it), an additional 'local index' is available
/// that identifies the component within each of its neighbors.
///
/// For example, if vertex `V` is the `k`th vertex in some face `F`, then when
/// `F` occurs in the set of incident vertices of `V`, the local index
/// corresponding to `F` will be `k`.  The ordering of local indices matches
/// the ordering of the incident component to which it corresponds.
impl<'a> TopologyLevel<'a> {
    /// Returns the vertices incident to a given face.
    pub fn face_vertices(&self, face: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceVertices(self.ptr, face.into());
            if 0 == arr.size() || arr.begin().is_null() || self.face_count() <= face.into() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the edges incident to a given face.
    pub fn face_edges(&self, face: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceEdges(self.ptr, face.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the vertices incident to a given edge.
    pub fn edge_vertices(&self, edge: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeVertices(self.ptr, edge.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the faces incident to a given edge.
    pub fn edge_faces(&self, f: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeFaces(self.ptr, f.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the faces incident to a given vertex.
    pub fn vertex_faces(&self, vertex: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexFaces(self.ptr, vertex.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the edges incident to a given vertex.
    pub fn vertex_edges(&self, vertex: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexEdges(self.ptr, vertex.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as _,
                ))
            }
        }
    }

    /// Returns the local indices of a vertex with respect to its incident
    /// faces.
    pub fn vertex_face_local_indices(&self, vertex: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexFaceLocalIndices(self.ptr, vertex.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Returns the local indices of a vertex with respect to its incident
    /// edges.
    pub fn vertex_edge_local_indices(&self, vertex: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetVertexEdgeLocalIndices(self.ptr, vertex.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Returns the local indices of an edge with respect to its incident faces.
    pub fn edge_face_local_indices(&self, face: Index) -> Option<&[LocalIndex]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeFaceLocalIndices(self.ptr, face.into());
            if arr.size() == 0 || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(arr.begin(), arr.size() as _))
            }
        }
    }

    /// Identify the edge matching the given vertex pair.
    #[inline]
    pub fn find_edge(&self, vertex0: Index, vertex1: Index) -> Option<Index> {
        let i =
            unsafe { sys::far::TopologyLevel_FindEdge(self.ptr, vertex0.into(), vertex1.into()) };
        if INVALID_INDEX == i {
            None
        } else {
            Some(i.into())
        }
    }
}

/// An iterator over the face vertices of this [`TopologyLevel`].
#[derive(Copy, Clone)]
pub struct FaceVerticesIter<'a> {
    level: &'a TopologyLevel<'a>,
    num: u32,
    current: u32,
}

impl<'a> Iterator for FaceVerticesIter<'a> {
    type Item = &'a [Index];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.num {
            None
        } else {
            self.current += 1;
            self.level.face_vertices((self.current - 1).into())
        }
    }
}

// Parallel iterator support with rayon
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// A parallel iterator over the face vertices of this [`TopologyLevel`].
#[cfg(feature = "rayon")]
#[derive(Copy, Clone)]
pub struct FaceVerticesParIter<'a> {
    level: &'a TopologyLevel<'a>,
    num: u32,
}

#[cfg(feature = "rayon")]
impl<'a> FaceVerticesParIter<'a> {
    fn new(level: &'a TopologyLevel<'a>) -> Self {
        Self {
            level,
            num: level.face_count() as u32,
        }
    }
}

#[cfg(feature = "rayon")]
impl<'a> ParallelIterator for FaceVerticesParIter<'a> {
    type Item = &'a [Index];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        (0..self.num)
            .into_par_iter()
            .map(|i| self.level.face_vertices(i.into()).unwrap())
            .drive_unindexed(consumer)
    }
}

#[cfg(feature = "rayon")]
impl<'a> IndexedParallelIterator for FaceVerticesParIter<'a> {
    fn len(&self) -> usize {
        self.num as usize
    }

    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::Consumer<Self::Item>,
    {
        (0..self.num)
            .into_par_iter()
            .map(|i| self.level.face_vertices(i.into()).unwrap())
            .drive(consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: rayon::iter::plumbing::ProducerCallback<Self::Item>,
    {
        (0..self.num)
            .into_par_iter()
            .map(|i| self.level.face_vertices(i.into()).unwrap())
            .with_producer(callback)
    }
}

/// ### Methods to Inspect Other Topological Properties of Individual Components
impl<'a> TopologyLevel<'a> {
    /// Returns `true` if the edge is non-manifold.
    #[inline]
    pub fn is_edge_non_manifold(&self, edge: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsEdgeNonManifold(self.ptr, edge.into()) }
    }

    /// Returns `true` if the vertex is non-manifold.
    #[inline]
    pub fn is_vertex_non_manifold(&self, vertex: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsVertexNonManifold(self.ptr, vertex.into()) }
    }

    /// Returns `true` if the edge is a boundary.
    #[inline]
    pub fn is_edge_boundary(&self, edge: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsEdgeBoundary(self.ptr, edge.into()) }
    }

    /// Returns `true` if the vertex is a boundary.
    #[inline]
    pub fn is_vertex_boundary(&self, vertex: Index) -> bool {
        unsafe { sys::far::TopologyLevel_IsVertexBoundary(self.ptr, vertex.into()) }
    }
}

/// ### Methods to Inspect Face-Varying Data.
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
    /// Returns the number of face-varying channels (should be same for all
    /// levels).
    #[inline]
    pub fn face_varying_channel_count(&self) -> usize {
        unsafe { sys::far::TopologyLevel_GetNumFVarChannels(self.ptr) as _ }
    }

    /// Return the number of face-varying channels (should be same for all
    /// levels).
    #[deprecated(since = "0.3.0", note = "Use `face_varying_channel_count` instead")]
    #[inline]
    pub fn face_varying_channels_len(&self) -> usize {
        self.face_varying_channel_count()
    }

    /// Returns the total number of face-varying values in a particular channel
    /// (the upper bound of a face-varying value index).
    #[inline]
    pub fn face_varying_value_count(&self, channel: usize) -> usize {
        unsafe {
            // Channel index is typically small (0-3), so saturating to i32::MAX is
            // reasonable
            let channel_i32 = channel.min(i32::MAX as usize) as i32;
            sys::far::TopologyLevel_GetNumFVarValues(self.ptr, channel_i32) as _
        }
    }

    /// Return the total number of face-varying values in a particular channel.
    /// (the upper bound of a face-varying value index).
    #[deprecated(since = "0.3.0", note = "Use `face_varying_value_count` instead")]
    #[inline]
    pub fn face_varying_values_len(&self, channel: usize) -> usize {
        self.face_varying_value_count(channel)
    }

    /// Returns the face-varying values associated with a particular face.
    #[inline]
    pub fn face_varying_values_on_face(&self, face: Index, channel: usize) -> Option<&[Index]> {
        unsafe {
            let arr =
                sys::far::TopologyLevel_GetFaceFVarValues(self.ptr, face.into(), channel as _);
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Returns `true` if face-varying topology around a vertex matches.
    #[inline]
    pub fn vertex_face_varying_topology_matches(&self, vertex: Index, channel: usize) -> bool {
        unsafe {
            let channel_i32 = channel.min(i32::MAX as usize) as i32;
            sys::far::TopologyLevel_DoesVertexFVarTopologyMatch(
                self.ptr,
                vertex.into(),
                channel_i32,
            )
        }
    }

    /// Returns `true` if face-varying topology across the edge only matches.
    #[inline]
    pub fn edge_face_varying_topology_matches(&self, edge: Index, channel: usize) -> bool {
        unsafe {
            let channel_i32 = channel.min(i32::MAX as usize) as i32;
            sys::far::TopologyLevel_DoesEdgeFVarTopologyMatch(self.ptr, edge.into(), channel_i32)
        }
    }

    /// Returns `true` if face-varying topology around a face matches.
    #[inline]
    pub fn face_varying_topology_on_face_matches(&self, face: Index, channel: usize) -> bool {
        unsafe {
            let channel_i32 = channel.min(i32::MAX as usize) as i32;
            sys::far::TopologyLevel_DoesFaceFVarTopologyMatch(self.ptr, face.into(), channel_i32)
        }
    }
}

/// ### Methods to Identify Parent or Child Components in Adjoining Levels of
/// Refinement.
impl<'a> TopologyLevel<'a> {
    /// Returns the child faces (in the next level) of a given face.
    #[inline]
    pub fn face_child_faces(&self, face: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceChildFaces(self.ptr, face.into());
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Returns the child edges (in the next level) of a given face.
    #[inline]
    pub fn face_child_edges(&self, face: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetFaceChildEdges(self.ptr, face.into());
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Returns the child edges (in the next level) of a given edge.
    #[inline]
    pub fn edge_child_edges(&self, edge: Index) -> Option<&[Index]> {
        unsafe {
            let arr = sys::far::TopologyLevel_GetEdgeChildEdges(self.ptr, edge.into());
            if 0 == arr.size() || arr.begin().is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    arr.begin() as *const Index,
                    arr.size() as usize,
                ))
            }
        }
    }

    /// Returns the child vertex (in the next level) of a given face.
    #[inline]
    pub fn face_child_vertex(&self, face: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetFaceChildVertex(self.ptr, face.into()).into() }
    }

    /// Returns the child vertex (in the next level) of a given edge.
    #[inline]
    pub fn edge_child_vertex(&self, edge: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetEdgeChildVertex(self.ptr, edge.into()).into() }
    }

    /// Returns the child vertex (in the next level) of a given vertex.
    #[inline]
    pub fn vertex_child_vertex(&self, vertex: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetVertexChildVertex(self.ptr, vertex.into()).into() }
    }

    /// Returns the parent face (in the previous level) of a given face.
    #[inline]
    pub fn face_parent_face(&self, face: Index) -> Index {
        unsafe { sys::far::TopologyLevel_GetFaceParentFace(self.ptr, face.into()).into() }
    }
}
