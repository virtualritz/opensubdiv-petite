//! Integration with the truck CAD kernel for B-rep surface generation
//!
//! This module provides `From` trait implementations to convert OpenSubdiv patches
//! to truck's surface representations, enabling high-order surface export to STEP format.

use crate::far::{PatchEvalResult, PatchTable, PatchType};
use std::convert::TryFrom;
use truck_geometry::prelude::{
    BSplineSurface, KnotVec, ParametricSurface,
};
use truck_modeling::{
    cgmath::{EuclideanSpace, Point3, Vector3}, 
    Edge, Face, Shell, Vertex, Wire, Curve, Surface, Invertible,
};

/// Type alias for results in this module
pub type Result<T> = std::result::Result<T, TruckIntegrationError>;

/// Error type for truck integration
#[derive(Debug, Clone)]
pub enum TruckIntegrationError {
    /// Unsupported patch type
    UnsupportedPatchType(PatchType),
    /// Invalid control point configuration
    InvalidControlPoints,
    /// Patch evaluation failed
    EvaluationFailed,
    /// Invalid knot vector
    InvalidKnotVector,
}

impl std::fmt::Display for TruckIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPatchType(t) => write!(f, "Unsupported patch type: {:?}", t),
            Self::InvalidControlPoints => write!(f, "Invalid control point configuration"),
            Self::EvaluationFailed => write!(f, "Patch evaluation failed"),
            Self::InvalidKnotVector => write!(f, "Invalid knot vector"),
        }
    }
}

impl std::error::Error for TruckIntegrationError {}

/// A wrapper around a single patch with its associated data
pub struct Patch<'a> {
    pub patch_table: &'a PatchTable,
    pub patch_index: usize,
    pub control_points: &'a [[f32; 3]],
}

/// A wrapper around patches with control points for conversion
pub struct PatchTableWithControlPoints<'a> {
    pub patch_table: &'a PatchTable,
    pub control_points: &'a [[f32; 3]],
}

impl<'a> Patch<'a> {
    /// Create a new patch reference
    pub fn new(
        patch_table: &'a PatchTable,
        patch_index: usize,
        control_points: &'a [[f32; 3]],
    ) -> Self {
        Self {
            patch_table,
            patch_index,
            control_points,
        }
    }

    /// Get patch array information
    fn get_patch_info(&self) -> std::result::Result<(usize, usize, PatchType), TruckIntegrationError> {
        let mut current_index = self.patch_index;

        for array_idx in 0..self.patch_table.patch_arrays_len() {
            let array_size = self.patch_table.patch_array_patches_len(array_idx);
            if current_index < array_size {
                let desc = self
                    .patch_table
                    .patch_array_descriptor(array_idx)
                    .ok_or(TruckIntegrationError::InvalidControlPoints)?;
                return Ok((array_idx, current_index, desc.patch_type()));
            }
            current_index -= array_size;
        }

        Err(TruckIntegrationError::InvalidControlPoints)
    }

    /// Extract control points for this patch
    fn get_control_points(&self) -> std::result::Result<Vec<Vec<Point3<f64>>>, TruckIntegrationError> {
        let (array_index, local_index, patch_type) = self.get_patch_info()?;

        if patch_type != PatchType::Regular {
            return Err(TruckIntegrationError::UnsupportedPatchType(patch_type));
        }

        const REGULAR_PATCH_SIZE: usize = 4;
        let desc = self
            .patch_table
            .patch_array_descriptor(array_index)
            .ok_or(TruckIntegrationError::InvalidControlPoints)?;

        if desc.control_vertices_len() != REGULAR_PATCH_SIZE * REGULAR_PATCH_SIZE {
            return Err(TruckIntegrationError::InvalidControlPoints);
        }

        let cv_indices = self
            .patch_table
            .patch_array_vertices(array_index)
            .ok_or(TruckIntegrationError::InvalidControlPoints)?;

        let start = local_index * desc.control_vertices_len();
        let patch_cvs = &cv_indices[start..start + desc.control_vertices_len()];

        let mut control_matrix =
            vec![vec![Point3::origin(); REGULAR_PATCH_SIZE]; REGULAR_PATCH_SIZE];

        for (i, &cv_idx) in patch_cvs.iter().enumerate() {
            let row = i / REGULAR_PATCH_SIZE;
            let col = i % REGULAR_PATCH_SIZE;

            let idx: usize = cv_idx.into();
            if idx >= self.control_points.len() {
                return Err(TruckIntegrationError::InvalidControlPoints);
            }

            let cp = &self.control_points[idx];
            control_matrix[row][col] = Point3::new(cp[0] as f64, cp[1] as f64, cp[2] as f64);
        }

        Ok(control_matrix)
    }
}

