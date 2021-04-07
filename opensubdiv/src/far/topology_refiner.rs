//! Topology refinement.
//!
//! [`TopologyRefiner`] is the building block for many other useful structs in
//! `far`. It performs refinement of an arbitrary mesh and provides access to
//! the refined mesh topology.
//!
//! It can be used for primvar refinement directly
//! through a [`PrimvarRefiner`](super::primvar_refiner::PrimvarRefiner).  Or
//! indirectly by being used to create a
//! [`StencilTable`](super::stencil_table::StencilTable), or a `PatchTable`,
//! etc.
//!
//! `TopologyRefiner` provides these refinement methods:
//! * [`refine_uniform()`](TopologyRefiner::refine_uniform()) – Does uniform
//!   refinenment as specified in the [`UniformRefinementOptions`].
//! * [`refine_adaptive()`](TopologyRefiner::refine_adaptive()) – Does adaptive
//!   refinement as specified in the [`AdaptiveRefinementOptions`].
//!
//! The result can be accessed via:
//! * [`level()`](TopologyRefiner::level()) – Gives access to the refined
//!   topology at through a [`TopologyLevel`] instance.
use opensubdiv_sys as sys;
use std::convert::TryInto;

use crate::far::TopologyDescriptor;
use crate::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Stores topology data for a specified set of refinement options.
pub struct TopologyRefiner(
    pub(crate) sys::topology_refiner::TopologyRefinerPtr,
);

