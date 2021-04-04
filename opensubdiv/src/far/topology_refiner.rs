use num_enum::TryFromPrimitive;
use opensubdiv_sys as sys;
use std::convert::TryInto;

use crate::far::TopologyDescriptor;
pub use crate::sdc;
use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

//pub use sys::topology_refiner::{UniformOptions, UniformOptionsBuilder};

use super::topology_level::TopologyLevel;

/*
pub fn uniform_options() -> UniformOptionsBuilder {
    UniformOptionsBuilder::new()
}*/

#[derive(Copy, Clone, Debug)]
pub struct UniformRefinementOptions(
    sys::OpenSubdiv_v3_4_4_Far_TopologyRefiner_UniformOptions,
);

impl UniformRefinementOptions {
    pub fn new(level: u32, order_vertices: bool, full_topology: bool) -> Self {
        let mut options: sys::OpenSubdiv_v3_4_4_Far_TopologyRefiner_UniformOptions = unsafe{ std::mem::zeroed() };
        options.set_refinementLevel(level.try_into().unwrap());
        options.set_orderVerticesFromFacesFirst(order_vertices as _);
        options.set_fullTopologyInLastLevel(full_topology as _);
        Self(options)
    }

    pub fn refinement_level(&mut self, level: u32) -> &mut Self {
        self.0.set_refinementLevel(level.try_into().unwrap());
        self
    }

    pub fn order_vertices_from_faces_first(
        &mut self,
        order_vertices: bool,
    ) -> &mut Self {
        self.0.set_orderVerticesFromFacesFirst(order_vertices as _);
        self
    }

    pub fn full_topology_in_last_level(
        &mut self,
        full_topology: bool,
    ) -> &mut Self {
        self.0.set_fullTopologyInLastLevel(full_topology as _);
        self
    }

    pub fn finalize(self) -> Self {
        self
    }
}

impl Default for UniformRefinementOptions {
    fn default() -> Self {
        let mut options: sys::OpenSubdiv_v3_4_4_Far_TopologyRefiner_UniformOptions = unsafe{ std::mem::zeroed() };
        options.set_refinementLevel(4);
        options.set_orderVerticesFromFacesFirst(true as _);
        options.set_fullTopologyInLastLevel(true as _);
        Self(options)
    }
}

