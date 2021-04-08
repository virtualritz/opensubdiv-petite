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
        options: TopologyRefinerOptions,
    ) -> Result<Self> {
        let mut sdc_options: sys::sdc::Options = unsafe { std::mem::zeroed() };
        sdc_options._bitfield_1 = sys::sdc::Options::new_bitfield_1(
            options.boundary_interpolation as _,
            options.face_varying_linear_interpolation as _,
            options.creasing_method as _,
            options.triangle_subdivision as _,
        );

        let mut sys_options: sys::far::topology_refiner::Options =
            unsafe { std::mem::zeroed() };
        sys_options.schemeType = options.scheme as _;
        sys_options.schemeOptions = sdc_options;

        let ptr = unsafe {
            sys::TopologyRefinerFactory_TopologyDescriptor_Create(
                &descriptor.descriptor as _,
                sys_options,
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
    pub fn options(&self) -> TopologyRefinerOptions {
        let options = unsafe { &(*self.0)._subdivOptions };
        TopologyRefinerOptions {
            scheme: unsafe { (*self.0)._subdivType }.try_into().unwrap(),
            boundary_interpolation: options
                ._vtxBoundInterp()
                .try_into()
                .unwrap(),
            face_varying_linear_interpolation: options
                ._fvarLinInterp()
                .try_into()
                .unwrap(),
            creasing_method: options._creasingMethod().try_into().unwrap(),
            triangle_subdivision: options._triangleSub().try_into().unwrap(),
        }
    }

    /// Returns true if uniform refinement has been applied.
    #[inline]
    pub fn is_uniform(&self) -> bool {
        unsafe { (*self.0)._isUniform() != 0 }
    }

    /// Returns the number of refinement levels.
    #[inline]
    pub fn refinement_levels(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetNumLevels(self.0) as _ }
    }

    /// Returns the maximum vertex valence in all levels
    #[inline]
    pub fn max_valence(&self) -> usize {
        unsafe { (*self.0)._maxValence as _ }
    }

    /// Returns `true` if faces have been tagged as holes.
    #[inline]
    pub fn has_holes(&self) -> bool {
        unsafe { (*self.0)._hasHoles() != 0 }
    }

    /// Returns the total number of vertices in all levels.
    #[inline]
    pub fn vertices_total_len(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetNumVerticesTotal(self.0) as _ }
    }

    /// Returns the total number of edges in all levels.
    #[inline]
    pub fn edges_total_len(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetNumEdgesTotal(self.0) as _ }
    }

    /// Returns the total number of faces in all levels.
    #[inline]
    pub fn faces_total_len(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetNumFacesTotal(self.0) as _ }
    }

    /// Returns the total number of face vertices in all levels.
    #[inline]
    pub fn face_vertices_total_len(&self) -> usize {
        unsafe {
            sys::far::TopologyRefiner_GetNumFaceVerticesTotal(self.0) as _
        }
    }

    /// Returns the highest level of refinement.
    #[inline]
    pub fn max_level(&self) -> usize {
        unsafe { (*self.0)._maxLevel() as _ }
    }

    /// Returns a handle to access data specific to a particular refinement
    /// level.
    #[inline]
    pub fn level(&self, level: usize) -> Option<TopologyLevel> {
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
        let mut sys_options: sys::far::topology_refiner::UniformRefinementOptions =
            unsafe { std::mem::zeroed() };

        sys_options._bitfield_1 =
            sys::far::topology_refiner::UniformRefinementOptions::new_bitfield_1(
                options.refinement_level.try_into().unwrap(),
                options.order_vertices_from_faces_first as _,
                options.full_topology_in_last_level as _,
            );

        unsafe {
            (*self.0).RefineUniform(sys_options);
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
        let mut sys_options: sys::far::topology_refiner::AdaptiveRefinementOptions =
            unsafe { std::mem::zeroed() };

        sys_options._bitfield_1 =
        sys::far::topology_refiner::AdaptiveRefinementOptions::new_bitfield_1(
            options.isolation_level.try_into().unwrap(),
            options.secondary_level.try_into().unwrap(),
            options.single_crease_patch as _,
            options.infintely_sharp_patch as _,
            options.consider_face_varying_channels as _,
            options.order_vertices_from_faces_first as _,
        );

        let const_array = sys::topology_refiner::ConstIndexArray {
            _begin: selected_faces.as_ptr() as _,
            _size: selected_faces.len().try_into().unwrap(),
            _phantom_0: std::marker::PhantomData,
        };

        unsafe {
            (*self.0).RefineAdaptive(sys_options, const_array);
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
pub struct TopologyRefinerOptions {
    pub scheme: Scheme,
    pub boundary_interpolation: BoundaryInterpolation,
    pub face_varying_linear_interpolation: FaceVaryingLinearInterpolation,
    pub creasing_method: CreasingMethod,
    pub triangle_subdivision: TriangleSubdivision,
}

impl Default for TopologyRefinerOptions {
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
        Self {
            scheme: Scheme::CatmullClark,
            boundary_interpolation: BoundaryInterpolation::None,
            face_varying_linear_interpolation:
                FaceVaryingLinearInterpolation::All,
            creasing_method: CreasingMethod::Uniform,
            triangle_subdivision: TriangleSubdivision::CatmullClark,
        }
    }
}

/// Uniform topology refinement options.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformRefinementOptions {
    pub refinement_level: usize,
    pub order_vertices_from_faces_first: bool,
    pub full_topology_in_last_level: bool,
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
        Self {
            refinement_level: 4,
            order_vertices_from_faces_first: true,
            full_topology_in_last_level: true,
        }
    }
}

/// Adaptive topology refinement options.
#[derive(Copy, Clone, Debug)]
pub struct AdaptiveRefinementOptions {
    pub isolation_level: usize,
    pub secondary_level: usize,
    pub single_crease_patch: bool,
    pub infintely_sharp_patch: bool,
    pub consider_face_varying_channels: bool,
    pub order_vertices_from_faces_first: bool,
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
        Self {
            isolation_level: 4,
            secondary_level: 15,
            single_crease_patch: false,
            infintely_sharp_patch: false,
            consider_face_varying_channels: false,
            order_vertices_from_faces_first: false,
        }
    }
}
