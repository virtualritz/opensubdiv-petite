//! Thin safe wrapper for OpenSubdiv BFR surfaces (per-face limit patches).

use crate::{Error, Index};
use opensubdiv_petite_sys as sys;

/// Errors from BFR surface handling.
#[derive(Debug, Clone)]
pub enum BfrError {
    /// Failed to create or initialize a BFR surface.
    InitializationFailed,
    /// Surface is invalid.
    InvalidSurface,
    /// Control point buffer too small.
    BufferTooSmall,
    /// Unsupported patch point count for regular export.
    UnsupportedPatchPointCount(usize),
}

/// Wrapper around `Bfr::RefinerSurfaceFactory` (float).
pub struct SurfaceFactory {
    ptr: *mut sys::bfr::surface_factory::Bfr_SurfaceFactory_f,
}

unsafe impl Send for SurfaceFactory {}
unsafe impl Sync for SurfaceFactory {}

impl SurfaceFactory {
    /// Create a factory from a `TopologyRefiner` with approximation levels for
    /// smooth and sharp features.
    pub fn new(
        refiner: &crate::far::TopologyRefiner,
        approx_smooth: i32,
        approx_sharp: i32,
    ) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::bfr::surface_factory::Bfr_SurfaceFactory_Create(
                refiner.as_ptr(),
                approx_smooth,
                approx_sharp,
            );

            if ptr.is_null() {
                Err(Error::PatchTableCreation)
            } else {
                Ok(Self { ptr })
            }
        }
    }

    /// Initialize a vertex surface for the given base face.
    pub fn init_vertex_surface(&self, face_index: Index) -> Result<Surface, BfrError> {
        unsafe {
            let surface = sys::bfr::surface_factory::Bfr_Surface_Create();
            if surface.is_null() {
                return Err(BfrError::InitializationFailed);
            }

            let ok = sys::bfr::surface_factory::Bfr_SurfaceFactory_InitVertexSurface(
                self.ptr,
                face_index.0 as i32,
                surface,
            );

            if !ok {
                sys::bfr::surface_factory::Bfr_Surface_Destroy(surface);
                return Err(BfrError::InitializationFailed);
            }

            Ok(Surface { ptr: surface })
        }
    }
}

impl Drop for SurfaceFactory {
    fn drop(&mut self) {
        unsafe {
            sys::bfr::surface_factory::Bfr_SurfaceFactory_Destroy(self.ptr);
        }
    }
}

/// Wrapper around `Bfr::Surface<float>`.
pub struct Surface {
    ptr: *mut sys::bfr::surface_factory::Bfr_Surface_f,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    /// Check validity.
    pub fn is_valid(&self) -> bool {
        unsafe { sys::bfr::surface_factory::Bfr_Surface_IsValid(self.ptr) }
    }

    /// Returns true if the surface is a single regular patch.
    pub fn is_regular(&self) -> bool {
        unsafe { sys::bfr::surface_factory::Bfr_Surface_IsRegular(self.ptr) }
    }

    /// Number of control points affecting this surface.
    pub fn control_point_count(&self) -> usize {
        unsafe { sys::bfr::surface_factory::Bfr_Surface_GetNumControlPoints(self.ptr) as usize }
    }

    /// Get control point indices into the mesh vertex array.
    pub fn control_point_indices(&self) -> Result<Vec<Index>, BfrError> {
        let count = self.control_point_count();
        let mut buf = vec![0i32; count];
        let written = unsafe {
            sys::bfr::surface_factory::Bfr_Surface_GetControlPointIndices(
                self.ptr,
                buf.as_mut_ptr(),
                count as i32,
            )
        };

        if written as usize != count {
            return Err(BfrError::BufferTooSmall);
        }

        Ok(buf.iter().map(|&v| Index::from(v as u32)).collect())
    }

    /// Evaluate position at (u,v) using mesh points (stride = 3).
    pub fn evaluate_position(
        &self,
        u: f32,
        v: f32,
        mesh_points: &[[f32; 3]],
    ) -> Result<[f32; 3], BfrError> {
        if !self.is_valid() {
            return Err(BfrError::InvalidSurface);
        }

        let mut out = [0.0f32; 3];
        let ok = unsafe {
            sys::bfr::surface_factory::Bfr_Surface_EvaluatePosition(
                self.ptr,
                u,
                v,
                mesh_points.as_ptr() as *const f32,
                3,
                out.as_mut_ptr(),
            )
        };

        if ok {
            Ok(out)
        } else {
            Err(BfrError::InvalidSurface)
        }
    }

    /// Number of patch points (including computed irregular points).
    pub fn patch_point_count(&self) -> usize {
        unsafe { sys::bfr::surface_factory::Bfr_Surface_GetNumPatchPoints(self.ptr) as usize }
    }

    /// Gather patch points (control points for the surface's patch) into a
    /// vector. Returns the points in the order provided by BFR.
    pub fn gather_patch_points(&self, mesh_points: &[[f32; 3]]) -> Result<Vec<[f32; 3]>, BfrError> {
        if !self.is_valid() {
            return Err(BfrError::InvalidSurface);
        }

        let count = self.patch_point_count();
        let mut buf = vec![0.0f32; count * 3];
        let ok = unsafe {
            sys::bfr::surface_factory::Bfr_Surface_GatherPatchPoints(
                self.ptr,
                mesh_points.as_ptr() as *const f32,
                3,
                buf.as_mut_ptr(),
                count as i32,
            )
        };

        if !ok {
            return Err(BfrError::BufferTooSmall);
        }

        Ok(buf.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect())
    }
}

#[cfg(feature = "truck")]
use truck_geometry::prelude::{BSplineSurface, KnotVec, Point3};

#[cfg(feature = "truck")]
impl SurfaceFactory {
    /// Build B-spline surfaces for regular faces at the base level using BFR.
    /// Irregular faces are skipped.
    pub fn build_regular_surfaces(
        &self,
        refiner: &crate::far::TopologyRefiner,
        mesh_points: &[[f32; 3]],
    ) -> Result<Vec<BSplineSurface<Point3>>, BfrError> {
        let base = refiner.level(0).ok_or(BfrError::InitializationFailed)?;

        let mut surfaces = Vec::new();

        for face in 0..base.face_count() {
            let surface = self.init_vertex_surface(Index::from(face))?;
            if !surface.is_regular() {
                continue;
            }

            let patch_points = surface.gather_patch_points(mesh_points)?;
            if patch_points.len() != 16 {
                return Err(BfrError::UnsupportedPatchPointCount(patch_points.len()));
            }

            let mut control_matrix = vec![vec![Point3::new(0.0, 0.0, 0.0); 4]; 4];
            for (i, p) in patch_points.iter().enumerate() {
                let row = i / 4;
                let col = i % 4;
                control_matrix[row][col] = Point3::new(p[0] as f64, p[1] as f64, p[2] as f64);
            }

            let knots = KnotVec::from(vec![-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0]);
            surfaces.push(BSplineSurface::new((knots.clone(), knots), control_matrix));
        }

        Ok(surfaces)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { sys::bfr::surface_factory::Bfr_Surface_Destroy(self.ptr) }
    }
}
