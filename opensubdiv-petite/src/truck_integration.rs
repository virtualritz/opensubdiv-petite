//! Integration with the truck CAD kernel for B-rep surface generation
//!
//! This module provides `From` trait implementations to convert OpenSubdiv patches 
//! to truck's surface representations, enabling high-order surface export to STEP format.

use crate::far::{PatchTable, PatchType, PatchEvalResult};
use truck_geometry::prelude::*;
use truck_modeling::*;
use std::convert::TryFrom;

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
    pub fn new(patch_table: &'a PatchTable, patch_index: usize, control_points: &'a [[f32; 3]]) -> Self {
        Self {
            patch_table,
            patch_index,
            control_points,
        }
    }

    /// Get patch array information
    fn get_patch_info(&self) -> Result<(usize, usize, PatchType), TruckIntegrationError> {
        let mut current_index = self.patch_index;
        
        for array_idx in 0..self.patch_table.patch_arrays_len() {
            let array_size = self.patch_table.patch_array_patches_len(array_idx);
            if current_index < array_size {
                let desc = self.patch_table
                    .patch_array_descriptor(array_idx)
                    .ok_or(TruckIntegrationError::InvalidControlPoints)?;
                return Ok((array_idx, current_index, desc.patch_type()));
            }
            current_index -= array_size;
        }
        
        Err(TruckIntegrationError::InvalidControlPoints)
    }

    /// Extract control points for this patch
    fn get_control_points(&self) -> Result<Vec<Vec<Point3>>, TruckIntegrationError> {
        let (array_index, local_index, patch_type) = self.get_patch_info()?;
        
        if patch_type != PatchType::Regular {
            return Err(TruckIntegrationError::UnsupportedPatchType(patch_type));
        }
        
        const REGULAR_PATCH_SIZE: usize = 4;
        let desc = self.patch_table
            .patch_array_descriptor(array_index)
            .ok_or(TruckIntegrationError::InvalidControlPoints)?;
        
        if desc.control_vertices_len() != REGULAR_PATCH_SIZE * REGULAR_PATCH_SIZE {
            return Err(TruckIntegrationError::InvalidControlPoints);
        }
        
        let cv_indices = self.patch_table
            .patch_array_vertices(array_index)
            .ok_or(TruckIntegrationError::InvalidControlPoints)?;
        
        let start = local_index * desc.control_vertices_len();
        let patch_cvs = &cv_indices[start..start + desc.control_vertices_len()];
        
        let mut control_matrix = vec![vec![Point3::origin(); REGULAR_PATCH_SIZE]; REGULAR_PATCH_SIZE];
        
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
impl<'a> TryFrom<Patch<'a>> for BSplineSurface<Point3> {
    type Error = TruckIntegrationError;

    fn try_from(patch: Patch<'a>) -> Result<Self, Self::Error> {
        let control_matrix = patch.get_control_points()?;
        
        // Create uniform cubic B-spline knot vectors
        // For a cubic B-spline with 4 control points, we need 8 knots: [0,0,0,0,1,1,1,1]
        let u_knots = KnotVec::bezier_knot(3); // degree 3
        let v_knots = KnotVec::bezier_knot(3);
        
        Ok(BSplineSurface::new((u_knots, v_knots), control_matrix))
    }
}

/// Convert all regular patches to B-spline surfaces
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Vec<BSplineSurface<Point3>> {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> Result<Self, Self::Error> {
        let mut surfaces = Vec::new();
        let mut patch_index = 0;
        
        for array_idx in 0..patches.patch_table.patch_arrays_len() {
            if let Some(desc) = patches.patch_table.patch_array_descriptor(array_idx) {
                if desc.patch_type() == PatchType::Regular {
                    for _ in 0..patches.patch_table.patch_array_patches_len(array_idx) {
                        let patch = Patch::new(patches.patch_table, patch_index, patches.control_points);
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
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Shell<Point3, Curve, Surface> {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> Result<Self, Self::Error> {
        let surfaces: Vec<BSplineSurface<Point3>> = patches.try_into()?;
        let mut faces = Vec::new();
        
        for surface in surfaces {
            // Get parameter ranges
            let (u_range, v_range) = surface.parameter_range();
            let u_min = u_range.start;
            let u_max = u_range.end;
            let v_min = v_range.start;
            let v_max = v_range.end;
            
            // Create boundary curves
            let bottom_curve = surface.sectional_curve(u_min, 0);
            let right_curve = surface.sectional_curve(u_max, 1);
            let top_curve = surface.sectional_curve(u_max, 0);
            let left_curve = surface.sectional_curve(u_min, 1);
            
            // Create vertices
            let v00 = Vertex::new(surface.subs(u_min, v_min));
            let v10 = Vertex::new(surface.subs(u_max, v_min));
            let v11 = Vertex::new(surface.subs(u_max, v_max));
            let v01 = Vertex::new(surface.subs(u_min, v_max));
            
            // Create edges
            let e0 = Edge::new(&v00, &v10, bottom_curve);
            let e1 = Edge::new(&v10, &v11, right_curve);
            let e2 = Edge::new(&v11, &v01, top_curve.inverse());
            let e3 = Edge::new(&v01, &v00, left_curve.inverse());
            
            // Create wire boundary
            let wire = Wire::from(vec![e0, e1, e2, e3]);
            
            // Create face
            let face = Face::new(vec![wire], surface);
            faces.push(face);
        }
        
        Ok(Shell::from(faces))
    }
}

/// Convert patch evaluation result to Point3
impl From<PatchEvalResult> for Point3 {
    fn from(result: PatchEvalResult) -> Self {
        Point3::new(
            result.point[0] as f64,
            result.point[1] as f64,
            result.point[2] as f64,
        )
    }
}

/// Convert patch evaluation result to Vector3 (for derivatives)
impl From<&[f32; 3]> for Vector3 {
    fn from(v: &[f32; 3]) -> Self {
        Vector3::new(v[0] as f64, v[1] as f64, v[2] as f64)
    }
}

/// Extension trait for PatchTable to provide conversion methods
pub trait PatchTableExt {
    /// Create a wrapper for conversion to truck surfaces
    fn with_control_points<'a>(&'a self, control_points: &'a [[f32; 3]]) -> PatchTableWithControlPoints<'a>;
    
    /// Get a specific patch for conversion
    fn patch<'a>(&'a self, index: usize, control_points: &'a [[f32; 3]]) -> Patch<'a>;
}

impl PatchTableExt for PatchTable {
    fn with_control_points<'a>(&'a self, control_points: &'a [[f32; 3]]) -> PatchTableWithControlPoints<'a> {
        PatchTableWithControlPoints {
            patch_table: self,
            control_points,
        }
    }
    
    fn patch<'a>(&'a self, index: usize, control_points: &'a [[f32; 3]]) -> Patch<'a> {
        Patch::new(self, index, control_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_traits() {
        // This would require a proper patch table setup
        // Example usage:
        // let patch_table: PatchTable = ...;
        // let control_points: Vec<[f32; 3]> = ...;
        // 
        // // Convert a single patch
        // let surface: BSplineSurface<Point3> = patch_table.patch(0, &control_points).try_into()?;
        // 
        // // Convert all patches to surfaces
        // let surfaces: Vec<BSplineSurface<Point3>> = patch_table.with_control_points(&control_points).try_into()?;
        // 
        // // Convert to shell
        // let shell: Shell<Point3, Curve, Surface> = patch_table.with_control_points(&control_points).try_into()?;
    }
}