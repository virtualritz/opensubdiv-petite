//! Integration with the truck CAD kernel for B-rep surface generation
//!
//! This module provides converters from OpenSubdiv patches to truck's
//! surface representations, enabling high-order surface export to STEP format.

use crate::far::{PatchTable, PatchType, PatchEvalResult};
use truck_geometry::prelude::*;
use truck_modeling::*;

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

/// Convert an OpenSubdiv regular B-spline patch to a truck BSplineSurface
pub fn regular_patch_to_bspline_surface(
    patch_table: &PatchTable,
    patch_index: usize,
    control_points: &[[f32; 3]],
) -> Result<BSplineSurface<Point3>, TruckIntegrationError> {
    // Get patch descriptor to verify it's a regular patch
    let (array_index, local_index) = find_patch_array(patch_table, patch_index)?;
    let desc = patch_table
        .patch_array_descriptor(array_index)
        .ok_or(TruckIntegrationError::InvalidControlPoints)?;
    
    if desc.patch_type() != PatchType::Regular {
        return Err(TruckIntegrationError::UnsupportedPatchType(desc.patch_type()));
    }
    
    // Regular patches have 16 control vertices (4x4)
    const REGULAR_PATCH_SIZE: usize = 4;
    if desc.control_vertices_len() != REGULAR_PATCH_SIZE * REGULAR_PATCH_SIZE {
        return Err(TruckIntegrationError::InvalidControlPoints);
    }
    
    // Get control vertex indices for this patch
    let cv_indices = patch_table
        .patch_array_vertices(array_index)
        .ok_or(TruckIntegrationError::InvalidControlPoints)?;
    
    let start = local_index * desc.control_vertices_len();
    let patch_cvs = &cv_indices[start..start + desc.control_vertices_len()];
    
    // Create control points matrix for B-spline surface
    let mut control_matrix = vec![vec![Point3::origin(); REGULAR_PATCH_SIZE]; REGULAR_PATCH_SIZE];
    
    for (i, &cv_idx) in patch_cvs.iter().enumerate() {
        let row = i / REGULAR_PATCH_SIZE;
        let col = i % REGULAR_PATCH_SIZE;
        
        let idx = cv_idx.into();
        if idx >= control_points.len() {
            return Err(TruckIntegrationError::InvalidControlPoints);
        }
        
        let cp = &control_points[idx];
        control_matrix[row][col] = Point3::new(cp[0] as f64, cp[1] as f64, cp[2] as f64);
    }
    
    // Create uniform cubic B-spline knot vectors
    // For a cubic B-spline with 4 control points, we need 8 knots: [0,0,0,0,1,1,1,1]
    let u_knots = KnotVec::bezier_knot(3); // degree 3
    let v_knots = KnotVec::bezier_knot(3);
    
    // Create the B-spline surface
    Ok(BSplineSurface::new((u_knots, v_knots), control_matrix))
}

/// Convert multiple patches to a compound surface
pub fn patches_to_surfaces(
    patch_table: &PatchTable,
    control_points: &[[f32; 3]],
) -> Result<Vec<BSplineSurface<Point3>>, TruckIntegrationError> {
    let mut surfaces = Vec::new();
    
    // Iterate through all patches
    let mut patch_index = 0;
    for array_idx in 0..patch_table.patch_arrays_len() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            // Only convert regular B-spline patches for now
            if desc.patch_type() == PatchType::Regular {
                for _ in 0..patch_table.patch_array_patches_len(array_idx) {
                    if let Ok(surface) = regular_patch_to_bspline_surface(
                        patch_table,
                        patch_index,
                        control_points,
                    ) {
                        surfaces.push(surface);
                    }
                    patch_index += 1;
                }
            } else {
                // Skip non-regular patches for now
                patch_index += patch_table.patch_array_patches_len(array_idx);
            }
        }
    }
    
    Ok(surfaces)
}

/// Evaluate a patch and create a sampled surface
pub fn patch_to_sampled_surface(
    patch_table: &PatchTable,
    patch_index: usize,
    control_points: &[[f32; 3]],
    u_samples: usize,
    v_samples: usize,
) -> Result<Vec<Vec<Point3>>, TruckIntegrationError> {
    let mut points = vec![vec![Point3::origin(); v_samples]; u_samples];
    
    for u_idx in 0..u_samples {
        for v_idx in 0..v_samples {
            let u = u_idx as f32 / (u_samples - 1) as f32;
            let v = v_idx as f32 / (v_samples - 1) as f32;
            
            let eval_result = patch_table
                .evaluate_point(patch_index, u, v, control_points)
                .ok_or(TruckIntegrationError::EvaluationFailed)?;
            
            points[u_idx][v_idx] = Point3::new(
                eval_result.point[0] as f64,
                eval_result.point[1] as f64,
                eval_result.point[2] as f64,
            );
        }
    }
    
    Ok(points)
}

/// Helper to find which patch array a global patch index belongs to
fn find_patch_array(
    patch_table: &PatchTable,
    patch_index: usize,
) -> Result<(usize, usize), TruckIntegrationError> {
    let mut current_index = patch_index;
    
    for array_idx in 0..patch_table.patch_arrays_len() {
        let array_size = patch_table.patch_array_patches_len(array_idx);
        if current_index < array_size {
            return Ok((array_idx, current_index));
        }
        current_index -= array_size;
    }
    
    Err(TruckIntegrationError::InvalidControlPoints)
}

/// Create a truck Shell from OpenSubdiv patches
pub fn patches_to_shell(
    patch_table: &PatchTable,
    control_points: &[[f32; 3]],
) -> Result<Shell<Point3, Curve, Surface>, TruckIntegrationError> {
    let surfaces = patches_to_surfaces(patch_table, control_points)?;
    
    if surfaces.is_empty() {
        return Err(TruckIntegrationError::InvalidControlPoints);
    }
    
    // Create faces from surfaces
    let mut faces = Vec::new();
    
    for surface in surfaces {
        // Create edges for the surface boundary
        // For a B-spline surface, we need to extract the boundary curves
        let u_min = surface.parameter_range().0.start;
        let u_max = surface.parameter_range().0.end;
        let v_min = surface.parameter_range().1.start;
        let v_max = surface.parameter_range().1.end;
        
        // Create boundary curves
        let bottom_curve = surface.sectional_curve(u_min, 0); // u=0 curve
        let right_curve = surface.sectional_curve(u_max, 1);  // v=1 curve
        let top_curve = surface.sectional_curve(u_max, 0);    // u=1 curve
        let left_curve = surface.sectional_curve(u_min, 1);   // v=0 curve
        
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
    
    // Create shell from faces
    Ok(Shell::from(faces))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regular_patch_conversion() {
        // This would require a proper patch table setup
        // Placeholder for actual tests
    }
}