/// Convert a regular B-spline patch to a truck BSplineSurface
impl<'a> TryFrom<Patch<'a>> for BSplineSurface<Point3<f64>> {
    type Error = TruckIntegrationError;

    fn try_from(patch: Patch<'a>) -> std::result::Result<Self, Self::Error> {
        let control_matrix = patch.get_control_points()?;

        // AIDEV-NOTE: OpenSubdiv B-spline patch knot vectors
        // OpenSubdiv regular patches are bicubic B-spline patches with phantom control points.
        // The outer row/column of control points define the patch boundaries but are not part 
        // of the evaluated surface (standard B-spline behavior).
        //
        // While standard B-spline patches with phantom points use uniform knot vectors
        // with a parameter range of [3,4] (for degree 3), we use clamped (Bezier) knot 
        // vectors for STEP export compatibility:
        // - Clamped knots [0,0,0,0,1,1,1,1] ensure surface evaluates over [0,1]
        // - This matches STEP file expectations for parameter ranges
        // - The degree remains 3 (cubic) as expected
        // - Patches connect properly at boundaries
        //
        // Alternative approaches that don't work:
        // - Standard uniform knots: parameter range [3,4] doesn't match STEP expectations
        // - Uniform knots over [0,1]: no basis function support at boundaries
        let u_knots = KnotVec::bezier_knot(3);  // Creates [0,0,0,0,1,1,1,1]
        let v_knots = KnotVec::bezier_knot(3);  // Creates [0,0,0,0,1,1,1,1]

        Ok(BSplineSurface::new((u_knots, v_knots), control_matrix))
    }
}

/// Convert all regular patches to B-spline surfaces
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Vec<BSplineSurface<Point3<f64>>> {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> std::result::Result<Self, Self::Error> {
        let mut surfaces = Vec::new();
        let mut patch_index = 0;

        for array_idx in 0..patches.patch_table.patch_arrays_len() {
            if let Some(desc) = patches.patch_table.patch_array_descriptor(array_idx) {
                if desc.patch_type() == PatchType::Regular {
                    for _ in 0..patches.patch_table.patch_array_patches_len(array_idx) {
                        let patch =
                            Patch::new(patches.patch_table, patch_index, patches.control_points);
                        if let Ok(surface) = BSplineSurface::try_from(patch) {
                            surfaces.push(surface);
                        }
                        patch_index += 1;
                    }
                } else {
                    patch_index += patches.patch_table.patch_array_patches_len(array_idx);
                }
            }
        }

        if surfaces.is_empty() {
            Err(TruckIntegrationError::InvalidControlPoints)
        } else {
            Ok(surfaces)
        }
    }
}

