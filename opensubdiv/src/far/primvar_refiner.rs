//! Primitive variable (primvar) refinement.
//!
//! [`PrimvarRefiner`] supports refinement of arbitrary primvar data at the
//! locations of topological vertices. A `PrimvarRefiner` accesses topology
//! data directly from a [`TopologyRefiner`].
//!
//! Different methods are provided to support three different classes of
//! primvar interpolation. These methods may be used to refine primvar data
//! to a specified refinement level or copy it down.
//!
//! * [`interpolate()`](PrimvarRefiner::interpolate()) – Interpolate using
//!   vertex weights.
//! * [`interpolate_varying()`](PrimvarRefiner::interpolate_varying()) –
//!   Interpolate using linear weights
//! * [`interpolate_face_varying()`](PrimvarRefiner::interpolate_face_varying())
//!   – Interpolate using face-varying weights.
//! * [`interpolate_face_uniform()`](PrimvarRefiner::interpolate_face_uniform())
//!   – Copy data down.
//!
//! Additional methods allow primvar data to be interpolated to the final
//! limit surface including the calculation of first derivative tangents.
//!
//! * `limit()` – Interpolate to the limit surface using vertex weights
//! * `limit_derive()` – Interpolate including first derivatives to the limit
//!   surface using vertex weights
//! * `limit_face_varying()` – Interpolate to the limit surface using
//!   face-varying weights.
//!
//!`PrimarRefiner` provides a straightforward interface for refining
//! primvar data, but depending on the application use case, it can be more
//! efficient to create and use a
//! [`StencilTable`](crate::far::stencil_table::StencilTable), or `PatchTable`,
//! to refine primvar data.
use opensubdiv_sys as sys;
use std::convert::TryInto;

use super::TopologyRefiner;

/// Applies refinement operations to generic primvar data.
pub struct PrimvarRefiner<'a> {
    ptr: sys::far::PrimvarRefinerPtr,
    topology_refiner: &'a TopologyRefiner,
}

impl<'a> PrimvarRefiner<'a> {
    /// Create a new primvar refiner.
    pub fn new(topology_refiner: &TopologyRefiner) -> PrimvarRefiner {
        unsafe {
            let ptr = sys::far::PrimvarRefiner_create(topology_refiner.0);
            if ptr.is_null() {
                panic!("PrimvarRefiner_create() returned null");
            }
            PrimvarRefiner {
                ptr,
                topology_refiner,
            }
        }
    }

    /// Apply vertex interpolation weights to a flat primvar buffer for a single
    /// level of refinement.
    ///
    /// Returns a flat a [`Vec`] of interpolated values or [`None`] if the
    /// `refinement_level` exceeds the
    /// [`max_level()`](TopologyRefiner::max_level())
    /// of the [`TopologyRefiner`] fed to [`PrimvarRefiner::new()`].
    pub fn interpolate(
        &self,
        refinement_level: u32,
        tuple_len: u32,
        source: &[f32],
    ) -> Option<Vec<f32>> {
        match self.topology_refiner.level(refinement_level) {
            Some(refiner_level) => {
                let dest_len =
                    (tuple_len * refiner_level.vertices_len()) as usize;
                let mut dest = Vec::<f32>::with_capacity(dest_len);
                unsafe {
                    dest.set_len(dest_len);
                    sys::far::PrimvarRefiner_Interpolate(
                        self.ptr,
                        tuple_len.try_into().unwrap(),
                        refinement_level.try_into().unwrap(),
                        source.as_ptr(),
                        dest.as_mut_ptr(),
                    );
                }
                Some(dest)
            }
            None => None,
        }
    }

