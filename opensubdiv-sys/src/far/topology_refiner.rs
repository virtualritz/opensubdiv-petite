use super::topology_level::TopologyLevelPtr;
use crate::sdc;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TopologyRefiner_obj {
    _unused: [u8; 0],
}
pub type TopologyRefinerPtr = *mut TopologyRefiner_obj;

bitfield! {
    #[repr(C)] pub struct UniformOptions(u32);
    u32;
    refinement_level, set_refinement_level: 3, 0;
    order_vertices_from_faces_first, set_order_vertices_from_faces_first: 4;
    full_topology_in_last_level, set_full_topology_in_last_level: 5;
}

impl UniformOptions {
    pub fn new(
        refinement_level: u32,
        order_vertices_from_faces_first: bool,
        full_topology_in_last_level: bool,
    ) -> UniformOptions {
        let mut opt = UniformOptions(0);
        opt.set_refinement_level(refinement_level);
        opt.set_order_vertices_from_faces_first(
            order_vertices_from_faces_first,
        );
        opt.set_full_topology_in_last_level(full_topology_in_last_level);
        opt
    }
}

pub struct UniformOptionsBuilder {
    refinement_level: u32,
    order_vertices_from_faces_first: bool,
    full_topology_in_last_level: bool,
}

impl UniformOptionsBuilder {
    pub fn new() -> UniformOptionsBuilder {
        UniformOptionsBuilder {
            refinement_level: 0,
            order_vertices_from_faces_first: false,
            full_topology_in_last_level: false,
        }
    }

    pub fn refinement_level(mut self, refinement_level: u32) -> Self {
        self.refinement_level = refinement_level;
        self
    }

    pub fn order_vertices_from_faces_first(
        mut self,
        order_vertices_from_faces_first: bool,
    ) -> Self {
        self.order_vertices_from_faces_first = order_vertices_from_faces_first;
        self
    }

    pub fn full_topology_in_last_level(
        mut self,
        full_topology_in_last_level: bool,
    ) -> Self {
        self.full_topology_in_last_level = full_topology_in_last_level;
        self
    }

    pub fn build(self) -> UniformOptions {
        UniformOptions::new(
            self.refinement_level,
            self.order_vertices_from_faces_first,
            self.full_topology_in_last_level,
        )
    }
}

extern "C" {
    pub fn TopologyRefiner_RefineUniform(
        refiner: TopologyRefinerPtr,
        options: UniformOptions,
    );
    pub fn TopologyRefiner_destroy(refiner: TopologyRefinerPtr);
    /// \brief Returns the subdivision scheme
    pub fn TopologyRefiner_GetSchemeType(
        refiner: TopologyRefinerPtr,
    ) -> sdc::Scheme;
    /// \brief Returns the subdivision options
    pub fn TopologyRefiner_GetSchemeOptions(
        refiner: TopologyRefinerPtr,
    ) -> sdc::Options;
    /// \brief Returns true if uniform refinement has been applied
    pub fn TopologyRefiner_IsUniform(refiner: TopologyRefinerPtr) -> bool;
    /// \brief Returns the number of refinement levels
    pub fn TopologyRefiner_GetNumLevels(refiner: TopologyRefinerPtr) -> i32;
    /// \brief Returns the highest level of refinement
    pub fn TopologyRefiner_GetMaxLevel(refiner: TopologyRefinerPtr) -> i32;
    /// \brief Returns the maximum vertex valence in all levels
    pub fn TopologyRefiner_GetMaxValence(refiner: TopologyRefinerPtr) -> i32;
    /// \brief Returns true if faces have been tagged as holes
    pub fn TopologyRefiner_HasHoles(refiner: TopologyRefinerPtr) -> bool;
    /// \brief Returns the total number of vertices in all levels
    pub fn TopologyRefiner_GetNumVerticesTotal(
        refiner: TopologyRefinerPtr,
    ) -> i32;
    /// \brief Returns the total number of edges in all levels
    pub fn TopologyRefiner_GetNumEdgesTotal(refiner: TopologyRefinerPtr)
        -> i32;
    /// \brief Returns the total number of faces in all levels
    pub fn TopologyRefiner_GetNumFacesTotal(refiner: TopologyRefinerPtr)
        -> i32;
    /// \brief Returns the total number of face vertices in all levels
    pub fn TopologyRefiner_GetNumFaceVerticesTotal(
        refiner: TopologyRefinerPtr,
    ) -> i32;
    /// \brief Returns a handle to access data specific to a particular level
    pub fn TopologyRefiner_GetLevel(
        refiner: TopologyRefinerPtr,
        level: i32,
    ) -> TopologyLevelPtr;
}
