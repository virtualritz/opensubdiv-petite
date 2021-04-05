use super::topology_level::TopologyLevelPtr;
use num_enum::TryFromPrimitive;

#[repr(u32)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum Scheme {
    Bilinear = crate::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_BILINEAR,
    CatmullClark = crate::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_CATMARK,
    Loop = crate::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_LOOP,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum BoundaryInterpolation {
    None = crate::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_NONE,
    EdgeOnly = crate::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_ONLY,
    EdgeAndCorner = crate::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_AND_CORNER,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum FaceVaryingLinearInterpolation {
    None = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_NONE,
    CornersOnly = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_ONLY,
    CornersPlusOne = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS1,
    CornersPlusTwo = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS2,
    Boundaries = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_BOUNDARIES,
    All = crate::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_ALL,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CreasingMethod {
    Uniform =
        crate::OpenSubdiv_v3_4_4_Sdc_Options_CreasingMethod_CREASE_UNIFORM,
    Chaikin =
        crate::OpenSubdiv_v3_4_4_Sdc_Options_CreasingMethod_CREASE_CHAIKIN,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TriangleSubdivision {
    CatmullClark = crate::OpenSubdiv_v3_4_4_Sdc_Options_TriangleSubdivision_TRI_SUB_CATMARK,
    Smooth =
        crate::OpenSubdiv_v3_4_4_Sdc_Options_TriangleSubdivision_TRI_SUB_SMOOTH,
}

pub type UniformRefinementOptions =
    crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner_UniformOptions;
pub type AdaptiveRefinementOptions =
    crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner_AdaptiveOptions;
pub type Options = crate::OpenSubdiv_v3_4_4_Far_TopologyRefinerFactory_Options;
pub type TopologyRefiner = crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner;
pub type TopologyRefinerPtr = *mut TopologyRefiner;

extern "C" {
    pub fn TopologyRefinerFactory_TopologyDescriptor_Create(
        descriptor: *const crate::OpenSubdiv_v3_4_4_Far_TopologyDescriptor,
        options: crate::OpenSubdiv_v3_4_4_Far_TopologyRefinerFactory_Options,
    ) -> TopologyRefinerPtr;
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
