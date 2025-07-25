use super::topology_level::TopologyLevelPtr;
use num_enum::TryFromPrimitive;

// Constants for "None" values that are used when Option::None is passed
pub const VTX_BOUNDARY_NONE: u8 =
    crate::OpenSubdiv_v3_6_1_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_NONE as u8;
pub const FVAR_LINEAR_NONE: u8 =
    crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_NONE as u8;

#[repr(u32)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum Scheme {
    /// *Bilinear* interpolation.
    Bilinear = crate::OpenSubdiv_v3_6_1_Sdc_SchemeType_SCHEME_BILINEAR as _,
    /// [*Catmull-Clark* subdivision](https://en.wikipedia.org/wiki/Catmull%E2%80%93Clark_subdivision_surface).
    CatmullClark = crate::OpenSubdiv_v3_6_1_Sdc_SchemeType_SCHEME_CATMARK as _,
    /// [*Loop* subdivision](https://en.wikipedia.org/wiki/Loop_subdivision_surface).
    Loop = crate::OpenSubdiv_v3_6_1_Sdc_SchemeType_SCHEME_LOOP as _,
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum BoundaryInterpolation {
    /// A sequence of boundary vertices defines a smooth curve to which the
    /// limit surface along boundary faces extends.
    EdgeOnly =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_ONLY as _,
    /// Similar to edge-only but the smooth curve resulting on the boundary is
    /// made to interpolate corner vertices (vertices with exactly one incident
    /// face).
    EdgeAndCorner =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_AND_CORNER
            as _,
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum FaceVaryingLinearInterpolation {
    /// Linearly interpolate (sharpen or pin) corners only,
    CornersOnly =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_ONLY as _,
    /// `CornersOnly` + sharpening of junctions of three or more regions.
    CornersPlusOne =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS1 as _,
    /// `CornersPlusOne` + sharpening of darts and concave corners.
    CornersPlusTwo =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS2 as _,
    /// `Linear interpolation along all boundary edges and corners.
    Boundaries =
        crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_BOUNDARIES as _,
    /// Linear interpolation everywhere (boundaries and interior).
    All = crate::OpenSubdiv_v3_6_1_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_ALL as _,
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum CreasingMethod {
    /// Apply regular, *Catmull-Clark* semi-sharp crease rules.
    ///
    /// * Note that this may give a jagged look when crease values vary along an
    ///   [edge loop](https://en.wikipedia.org/wiki/Edge_loop).
    Uniform = crate::OpenSubdiv_v3_6_1_Sdc_Options_CreasingMethod_CREASE_UNIFORM as _,
    /// Apply *Chaikin* semi-sharp crease rules.
    ///
    /// The *Chaikin Rule* is a variation of the semi-sharp creasing method
    /// that attempts to improve the appearance of creases along a sequence of
    /// connected edges when the sharpness values differ. This choice modifies
    /// the subdivision of sharpness values using Chaikin's curve subdivision
    /// algorithm to consider all sharpness values of edges around a common
    /// vertex when determining the sharpness of child edges.
    Chaikin = crate::OpenSubdiv_v3_6_1_Sdc_Options_CreasingMethod_CREASE_CHAIKIN as _,
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum TriangleSubdivision {
    /// Default *Catmull-Clark* scheme weights at triangles.
    CatmullClark = crate::OpenSubdiv_v3_6_1_Sdc_Options_TriangleSubdivision_TRI_SUB_CATMARK as _,
    /// Modifies the subdivision behavior at triangular faces to improve the
    /// undesirable surface artefacts that often result in such areas.
    Smooth = crate::OpenSubdiv_v3_6_1_Sdc_Options_TriangleSubdivision_TRI_SUB_SMOOTH as _,
}

pub type UniformRefinementOptions = crate::OpenSubdiv_v3_6_1_Far_TopologyRefiner_UniformOptions;
pub type AdaptiveRefinementOptions = crate::OpenSubdiv_v3_6_1_Far_TopologyRefiner_AdaptiveOptions;
pub type TopologyRefinerFactoryOptions =
    crate::OpenSubdiv_v3_6_1_Far_TopologyRefinerFactory_Options;
pub type ConstIndexArray = crate::OpenSubdiv_v3_6_1_Far_ConstIndexArray;
pub type TopologyRefiner = crate::OpenSubdiv_v3_6_1_Far_TopologyRefiner;
pub type TopologyRefinerPtr = *mut TopologyRefiner;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn TopologyRefinerFactory_TopologyDescriptor_Create(
        descriptor: *const crate::OpenSubdiv_v3_6_1_Far_TopologyDescriptor,
        options: TopologyRefinerFactoryOptions,
    ) -> TopologyRefinerPtr;
    /// \brief Returns true if uniform refinement has been applied
    pub fn TopologyRefiner_GetNumLevels(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns the maximum vertex valence in all levels
    pub fn TopologyRefiner_GetMaxValence(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns true if faces have been tagged as holes
    pub fn TopologyRefiner_GetNumVerticesTotal(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns the total number of edges in all levels
    pub fn TopologyRefiner_GetNumEdgesTotal(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns the total number of faces in all levels
    pub fn TopologyRefiner_GetNumFacesTotal(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns the total number of face vertices in all levels
    pub fn TopologyRefiner_GetNumFaceVerticesTotal(refiner: TopologyRefinerPtr) -> u32;
    /// \brief Returns a handle to access data specific to a particular level
    pub fn TopologyRefiner_GetLevel(refiner: TopologyRefinerPtr, level: i32) -> TopologyLevelPtr;
}