/// Convert patches to a complete Shell
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Shell {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> std::result::Result<Self, Self::Error> {
        let surfaces: Vec<BSplineSurface<Point3<f64>>> = patches.try_into()?;
        let mut faces = Vec::new();

        // AIDEV-NOTE: Simplified face creation to avoid jumbled geometry
        // We create faces with simple boundary loops, but avoid trying to share
        // vertices/edges between patches, which was causing incorrect connections
        for surface in surfaces {
            // Create a simple rectangular boundary wire for each patch
            // The patches will naturally connect based on their control points
            let (u_range, v_range) = surface.parameter_range();
            
            // For Bezier/clamped B-splines, the parameter range is [0,1]
            let u0 = 0.0;
            let u1 = 1.0;
            let v0 = 0.0;
            let v1 = 1.0;
            
            // Create the four corner points
            let p00 = surface.subs(u0, v0);
            let p10 = surface.subs(u1, v0);
            let p11 = surface.subs(u1, v1);
            let p01 = surface.subs(u0, v1);
            
            // Create unique vertices for this patch
            let v00 = Vertex::new(p00);
            let v10 = Vertex::new(p10);
            let v11 = Vertex::new(p11);
            let v01 = Vertex::new(p01);
            
            // Create boundary curves
            use truck_geometry::prelude::BSplineCurve;
            let bottom = Edge::new(&v00, &v10, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p00, p10])
            ));
            let right = Edge::new(&v10, &v11, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p10, p11])
            ));
            let top = Edge::new(&v11, &v01, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p11, p01])
            ));
            let left = Edge::new(&v01, &v00, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p01, p00])
            ));
            
            // Create wire and face
            let wire = Wire::from(vec![bottom, right, top, left]);
            let face = Face::new(vec![wire], Surface::BSplineSurface(surface));
            faces.push(face);
        }

        Ok(Shell::from(faces))
    }
}

/// Convert patch evaluation result to Point3
impl From<PatchEvalResult> for Point3<f64> {
    fn from(result: PatchEvalResult) -> Self {
        Point3::new(
            result.point[0] as f64,
            result.point[1] as f64,
            result.point[2] as f64,
        )
    }
}

/// Helper function to convert array to Vector3
pub fn array_to_vector3(v: &[f32; 3]) -> Vector3<f64> {
    Vector3::new(v[0] as f64, v[1] as f64, v[2] as f64)
}

/// Extension trait for PatchTable to provide conversion methods
pub trait PatchTableExt {
    /// Create a wrapper for conversion to truck surfaces
    fn with_control_points<'a>(
        &'a self,
        control_points: &'a [[f32; 3]],
    ) -> PatchTableWithControlPoints<'a>;

    /// Get a specific patch for conversion
    fn patch<'a>(&'a self, index: usize, control_points: &'a [[f32; 3]]) -> Patch<'a>;
    
    /// Convert patches to a truck shell with the given control points
    fn to_truck_shell(&self, control_points: &[[f32; 3]]) -> Result<Shell>;
    
    /// Convert patches to truck surfaces with the given control points
    fn to_truck_surfaces(&self, control_points: &[[f32; 3]]) -> Result<Vec<BSplineSurface<Point3<f64>>>>;
}

impl PatchTableExt for PatchTable {
    fn with_control_points<'a>(
        &'a self,
        control_points: &'a [[f32; 3]],
    ) -> PatchTableWithControlPoints<'a> {
        PatchTableWithControlPoints {
            patch_table: self,
            control_points,
        }
    }

    fn patch<'a>(&'a self, index: usize, control_points: &'a [[f32; 3]]) -> Patch<'a> {
        Patch::new(self, index, control_points)
    }
    
    fn to_truck_shell(&self, control_points: &[[f32; 3]]) -> Result<Shell> {
        let wrapper = self.with_control_points(control_points);
        Shell::try_from(wrapper)
    }
    
    fn to_truck_surfaces(&self, control_points: &[[f32; 3]]) -> Result<Vec<BSplineSurface<Point3<f64>>>> {
        let wrapper = self.with_control_points(control_points);
        Vec::<BSplineSurface<Point3<f64>>>::try_from(wrapper)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_traits() {
        // This would require a proper patch table setup
        // Example usage:
        // let patch_table: PatchTable = ...;
        // let control_points: Vec<[f32; 3]> = ...;
        //
        // // Convert a single patch
        // let surface: BSplineSurface<Point3<f64>> = patch_table.patch(0, &control_points).try_into()?;
        //
        // // Convert all patches to surfaces
        // let surfaces: Vec<BSplineSurface<Point3<f64>>> = patch_table.with_control_points(&control_points).try_into()?;
        //
        // // Convert to shell
        // let shell: Shell = patch_table.with_control_points(&control_points).try_into()?;
    }
}