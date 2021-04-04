use super::topology_level::TopologyLevelPtr;

pub type TopologyRefinerPtr = *mut crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner;

extern "C" {

    /// \brief Returns true if uniform refinement has been applied
    pub fn TopologyRefiner_GetNumLevels(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns the maximum vertex valence in all levels
    pub fn TopologyRefiner_GetMaxValence(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns true if faces have been tagged as holes
    pub fn TopologyRefiner_GetNumVerticesTotal(
        refiner: TopologyRefinerPtr,
    ) -> u32;
    /// \brief Returns the total number of edges in all levels
    pub fn TopologyRefiner_GetNumEdgesTotal(refiner: TopologyRefinerPtr)
        -> u32;
    /// \brief Returns the total number of faces in all levels
    pub fn TopologyRefiner_GetNumFacesTotal(refiner: TopologyRefinerPtr)
        -> u32;
    /// \brief Returns the total number of face vertices in all levels
    pub fn TopologyRefiner_GetNumFaceVerticesTotal(
        refiner: TopologyRefinerPtr,
    ) -> u32;
    /// \brief Returns a handle to access data specific to a particular level
    pub fn TopologyRefiner_GetLevel(
        refiner: TopologyRefinerPtr,
        level: i32,
    ) -> TopologyLevelPtr;
}