impl TopologyRefiner {
    /// Create a new topology refiner.
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
    pub fn refinement_levels(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumLevels(self.0) as _ }
    }

    /// Returns the maximum vertex valence in all levels
    #[inline]
    pub fn max_valence(&self) -> u32 {
        unsafe { (*self.0)._maxValence as _ }
    }

    /// Returns `true` if faces have been tagged as holes.
    #[inline]
    pub fn has_holes(&self) -> bool {
        unsafe { (*self.0)._hasHoles() != 0 }
    }

    /// Returns the total number of vertices in all levels.
    #[inline]
    pub fn vertices_total_len(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumVerticesTotal(self.0) as _ }
    }

    /// Returns the total number of edges in all levels.
    #[inline]
    pub fn edges_total_len(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumEdgesTotal(self.0) as _ }
    }

    /// Returns the total number of faces in all levels.
    #[inline]
    pub fn faces_total_len(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumFacesTotal(self.0) as _ }
    }

    /// Returns the total number of face vertices in all levels.
    #[inline]
    pub fn face_vertices_total_len(&self) -> u32 {
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
    /// * `options` - Options controlling uniform refinement.
    #[inline]
    pub fn refine_uniform(&mut self, options: UniformRefinementOptions) {
        unsafe {
            (*self.0).RefineUniform(options.0);
        }
    }

    /// Refine the topology adaptively.
    ///
    /// This method applies uniform refinement to the level specified in the
    /// given [`AdaptiveRefinementOptions`]s.
    ///
    /// * `options` - Options controlling uniform refinement.
    #[inline]
    pub fn refine_adaptive(
        &mut self,
        options: AdaptiveRefinementOptions,
        selected_faces: &[u32],
    ) {
        let tmp = sys::topology_refiner::ConstIndexArray {
            _begin: selected_faces.as_ptr() as _,
            _size: selected_faces.len().try_into().unwrap(),
            _phantom_0: std::marker::PhantomData,
        };
        unsafe {
            (*self.0).RefineAdaptive(options.0, tmp);
        }
    }

    /// Unrefine the topology, keeping only the base level.
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

/// All supported options applying to a subdivision scheme.
///
/// This  contains all supported options that can be applied to a subdivision
/// [`Scheme`] to affect the shape of the limit surface. These differ
/// from approximations that may be applied at a higher level.  I.e. options to
/// limit the level of feature adaptive subdivision, options to ignore
/// fractional creasing, or creasing entirely, etc.  These options define the
/// shape of a particular limit surface, including the ‘shape of primitive
/// variable data associated with it.
///
/// The intent is that these sets of options be defined at a high level and
/// propagated into the lowest-level computation in support of each subdivision
/// scheme.  Ideally it remains a set of bit-fields (essentially an int) and so
/// remains light weight and easily passed around by value.
#[derive(Copy, Clone, Debug)]
pub struct Options(sys::far::topology_refiner::Options);

impl Options {
    /// Creates new options.
    #[inline]
    pub fn new(
        scheme: Scheme,
        boundary_interpolation: BoundaryInterpolation,
        face_varying_linear_interpolation: FaceVaryingLinearInterpolation,
        creasing_method: CreasingMethod,
        triangle_subdivision: TriangleSubdivision,
    ) -> Self {
        let mut sdc_options: sys::sdc::Options = unsafe { std::mem::zeroed() };
        sdc_options._bitfield_1 = sys::sdc::Options::new_bitfield_1(
            boundary_interpolation as _,
            face_varying_linear_interpolation as _,
            creasing_method as _,
            triangle_subdivision as _,
        );

        let mut options: sys::far::topology_refiner::Options =
            unsafe { std::mem::zeroed() };
        options.schemeType = scheme as _;
        options.schemeOptions = sdc_options;

        Self(options)
    }

    /// Sets the subdivision [`Scheme`] to use.
    #[inline]
    pub fn scheme(&mut self, scheme: Scheme) -> &mut Self {
        self.0.schemeType = scheme as _;
        self
    }

    /// Sets the vertex boundary interpolation rule.
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

    /// Sets the vertex boundary interpolation rule.
    #[inline]
    pub fn face_varying_linear_interpolation(
        &mut self,
        face_varying_linear_interpolation: FaceVaryingLinearInterpolation,
    ) -> &mut Self {
        self.0
            .schemeOptions
            .set__fvarLinInterp(boundary_interpolation as _);
        self
    }

    /// Set the edge crease rule.
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

    /// Set the triangle subdivision weights rule.
    ///
    /// Only applies to the [`Catmull-Clark`](Scheme::CatmullClark) scheme –
    /// ignored otherwise.
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

    /// Apply more extensive validation of the constructed topology – intended
    /// for *debugging*.
    #[inline]
    pub fn validate_full_topology(
        &mut self,
        validate_full_topology: bool,
    ) -> &mut Self {
        self.0.set_validateFullTopology(validate_full_topology as _);
        self
    }
}

impl Default for Options {
    /// Create options with the following defaults:
    ///
    /// | Property                            | Value
    /// | |----------------
    /// --|-----------------------------------------------------| | `scheme`
    /// | [`CatmullClark`](Scheme::CatmullClark)              |
    /// | `boundary_interpolation`            |
    /// [`None`](BoundaryInterpolation::None)               |
    /// | `face_varying_linear_interpolation` |
    /// [`All`](FaceVaryingLinearInterpolation::All)        |
    /// | `creasing_method`                   |
    /// [`Uniform`](CreasingMethod::Uniform)                |
    /// | `triangle_subdivision`              |
    /// [`CatmullClark`](TriangleSubdivision::CatmullClark) |
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

/// Uniform topology refinement options.
#[derive(Copy, Clone, Debug)]
pub struct UniformRefinementOptions(
    sys::far::topology_refiner::UniformRefinementOptions,
);

impl UniformRefinementOptions {
    /// Create new uniform refinement options.
    ///
    /// Use [`default()`](UniformRefinementOptions::default()) to create options
    /// with default values.
    pub fn new(
        refinement_level: u32,
        order_vertices_from_faces_first: bool,
        full_topology_in_last_level: bool,
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

    /// Number of refinement iterations.
    pub fn refinement_level(&mut self, refinement_level: u32) -> &mut Self {
        self.0.set_refinementLevel(refinement_level);
        self
    }

    /// Order child vertices from faces first instead of child vertices of
    /// vertices.
    pub fn order_vertices_from_faces_first(
        &mut self,
        order_vertices_from_faces_first: bool,
    ) -> &mut Self {
        self.0.set_orderVerticesFromFacesFirst(
            order_vertices_from_faces_first as _,
        );
        self
    }

    /// Skip topological relationships in the last level of refinement that are
    /// not needed for interpolation (keep false if using limit).
    pub fn full_topology_in_last_level(
        &mut self,
        full_topology_in_last_level: bool,
    ) -> &mut Self {
        self.0
            .set_fullTopologyInLastLevel(full_topology_in_last_level as _);
        self
    }
}

impl Default for UniformRefinementOptions {
    /// Create uniform refinement options with the following defaults:
    ///
    /// | Property                          | Value   |
    /// |----------------                 --|---------|
    /// | `refinement_level`                | `4`     |
    /// | `order_vertices_from_faces_first` | `true`  |
    /// | `full_topology_in_last_level`     | `true`  |
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

/// Adaptive topology refinement options.
#[derive(Copy, Clone, Debug)]
pub struct AdaptiveRefinementOptions(
    sys::far::topology_refiner::AdaptiveRefinementOptions,
);

impl AdaptiveRefinementOptions {
    /// Create new adaptive refinement options.
    ///
    /// Use [`default()`](AdaptiveRefinementOptions::default()) to create
    /// options with default values.
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

    /// Number of iterations applied to isolate extraordinary vertices and
    /// creases.
    pub fn isolation_level(&mut self, isolation_level: u32) -> &mut Self {
        self.0.set_isolationLevel(isolation_level);
        self
    }

    /// Shallower level to stop isolation of smooth irregular features.
    pub fn secondary_level(&mut self, secondary_level: u32) -> &mut Self {
        self.0.set_secondaryLevel(secondary_level);
        self
    }

    /// Use 'single-crease' patch and stop isolation where applicable.
    pub fn use_single_crease_patch(
        &mut self,
        single_crease_patch: bool,
    ) -> &mut Self {
        self.0.set_useSingleCreasePatch(single_crease_patch as _);
        self
    }

    /// Use infinitely sharp patches and stop isolation where applicable.
    pub fn use_infintely_sharp_patch(
        &mut self,
        infintely_sharp_patch: bool,
    ) -> &mut Self {
        self.0.set_useInfSharpPatch(infintely_sharp_patch as _);
        self
    }

    /// Inspect face-varying channels and isolate when irregular features
    /// present.
    pub fn consider_face_varying_channels(
        &mut self,
        consider_face_varying_channels: bool,
    ) -> &mut Self {
        self.0
            .set_considerFVarChannels(consider_face_varying_channels as _);
        self
    }

    /// Order child vertices from faces first instead of child vertices of
    /// vertices.
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
    /// Create adaptive refinement options with the following defaults:
    ///
    /// | Property                          | Value   |
    /// |-----------------------------------|---------|
    /// | `isolation_level`                 | `4`     |
    /// | `secondary_level`                 | `15`    |
    /// | `single_crease_patch`             | `false` |
    /// | `infintely_sharp_patch`           | `false` |
    /// | `consider_face_varying_channels`  | `false` |
    /// | `order_vertices_from_faces_first` | `false` |
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
