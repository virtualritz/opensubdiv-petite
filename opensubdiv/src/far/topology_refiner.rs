use opensubdiv_sys as sys;
use std::convert::TryInto;

use crate::far::TopologyDescriptor;
use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

pub struct TopologyRefiner(
    pub(crate) sys::topology_refiner::TopologyRefinerPtr,
);

impl TopologyRefiner {
    pub fn new(
        descriptor: TopologyDescriptor,
        options: Options,
    ) -> Result<Self> {
        let ptr = unsafe {
            sys::TopologyRefinerFactory_TopologyDescriptor_Create(
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
            let mut options: sys::far::topology_refiner::Options =
                std::mem::zeroed();

            options.schemeType = (*self.0)._subdivType;
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
    pub fn len_levels(&self) -> u32 {
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
    pub fn len_vertices_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumVerticesTotal(self.0) as _ }
    }

    /// Returns the total number of edges in all levels.
    #[inline]
    pub fn len_edges_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumEdgesTotal(self.0) as _ }
    }

    /// Returns the total number of faces in all levels.
    #[inline]
    pub fn len_faces_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumFacesTotal(self.0) as _ }
    }

    /// Returns the total number of face vertices in all levels.
    #[inline]
    pub fn len_face_vertices_total(&self) -> u32 {
        unsafe {
            sys::far::TopologyRefiner_GetNumFaceVerticesTotal(self.0) as _
        }
    }

    /// Returns the highest level of refinement.
    #[inline]
    pub fn max_level(&self) -> u32 {
        unsafe { (*self.0)._maxLevel() as _ }
    }

    /// Returns a handle to access data specific to a particular refinement
    /// level.
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

    /// Refine the topology uniformly.
    ///
    /// This method applies uniform refinement to the level specified in the
    /// given [`UniformRefinementOptions`]s.
    ///
    /// Note the impact of the `UniformRefinementOptions` to generate full
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
    /// Refine the topology adaptively.
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

pub use sys::far::topology_refiner::{
    BoundaryInterpolation, CreasingMethod, FaceVaryingLinearInterpolation,
    Scheme, TriangleSubdivision,
};

use super::topology_level::TopologyLevel;

#[derive(Copy, Clone, Debug)]
pub struct Options(sys::far::topology_refiner::Options);

impl Options {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn validate_full_topology(
        &mut self,
        validate_full_topology: bool,
    ) -> &mut Self {
        self.0.set_validateFullTopology(validate_full_topology as _);
        self
    }

    #[inline]
    pub fn scheme(&mut self, scheme: Scheme) -> &mut Self {
        self.0.schemeType = scheme as _;
        self
    }

    #[inline]
    pub fn boundary_interpolation(
        &mut self,
        boundary_interpolation: BoundaryInterpolation,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__vtxBoundInterp(boundary_interpolation as _);
        self
    }

    #[inline]
    pub fn creasing_method(
        &mut self,
        creasing_method: CreasingMethod,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__creasingMethod(creasing_method as _);
        self
    }

    #[inline]
    pub fn triangle_subdivision(
        &mut self,
        triangle_subdivision: TriangleSubdivision,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__triangleSub(triangle_subdivision as _);
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        let mut sdc_options: sys::sdc::Options = unsafe { std::mem::zeroed() };
        sdc_options._bitfield_1 = sys::sdc::Options::new_bitfield_1(
            BoundaryInterpolation::None as _,
            FaceVaryingLinearInterpolation::All as _,
            CreasingMethod::Uniform as _,
            TriangleSubdivision::CatmullClark as _,
        );

        let mut options: sys::far::topology_refiner::Options =
            unsafe { std::mem::zeroed() };
        options.schemeType = Scheme::CatmullClark as _;
        options.schemeOptions = sdc_options;

        Self(options)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UniformRefinementOptions(
    sys::far::topology_refiner::UniformRefinementOptions,
);

impl UniformRefinementOptions {
    pub fn new(
        refinement_level: u32,
        order_vertices: bool,
        full_topology: bool,
    ) -> Self {
        let mut options: sys::far::topology_refiner::UniformRefinementOptions =
            unsafe { std::mem::zeroed() };

        options._bitfield_1 =
            sys::far::topology_refiner::UniformRefinementOptions::new_bitfield_1(
                refinement_level,
                order_vertices as _,
                full_topology as _,
            );

        Self(options)
    }

    pub fn refinement_level(&mut self, refinement_level: u32) -> &mut Self {
        self.0.set_refinementLevel(refinement_level);
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
}

impl Default for UniformRefinementOptions {
    fn default() -> Self {
        let mut options: sys::far::topology_refiner::UniformRefinementOptions =
            unsafe { std::mem::zeroed() };

        options._bitfield_1 =
            sys::far::topology_refiner::UniformRefinementOptions::new_bitfield_1(
                4,
                true as _,
                true as _,
            );

        Self(options)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AdaptiveRefinementOptions(
    sys::far::topology_refiner::AdaptiveRefinementOptions,
);

impl AdaptiveRefinementOptions {
    pub fn new(
        isolation_level: u32,
        secondary_level: u32,
        single_crease_patch: bool,
        infintely_sharp_patch: bool,
        consider_face_varying_channels: bool,
        order_vertices_from_faces_first: bool,
    ) -> Self {
        let mut options: sys::far::topology_refiner::AdaptiveRefinementOptions =
            unsafe { std::mem::zeroed() };

        options._bitfield_1 =
        sys::far::topology_refiner::AdaptiveRefinementOptions::new_bitfield_1(
            isolation_level as _,
            secondary_level as _,
            single_crease_patch as _,
            infintely_sharp_patch as _,
            consider_face_varying_channels as _,
            order_vertices_from_faces_first as _,
        );
        Self(options)
    }

    pub fn isolation_level(&mut self, isolation_level: u32) -> &mut Self {
        self.0.set_isolationLevel(isolation_level);
        self
    }

    pub fn secondary_level(&mut self, secondary_level: u32) -> &mut Self {
        self.0.set_secondaryLevel(secondary_level);
        self
    }

    pub fn use_single_crease_patch(
        &mut self,
        single_crease_patch: bool,
    ) -> &mut Self {
        self.0.set_useSingleCreasePatch(single_crease_patch as _);
        self
    }

    pub fn use_infintely_sharp_patch(
        &mut self,
        infintely_sharp_patch: bool,
    ) -> &mut Self {
        self.0.set_useInfSharpPatch(infintely_sharp_patch as _);
        self
    }

    pub fn consider_face_varying_channels(
        &mut self,
        consider_face_varying_channels: bool,
    ) -> &mut Self {
        self.0
            .set_considerFVarChannels(consider_face_varying_channels as _);
        self
    }

    pub fn order_vertices_from_faces_first(
        &mut self,
        order_vertices_from_faces_first: bool,
    ) -> &mut Self {
        self.0.set_orderVerticesFromFacesFirst(
            order_vertices_from_faces_first as _,
        );
        self
    }
}

impl Default for AdaptiveRefinementOptions {
    fn default() -> Self {
        let mut options: sys::far::topology_refiner::AdaptiveRefinementOptions =
            unsafe { std::mem::zeroed() };

        options._bitfield_1 =
        sys::far::topology_refiner::AdaptiveRefinementOptions::new_bitfield_1(
                4,
             15,
             false as _,
            false as _,
             false as _,
             false as _,
        );

        Self(options)
    }
}
