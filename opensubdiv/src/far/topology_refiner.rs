use opensubdiv_sys as sys;
use std::convert::TryInto;

pub use crate::sdc;

pub use sys::topology_refiner::{UniformOptions, UniformOptionsBuilder};

use super::topology_level::TopologyLevel;

pub fn uniform_options() -> UniformOptionsBuilder {
    UniformOptionsBuilder::new()
}

pub struct TopologyRefiner {
    pub(crate) ptr: sys::far::TopologyRefinerPtr,
}

impl TopologyRefiner {
    /// Returns the subdivision scheme.
    #[inline]
    pub fn scheme(&self) -> sdc::Scheme {
        unsafe { sys::far::TopologyRefiner_GetSchemeType(self.ptr) }
    }

    /// Returns the subdivision options.
    #[inline]
    pub fn options(&self) -> sdc::Options {
        unsafe { sys::far::TopologyRefiner_GetSchemeOptions(self.ptr) }
    }

    /// Returns true if uniform refinement has been applied.
    #[inline]
    pub fn is_uniform(&self) -> bool {
        unsafe { sys::far::TopologyRefiner_IsUniform(self.ptr) }
    }

    /// Returns the number of refinement levels.
    #[inline]
    pub fn num_levels(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetNumLevels(self.ptr) as _ }
    }

    /// Returns the maximum vertex valence in all levels
    #[inline]
    pub fn max_valence(&self) -> usize {
        unsafe { sys::far::TopologyRefiner_GetMaxValence(self.ptr) as _ }
    }

    /// Returns true if faces have been tagged as holes.
    #[inline]
    pub fn has_holes(&self) -> bool {
        unsafe { sys::far::TopologyRefiner_HasHoles(self.ptr) }
    }

    /// Returns the total number of vertices in all levels.
    #[inline]
    pub fn num_vertices_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumVerticesTotal(self.ptr) as _ }
    }

    /// Returns the total number of edges in all levels.
    #[inline]
    pub fn num_edges_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumEdgesTotal(self.ptr) as _ }
    }

    /// Returns the total number of faces in all levels.
    #[inline]
    pub fn num_faces_total(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetNumFacesTotal(self.ptr) as _ }
    }

    /// Returns the total number of face vertices in all levels.
    #[inline]
    pub fn num_face_vertices_total(&self) -> u32 {
        unsafe {
            sys::far::TopologyRefiner_GetNumFaceVerticesTotal(self.ptr) as _
        }
    }

    /// Returns the highest level of refinement.
    #[inline]
    pub fn max_level(&self) -> u32 {
        unsafe { sys::far::TopologyRefiner_GetMaxLevel(self.ptr) as _ }
    }

    /// Returns a handle to access data specific to a particular level.
    #[inline]
    pub fn level(&self, level: u32) -> Option<TopologyLevel> {
        if level > self.max_level() {
            None
        } else {
            let ptr = unsafe {
                sys::far::TopologyRefiner_GetLevel(
                    self.ptr,
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
    pub fn refine_uniform(&mut self, options: UniformOptions) {
        unsafe {
            sys::far::TopologyRefiner_RefineUniform(self.ptr, options);
        }
    }
}

impl Drop for TopologyRefiner {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::far::TopologyRefiner_destroy(self.ptr);
        }
    }
}