    /// Apply face-varying interpolation weights to a primvar buffer associated
    /// with a particular face-varying channel.
    ///
    /// Unlike vertex and varying primvar buffers, there is not a 1-to-1
    /// correspondence between vertices and face-varying values – typically
    /// there are more face-varying values than vertices. Each face-varying
    /// channel is also independent in how its values relate to the vertices.
    ///
    /// Returns a flat a [`Vec`] of interpolated values or [`None`] if the
    /// `refinement_level` exceeds the
    /// [`max_level()`](TopologyRefiner::max_level())
    /// of the [`TopologyRefiner`] fed to [`PrimvarRefiner::new()`].
    pub fn interpolate_face_varying(
        &self,
        refinement_level: u32,
        tuple_len: u32,
        source: &[f32],
    ) -> Option<Vec<f32>> {
        match self.topology_refiner.level(refinement_level) {
            Some(refiner_level) => {
                let dest_len =
                    (tuple_len * refiner_level.vertices_len()) as usize;
                let mut dest = Vec::<f32>::with_capacity(dest_len);
                unsafe {
                    dest.set_len(dest_len);
                    sys::far::PrimvarRefiner_InterpolateFaceVarying(
                        self.ptr,
                        tuple_len.try_into().unwrap(),
                        refinement_level.try_into().unwrap(),
                        source.as_ptr(),
                        dest.as_mut_ptr(),
                    );
                }
                Some(dest)
            }
            None => None,
        }
    }

    /// Refine uniform (per-face) primvar data between levels.
    ///
    /// Data is simply copied from a parent face to its child faces and does not
    /// involve any weighting. Setting the source primvar data for the base
    /// level to be the index of each face allows the propagation of the base
    /// face to primvar data for child faces in all levels.
    ///
    /// Returns a flat a [`Vec`] of interpolated values or [`None`] if the
    /// `refinement_level` exceeds the
    /// [`max_level()`](TopologyRefiner::max_level())
    /// of the [`TopologyRefiner`] fed to [`PrimvarRefiner::new()`].
    pub fn interpolate_face_uniform(
        &self,
        refinement_level: u32,
        tuple_len: u32,
        source: &[f32],
    ) -> Option<Vec<f32>> {
        match self.topology_refiner.level(refinement_level) {
            Some(refiner_level) => {
                let dest_len =
                    (tuple_len * refiner_level.vertices_len()) as usize;
                let mut dest = Vec::<f32>::with_capacity(dest_len);
                unsafe {
                    dest.set_len(dest_len);
                    sys::far::PrimvarRefiner_InterpolateFaceUniform(
                        self.ptr,
                        tuple_len.try_into().unwrap(),
                        refinement_level.try_into().unwrap(),
                        source.as_ptr(),
                        dest.as_mut_ptr(),
                    );
                }
                Some(dest)
            }
            None => None,
        }
    }

    /// Apply only varying interpolation weights to a primvar buffer for a
    /// single level of refinement.
    ///
    /// This method can useful if the varying primvar data does not need to be
    /// re-computed over time.
    ///
    /// Returns a flat a [`Vec`] of interpolated values or [`None`] if the
    /// `refinement_level` exceeds the
    /// [`max_level()`](TopologyRefiner::max_level())
    /// of the [`TopologyRefiner`] fed to [`PrimvarRefiner::new()`].
    pub fn interpolate_varying(
        &self,
        refinement_level: u32,
        tuple_len: u32,
        source: &[f32],
    ) -> Option<Vec<f32>> {
        match self.topology_refiner.level(refinement_level) {
            Some(refiner_level) => {
                let dest_len =
                    (tuple_len * refiner_level.vertices_len()) as usize;
                let mut dest = Vec::<f32>::with_capacity(dest_len);
                unsafe {
                    dest.set_len(dest_len);
                    sys::far::PrimvarRefiner_InterpolateVarying(
                        self.ptr,
                        tuple_len.try_into().unwrap(),
                        refinement_level.try_into().unwrap(),
                        source.as_ptr(),
                        dest.as_mut_ptr(),
                    );
                }
                Some(dest)
            }
            None => None,
        }
    }
}

impl<'a> Drop for PrimvarRefiner<'a> {
    fn drop(&mut self) {
        unsafe { sys::far::PrimvarRefiner_destroy(self.ptr) };
    }
}