#[repr(u32)]
#[derive(TryFromPrimitive, Copy, Clone, Debug)]
pub enum Scheme {
    Bilinear = sys::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_BILINEAR,
    CatmullClark = sys::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_CATMARK,
    Loop = sys::OpenSubdiv_v3_4_4_Sdc_SchemeType_SCHEME_LOOP,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum BoundaryInterpolation {
    None = sys::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_NONE,
    EdgeOnly = sys::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_ONLY,
    EdgeAndCorner = sys::OpenSubdiv_v3_4_4_Sdc_Options_VtxBoundaryInterpolation_VTX_BOUNDARY_EDGE_AND_CORNER,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum FaceVaryingLinearInterpolation {
    None = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_NONE,
    CornersOnly = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_ONLY,
    CornersPlusOne = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS1,
    CornersPlusTwo = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_CORNERS_PLUS2,
    Boundaries = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_BOUNDARIES,
    All = sys::OpenSubdiv_v3_4_4_Sdc_Options_FVarLinearInterpolation_FVAR_LINEAR_ALL,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CreasingMethod {
    Uniform = sys::OpenSubdiv_v3_4_4_Sdc_Options_CreasingMethod_CREASE_UNIFORM,
    Chaikin = sys::OpenSubdiv_v3_4_4_Sdc_Options_CreasingMethod_CREASE_CHAIKIN,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TriangleSubdivision {
    CatmullClark =
        sys::OpenSubdiv_v3_4_4_Sdc_Options_TriangleSubdivision_TRI_SUB_CATMARK,
    Smooth =
        sys::OpenSubdiv_v3_4_4_Sdc_Options_TriangleSubdivision_TRI_SUB_SMOOTH,
}

#[derive(Copy, Clone, Debug)]
pub struct Options(sys::OpenSubdiv_v3_4_4_Far_TopologyRefinerFactory_Options);

impl Options {
    #[inline]
    pub fn new() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }

    #[inline]
    pub fn with_validate_full_topology(&mut self, validate: bool) -> &mut Self {
        self.0.set_validateFullTopology(validate as _);
        self
    }

    #[inline]
    pub fn with_scheme(&mut self, scheme: Scheme) -> &mut Self {
        self.0.schemeType = scheme as _;
        self
    }

    #[inline]
    pub fn with_boundary_interpolation(
        &mut self,
        boundary_interpolation: BoundaryInterpolation,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__vtxBoundInterp(boundary_interpolation as _);
        self
    }

    #[inline]
    pub fn with_creasing_method(
        &mut self,
        creasing_method: CreasingMethod,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__creasingMethod(creasing_method as _);
        self
    }

    #[inline]
    pub fn with_triangle_subdivision(
        &mut self,
        triangle_subdivision: TriangleSubdivision,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__triangleSub(triangle_subdivision as _);
        self
    }

    #[inline]
    pub fn finalize(self) -> Self {
        self
    }
}

pub struct TopologyRefiner(
    pub(crate) *mut sys::OpenSubdiv_v3_4_4_Far_TopologyRefiner,
);

impl TopologyRefiner {
    pub fn new(
        descriptor: TopologyDescriptor,
        options: Options,
    ) -> Result<Self> {
        let ptr = unsafe {
            sys::far::TopologyRefinerFactory_TopologyDescriptor_Create(
                &descriptor.descriptor as _,
                options.0,
            )
        };

        if ptr.is_null() {
            Err(Error::CreateTopologyRefinerFailed)
        } else {
            Ok(Self(ptr))
        }
    }

    /// Returns the subdivision options.
    #[inline]
    pub fn options(&self) -> Options {
        unsafe {
            let mut options: sys::OpenSubdiv_v3_4_4_Far_TopologyRefinerFactory_Options = std::mem::zeroed();

            options.schemeType = (*self.0)._subdivType.try_into().unwrap();
            options.schemeOptions = (*self.0)._subdivOptions;

            Options(options)
        }
    }

    /// Returns true if uniform refinement has been applied.
    #[inline]
    pub fn is_uniform(&self) -> bool {
        unsafe { (*self.0)._isUniform() != 0 }
    }

    /// Returns the number of refinement levels.
    #[inline]
    pub fn num_levels(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumLevels(self.0) as _ }
    }

    /// Returns the maximum vertex valence in all levels
    #[inline]
    pub fn max_valence(&self) -> u32 {
        unsafe { (*self.0)._maxValence as _ }
    }

    /// Returns true if faces have been tagged as holes.
    #[inline]
    pub fn has_holes(&self) -> bool {
        unsafe { (*self.0)._hasHoles() != 0 }
    }

    /// Returns the total number of vertices in all levels.
    #[inline]
    pub fn num_vertices_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumVerticesTotal(self.0) as _ }
    }

    /// Returns the total number of edges in all levels.
    #[inline]
    pub fn num_edges_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumEdgesTotal(self.0) as _ }
    }

    /// Returns the total number of faces in all levels.
    #[inline]
    pub fn num_faces_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumFacesTotal(self.0) as _ }
    }

    /// Returns the total number of face vertices in all levels.
    #[inline]
    pub fn num_face_vertices_total(&self) -> u32 {
        unsafe {
            sys::far::TopologyRefiner_GetNumFaceVerticesTotal(self.0) as _
        }
    }

    /// Returns the highest level of refinement.
    #[inline]
    pub fn max_level(&self) -> u32 {
        unsafe { (*self.0)._maxLevel() as _ }
    }

    /// Returns a handle to access data specific to a particular level.
    #[inline]
    pub fn level(&self, level: u32) -> Option<TopologyLevel> {
        if level > self.max_level() {
            None
        } else {
            let ptr = unsafe {
                sys::far::TopologyRefiner_GetLevel(
                    self.0,
                    level.try_into().unwrap(),
                )
            };
            if ptr.is_null() {
                None
            } else {
                Some(TopologyLevel {
                    ptr,
                    refiner: std::marker::PhantomData,
                })
            }
        }
    }

    /// Refine the topology uniformly
    ///
    /// This method applies uniform refinement to the level specified in the
    /// given [`UniformOption`]s.
    ///
    /// Note the impact of the [`UniformOption`] to generate full
    /// TopologyInLastLevel and be sure it is assigned to satisfy the needs
    /// of the resulting refinement.
    ///
    /// * `options` - Options controlling uniform refinement.
    #[inline]
    pub fn refine_uniform(&mut self, options: UniformRefinementOptions) {
        unsafe {
            (*self.0).RefineUniform(options.0);
        }
    }

    /*
    #[inline]
    pub fn refine_adaptive(&mut self, options: AdaptiveOptions) {
        unsafe {
            (*self.0).RefineAdaptive(self.0, options);
        }
    }*/

    #[inline]
    pub fn unrefine(&mut self) {
        unsafe {
            (*self.0).Unrefine();
        }
    }
}

impl Drop for TopologyRefiner {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (*self.0).destruct();
        }
    }
}
