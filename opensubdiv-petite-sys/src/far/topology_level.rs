use crate::vtr::types::*;

pub type TopologyLevelPtr = *mut crate::OpenSubdiv_v3_5_0_Far_TopologyLevel;

#[link(name = "osl-capi", kind = "static")]
extern "C" {
    /// Return the number of vertices in this level
    pub fn TopologyLevel_GetNumVertices(tl: TopologyLevelPtr) -> u32;
    /// Return the number of faces in this level
    pub fn TopologyLevel_GetNumFaces(tl: TopologyLevelPtr) -> u32;
    /// Return the number of edges in this level
    pub fn TopologyLevel_GetNumEdges(tl: TopologyLevelPtr) -> u32;
    /// Return the total number of face-vertices, i.e. the sum of all
    /// vertices for all faces
    pub fn TopologyLevel_GetNumFaceVertices(tl: TopologyLevelPtr) -> u32;

    /// Methods to inspect topological relationships for individual
    /// components:
    ///
    /// With three main component types (vertices, faces and edges), for each of
    /// the three components the TopologyLevel stores the incident/adjacent
    /// components of the other two types.  So there are six relationships
    /// available for immediate inspection.  All are accessed by methods
    /// that return an array of fixed size containing the indices of the
    /// incident components.
    ///
    /// For some of the relations, i.e. those for which the incident components
    /// are of higher order or 'contain' the component itself (e.g. a vertex
    /// has incident faces that contain it), an additional 'local index' is
    /// available that identifies the component within each of its
    /// neighbors.  For example, if vertex V is the k'th vertex in some face
    /// F, then when F occurs in the set of incident vertices of V,
    /// the local index corresponding to F will be k.  The ordering of local
    /// indices matches the ordering of the incident component to which it
    /// corresponds.

    /// Access the vertices incident a given face
    pub fn TopologyLevel_GetFaceVertices(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> ConstIndexArray;
    /// Access the edges incident a given face
    pub fn TopologyLevel_GetFaceEdges(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> ConstIndexArray;
    /// Access the vertices incident a given edge
    pub fn TopologyLevel_GetEdgeVertices(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> ConstIndexArray;
    /// Access the faces incident a given edge
    pub fn TopologyLevel_GetEdgeFaces(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> ConstIndexArray;

    /// Access the faces incident a given vertex
    pub fn TopologyLevel_GetVertexFaces(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> ConstIndexArray;
    /// Access the edges incident a given vertex
    pub fn TopologyLevel_GetVertexEdges(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> ConstIndexArray;

    /// Access the local indices of a vertex with respect to its incident
    /// faces
    pub fn TopologyLevel_GetVertexFaceLocalIndices(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> ConstLocalIndexArray;

    /// Access the local indices of a vertex with respect to its incident
    /// edges
    pub fn TopologyLevel_GetVertexEdgeLocalIndices(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> ConstLocalIndexArray;

    /// Access the local indices of an edge with respect to its incident
    /// faces
    pub fn TopologyLevel_GetEdgeFaceLocalIndices(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> ConstLocalIndexArray;

    /// Identify the edge matching the given vertex pair
    pub fn TopologyLevel_FindEdge(
        tl: TopologyLevelPtr,
        v0: Index,
        v1: Index,
    ) -> Index;

    /// Methods to inspect other topological properties of individual
    /// components:

    /// Return if the edge is non-manifold
    pub fn TopologyLevel_IsEdgeNonManifold(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> bool;

    /// Return if the vertex is non-manifold
    pub fn TopologyLevel_IsVertexNonManifold(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> bool;

    /// Return if the edge is a boundary
    pub fn TopologyLevel_IsEdgeBoundary(tl: TopologyLevelPtr, e: Index)
        -> bool;

    /// Return if the vertex is a boundary
    pub fn TopologyLevel_IsVertexBoundary(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> bool;

    /// Methods to inspect feature tags for individual components:
    ///
    /// While only a subset of components may have been tagged with features
    /// such as sharpness, all such features have a default value and so all
    /// components can be inspected.

    /// Return the sharpness assigned a given edge
    pub fn TopologyLevel_GetEdgeSharpness(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> f32;

    /// Return the sharpness assigned a given vertex
    pub fn TopologyLevel_GetVertexSharpness(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> f32;

    /// Return if a given face has been tagged as a hole
    pub fn TopologyLevel_IsFaceHole(tl: TopologyLevelPtr, f: Index) -> bool;

    /// Return the subdivision rule assigned a given vertex specific to
    /// this level
    pub fn TopologyLevel_GetVertexRule(v: Index) -> u32;

    //. Methods to inspect face-varying data:
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

    /// Return the number of face-varying channels (should be same for
    /// all levels)
    pub fn TopologyLevel_GetNumFVarChannels(tl: TopologyLevelPtr) -> i32;

    /// Return the total number of face-varying values in a particular
    /// channel (the upper bound of a face-varying value index)
    pub fn TopologyLevel_GetNumFVarValues(
        tl: TopologyLevelPtr,
        channel: i32,
    ) -> u32;

    /// Access the face-varying values associated with a particular face
    pub fn TopologyLevel_GetFaceFVarValues(
        tl: TopologyLevelPtr,
        f: Index,
        channel: i32,
    ) -> ConstIndexArray;

    /// Return if face-varying topology around a vertex matches
    pub fn TopologyLevel_DoesVertexFVarTopologyMatch(
        tl: TopologyLevelPtr,
        v: Index,
        channel: i32,
    ) -> bool;

    /// Return if face-varying topology across the edge only matches
    pub fn TopologyLevel_DoesEdgeFVarTopologyMatch(
        tl: TopologyLevelPtr,
        e: Index,
        channel: i32,
    ) -> bool;

    /// Return if face-varying topology around a face matches
    pub fn TopologyLevel_DoesFaceFVarTopologyMatch(
        tl: TopologyLevelPtr,
        f: Index,
        channel: i32,
    ) -> bool;

    /// Methods to identify parent or child components in adjoining levels
    /// of refinement:

    /// Access the child faces (in the next level) of a given face
    pub fn TopologyLevel_GetFaceChildFaces(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> ConstIndexArray;

    /// Access the child edges (in the next level) of a given face
    pub fn TopologyLevel_GetFaceChildEdges(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> ConstIndexArray;

    /// Access the child edges (in the next level) of a given edge
    pub fn TopologyLevel_GetEdgeChildEdges(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> ConstIndexArray;

    /// Return the child vertex (in the next level) of a given face
    pub fn TopologyLevel_GetFaceChildVertex(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> Index;

    /// Return the child vertex (in the next level) of a given edge
    pub fn TopologyLevel_GetEdgeChildVertex(
        tl: TopologyLevelPtr,
        e: Index,
    ) -> Index;

    /// Return the child vertex (in the next level) of a given vertex
    pub fn TopologyLevel_GetVertexChildVertex(
        tl: TopologyLevelPtr,
        v: Index,
    ) -> Index;

    /// Return the parent face (in the previous level) of a given face
    pub fn TopologyLevel_GetFaceParentFace(
        tl: TopologyLevelPtr,
        f: Index,
    ) -> Index;

    /// Debugging aides:
    pub fn TopologyLevel_ValidateTopology(tl: TopologyLevelPtr) -> bool;
    pub fn TopologyLevel_PrintTopology(tl: TopologyLevelPtr, children: bool);
}
