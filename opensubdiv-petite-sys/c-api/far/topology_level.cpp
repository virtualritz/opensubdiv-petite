#include <opensubdiv/far/topologyLevel.h>

#include "../vtr/types.hpp"

typedef OpenSubdiv::Far::TopologyLevel TopologyLevel;
typedef OpenSubdiv::Sdc::Crease::Rule SdcCreaseRule;
typedef OpenSubdiv::Vtr::ConstIndexArray ConstIndexArray;
typedef OpenSubdiv::Vtr::ConstLocalIndexArray ConstLocalIndexArray;
typedef OpenSubdiv::Vtr::Index Index;

extern "C"
{
    /// \brief Return the number of vertices in this level
    int TopologyLevel_GetNumVertices(TopologyLevel *tl)
    {
        return tl->GetNumVertices();
    }

    /// \brief Return the number of faces in this level
    int TopologyLevel_GetNumFaces(TopologyLevel *tl)
    {
        return tl->GetNumFaces();
    }

    /// \brief Return the number of edges in this level
    int TopologyLevel_GetNumEdges(TopologyLevel *tl)
    {
        return tl->GetNumEdges();
    }

    /// \brief Return the total number of face-vertices, i.e. the sum of all vertices
    /// for all faces
    int TopologyLevel_GetNumFaceVertices(TopologyLevel *tl)
    {
        return tl->GetNumFaceVertices();
    }
    //@}

    //@{
    /// @name Methods to inspect topological relationships for individual components:
    ///
    /// With three main component types (vertices, faces and edges), for each of the
    /// three components the TopologyLevel stores the incident/adjacent components of
    /// the other two types.  So there are six relationships available for immediate
    /// inspection.  All are accessed by methods that return an array of fixed size
    /// containing the indices of the incident components.
    ///
    /// For some of the relations, i.e. those for which the incident components are
    /// of higher order or 'contain' the component itself (e.g. a vertex has incident
    /// faces that contain it), an additional 'local index' is available that identifies
    /// the component within each of its neighbors.  For example, if vertex V is the
    /// k'th vertex in some face F, then when F occurs in the set of incident vertices
    /// of V, the local index corresponding to F will be k.  The ordering of local
    /// indices matches the ordering of the incident component to which it corresponds.
    //

    /// \brief Access the vertices incident a given face
    ConstIndexArray TopologyLevel_GetFaceVertices(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceVertices(f);
    }

    /// \brief Access the edges incident a given face
    ConstIndexArray TopologyLevel_GetFaceEdges(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceEdges(f);
    }

    /// \brief Access the vertices incident a given edge
    ConstIndexArray TopologyLevel_GetEdgeVertices(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeVertices(e);
    }

    /// \brief Access the faces incident a given edge
    ConstIndexArray TopologyLevel_GetEdgeFaces(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeFaces(e);
    }

    /// \brief Access the faces incident a given vertex
    ConstIndexArray TopologyLevel_GetVertexFaces(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexFaces(v);
    }

    /// \brief Access the edges incident a given vertex
    ConstIndexArray TopologyLevel_GetVertexEdges(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexEdges(v);
    }

    /// \brief Access the local indices of a vertex with respect to its incident faces
    ConstLocalIndexArray
    TopologyLevel_GetVertexFaceLocalIndices(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexFaceLocalIndices(v);
    }

    /// \brief Access the local indices of a vertex with respect to its incident edges
    ConstLocalIndexArray
    TopologyLevel_GetVertexEdgeLocalIndices(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexEdgeLocalIndices(v);
    }

    /// \brief Access the local indices of an edge with respect to its incident faces
    ConstLocalIndexArray
    TopologyLevel_GetEdgeFaceLocalIndices(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeFaceLocalIndices(e);
    }

    /// \brief Identify the edge matching the given vertex pair
    Index TopologyLevel_FindEdge(TopologyLevel *tl, Index v0, Index v1)
    {
        return tl->FindEdge(v0, v1);
    }
    //@}

    //@{
    /// @name Methods to inspect other topological properties of individual components:
    ///

    /// \brief Return if the edge is non-manifold
    bool TopologyLevel_IsEdgeNonManifold(TopologyLevel *tl, Index e)
    {
        return tl->IsEdgeNonManifold(e);
    }

    /// \brief Return if the vertex is non-manifold
    bool TopologyLevel_IsVertexNonManifold(TopologyLevel *tl, Index v)
    {
        return tl->IsVertexNonManifold(v);
    }

    /// \brief Return if the edge is a boundary
    bool TopologyLevel_IsEdgeBoundary(TopologyLevel *tl, Index e)
    {
        return tl->IsEdgeBoundary(e);
    }

    /// \brief Return if the vertex is a boundary
    bool TopologyLevel_IsVertexBoundary(TopologyLevel *tl, Index v)
    {
        return tl->IsVertexBoundary(v);
    }
    //@}

    //@{
    /// @name Methods to inspect feature tags for individual components:
    ///
    /// While only a subset of components may have been tagged with features such
    /// as sharpness, all such features have a default value and so all components
    /// can be inspected.

    /// \brief Return the sharpness assigned a given edge
    float TopologyLevel_GetEdgeSharpness(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeSharpness(e);
    }

    /// \brief Return the sharpness assigned a given vertex
    float TopologyLevel_GetVertexSharpness(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexSharpness(v);
    }

    /// \brief Return if a given face has been tagged as a hole
    bool TopologyLevel_IsFaceHole(TopologyLevel *tl, Index f)
    {
        return tl->IsFaceHole(f);
    }

    /// \brief Return the subdivision rule assigned a given vertex specific to this
    /// level
    SdcCreaseRule TopologyLevel_GetVertexRule(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexRule(v);
    }
    //@}

    //@{
    /// @name Methods to inspect face-varying data:
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

    /// \brief Return the number of face-varying channels (should be same for all
    /// levels)
    int TopologyLevel_GetNumFVarChannels(TopologyLevel *tl)
    {
        return tl->GetNumFVarChannels();
    }

    /// \brief Return the total number of face-varying values in a particular channel
    /// (the upper bound of a face-varying value index)
    int TopologyLevel_GetNumFVarValues(TopologyLevel *tl, int channel)
    {
        return tl->GetNumFVarValues(channel);
    }

    /// \brief Access the face-varying values associated with a particular face
    ConstIndexArray
    TopologyLevel_GetFaceFVarValues(TopologyLevel *tl, Index f, int channel)
    {
        return tl->GetFaceFVarValues(f, channel);
    }

    /// \brief Return if face-varying topology around a vertex matches
    bool
    TopologyLevel_DoesVertexFVarTopologyMatch(TopologyLevel *tl, Index v, int channel)
    {
        return tl->DoesVertexFVarTopologyMatch(v, channel);
    }

    /// \brief Return if face-varying topology across the edge only matches
    bool
    TopologyLevel_DoesEdgeFVarTopologyMatch(TopologyLevel *tl, Index e, int channel)
    {
        return tl->DoesEdgeFVarTopologyMatch(e, channel);
    }

    /// \brief Return if face-varying topology around a face matches
    bool
    TopologyLevel_DoesFaceFVarTopologyMatch(TopologyLevel *tl, Index f, int channel)
    {
        return tl->DoesFaceFVarTopologyMatch(f, channel);
    }

    //@}

    //@{
    /// @name Methods to identify parent or child components in adjoining levels of
    /// refinement:

    /// \brief Access the child faces (in the next level) of a given face
    ConstIndexArray TopologyLevel_GetFaceChildFaces(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceChildFaces(f);
    }

    /// \brief Access the child edges (in the next level) of a given face
    ConstIndexArray TopologyLevel_GetFaceChildEdges(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceChildEdges(f);
    }

    /// \brief Access the child edges (in the next level) of a given edge
    ConstIndexArray TopologyLevel_GetEdgeChildEdges(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeChildEdges(e);
    }

    /// \brief Return the child vertex (in the next level) of a given face
    Index TopologyLevel_GetFaceChildVertex(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceChildVertex(f);
    }

    /// \brief Return the child vertex (in the next level) of a given edge
    Index TopologyLevel_GetEdgeChildVertex(TopologyLevel *tl, Index e)
    {
        return tl->GetEdgeChildVertex(e);
    }

    /// \brief Return the child vertex (in the next level) of a given vertex
    Index TopologyLevel_GetVertexChildVertex(TopologyLevel *tl, Index v)
    {
        return tl->GetVertexChildVertex(v);
    }

    /// \brief Return the parent face (in the previous level) of a given face
    Index TopologyLevel_GetFaceParentFace(TopologyLevel *tl, Index f)
    {
        return tl->GetFaceParentFace(f);
    }
    //@}

    //@{
    /// @name Debugging aides:
    bool TopologyLevel_ValidateTopology(TopologyLevel *tl)
    {
        return tl->ValidateTopology();
    }
    void TopologyLevel_PrintTopology(TopologyLevel *tl, bool children)
    {
        return tl->PrintTopology(children);
    }
    //@}
}