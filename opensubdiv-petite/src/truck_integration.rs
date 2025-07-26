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
    Edge, Face, Shell, Vertex, Wire, Curve, Surface,
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
            Self::UnsupportedPatchType(t) => write!(f, "Unsupported patch type: {t:?}"),
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
        // OpenSubdiv regular patches are expressed as bicubic B-spline patches in Far::PatchTable.
        // The control points are B-spline control points, NOT Bezier control points.
        //
        // For OpenSubdiv patches, the standard approach is to use a uniform knot vector
        // and evaluate the surface in the parameter range [1/3, 2/3] to exclude phantom points.
        // However, since we need to work with STEP files which expect standard parameter ranges,
        // we'll use a knot vector that maps [0,1] to the interior of the patch.
        //
        // This knot vector creates the effect of evaluating a standard uniform B-spline
        // in the range [1/3, 2/3] but remapped to [0,1] for compatibility.
        let u_knots = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0/3.0, 2.0/3.0, 1.0, 1.0, 1.0]);
        let v_knots = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0/3.0, 2.0/3.0, 1.0, 1.0, 1.0]);

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

// AIDEV-NOTE: Commented out full B-rep Shell implementation with shared edges
// This implementation creates a proper B-rep with shared vertices and edges,
// but for debugging we're using a simpler disconnected patch approach below.
/*
/// Convert patches to a complete Shell with shared topology
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Shell {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> std::result::Result<Self, Self::Error> {
        let surfaces: Vec<BSplineSurface<Point3<f64>>> = patches.try_into()?;
        
        use std::collections::HashMap;
        use truck_geometry::prelude::BSplineCurve;
        
        // AIDEV-NOTE: Create proper B-rep with shared vertices and edges
        // Following the pattern from truck-topology's cube example, we need to:
        // 1. Create all vertices first
        // 2. Create all edges between vertices
        // 3. Build faces using these edges with proper orientation
        
        // Tolerance for position comparison
        const TOLERANCE: f64 = 1e-10;
        
        // First pass: collect all unique corner points and create vertices
        let mut vertex_map: HashMap<[i64; 3], Vertex> = HashMap::new();
        let mut surface_corners = Vec::new();
        
        for surface in &surfaces {
            // Get the four corner points
            let p00 = surface.subs(0.0, 0.0);
            let p10 = surface.subs(1.0, 0.0);
            let p11 = surface.subs(1.0, 1.0);
            let p01 = surface.subs(0.0, 1.0);
            
            // Get or create vertices
            let mut get_or_create_vertex = |point: Point3<f64>| -> Vertex {
                let key = [
                    (point.x / TOLERANCE).round() as i64,
                    (point.y / TOLERANCE).round() as i64,
                    (point.z / TOLERANCE).round() as i64,
                ];
                
                vertex_map.entry(key)
                    .or_insert_with(|| Vertex::new(point))
                    .clone()
            };
            
            let v00 = get_or_create_vertex(p00);
            let v10 = get_or_create_vertex(p10);
            let v11 = get_or_create_vertex(p11);
            let v01 = get_or_create_vertex(p01);
            
            surface_corners.push((v00, v10, v11, v01, p00, p10, p11, p01));
        }
        
        // Second pass: create all unique edges
        type EdgeKey = ([i64; 3], [i64; 3]);
        let mut edge_map: HashMap<EdgeKey, Edge> = HashMap::new();
        
        let make_edge_key = |p0: Point3<f64>, p1: Point3<f64>| -> EdgeKey {
            let k0 = [
                (p0.x / TOLERANCE).round() as i64,
                (p0.y / TOLERANCE).round() as i64,
                (p0.z / TOLERANCE).round() as i64,
            ];
            let k1 = [
                (p1.x / TOLERANCE).round() as i64,
                (p1.y / TOLERANCE).round() as i64,
                (p1.z / TOLERANCE).round() as i64,
            ];
            // Always order vertices consistently for the key
            if k0 <= k1 { (k0, k1) } else { (k1, k0) }
        };
        
        // Collect all edges from all patches
        for (v00, v10, v11, v01, p00, p10, p11, p01) in &surface_corners {
            // Helper to create or get edge
            let mut get_or_create_edge = |v0: &Vertex, v1: &Vertex, p0: Point3<f64>, p1: Point3<f64>| {
                let key = make_edge_key(p0, p1);
                edge_map.entry(key)
                    .or_insert_with(|| {
                        // Always create edge in consistent direction based on key
                        if make_edge_key(p0, p1) == (key.0, key.1) {
                            Edge::new(v0, v1, Curve::BSplineCurve(
                                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p0, p1])
                            ))
                        } else {
                            Edge::new(v1, v0, Curve::BSplineCurve(
                                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p1, p0])
                            ))
                        }
                    });
            };
            
            // Create all four edges for this patch
            get_or_create_edge(v00, v10, *p00, *p10);
            get_or_create_edge(v10, v11, *p10, *p11);
            get_or_create_edge(v11, v01, *p11, *p01);
            get_or_create_edge(v01, v00, *p01, *p00);
        }
        
        // Third pass: create faces using the shared edges
        let mut faces = Vec::new();
        
        for (i, (surface, (v00, v10, v11, v01, p00, p10, p11, p01))) in surfaces.into_iter().zip(surface_corners).enumerate() {
            // Get the edges for this face
            let bottom_edge = edge_map.get(&make_edge_key(p00, p10)).unwrap();
            let right_edge = edge_map.get(&make_edge_key(p10, p11)).unwrap();
            let top_edge = edge_map.get(&make_edge_key(p11, p01)).unwrap();
            let left_edge = edge_map.get(&make_edge_key(p01, p00)).unwrap();
            
            // Calculate face normal at the center to determine proper orientation
            let center_u = 0.5;
            let center_v = 0.5;
            let _center_pt = surface.subs(center_u, center_v);
            let du = surface.uder(center_u, center_v);
            let dv = surface.vder(center_u, center_v);
            let normal = du.cross(dv);
            
            // Compute the expected outward normal based on corner points
            // Using (p10-p00) x (p01-p00) which should point outward for CCW winding
            let edge1 = Vector3::new(p10.x - p00.x, p10.y - p00.y, p10.z - p00.z);
            let edge2 = Vector3::new(p01.x - p00.x, p01.y - p00.y, p01.z - p00.z);
            let expected_normal = edge1.cross(edge2);
            
            // Check if surface normal matches expected normal
            let dot = normal.dot(expected_normal);
            let needs_inversion = dot < 0.0;
            
            if needs_inversion {
                eprintln!("Warning: Face {} has inverted normal (dot = {})", i, dot);
            }
            
            // Determine proper orientation for each edge
            let bottom = if bottom_edge.front() == &v00 {
                bottom_edge.clone()
            } else {
                bottom_edge.inverse()
            };
            
            let right = if right_edge.front() == &v10 {
                right_edge.clone()
            } else {
                right_edge.inverse()
            };
            
            let top = if top_edge.front() == &v11 {
                top_edge.clone()
            } else {
                top_edge.inverse()
            };
            
            let left = if left_edge.front() == &v01 {
                left_edge.clone()
            } else {
                left_edge.inverse()
            };
            
            // Create wire and face
            let wire = if needs_inversion {
                // Reverse the edge order to flip the face normal
                Wire::from(vec![bottom.inverse(), left.inverse(), top.inverse(), right.inverse()])
            } else {
                Wire::from(vec![bottom, right, top, left])
            };
            
            let mut face = Face::new(vec![wire], Surface::BSplineSurface(surface));
            if needs_inversion {
                face.invert();
            }
            faces.push(face);
        }

        let shell = Shell::from(faces);
        
        Ok(shell)
    }
}
*/

/// Convert patches to a simple Shell with disconnected faces
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Shell {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> std::result::Result<Self, Self::Error> {
        let surfaces: Vec<BSplineSurface<Point3<f64>>> = patches.try_into()?;
        
        
        // Create disconnected faces, each with its own vertices and edges
        let mut faces = Vec::new();
        
        for surface in surfaces {
            // Get the four corner points of the surface
            // For our knot configuration, the valid surface is in [1/3, 2/3] range
            let p00 = surface.subs(1.0/3.0, 1.0/3.0);
            let p10 = surface.subs(2.0/3.0, 1.0/3.0);
            let p11 = surface.subs(2.0/3.0, 2.0/3.0);
            let p01 = surface.subs(1.0/3.0, 2.0/3.0);
            
            // Create vertices
            let v00 = Vertex::new(p00);
            let v10 = Vertex::new(p10);
            let v11 = Vertex::new(p11);
            let v01 = Vertex::new(p01);
            
            // Create edges with linear curves
            use truck_geometry::prelude::BSplineCurve;
            let e0 = Edge::new(&v00, &v10, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p00, p10])
            ));
            let e1 = Edge::new(&v10, &v11, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p10, p11])
            ));
            let e2 = Edge::new(&v11, &v01, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p11, p01])
            ));
            let e3 = Edge::new(&v01, &v00, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p01, p00])
            ));
            
            // Create wire and face
            let wire = Wire::from(vec![e0, e1, e2, e3]);
            let face = Face::new(vec![wire], Surface::BSplineSurface(surface));
            faces.push(face);
        }
        
        Ok(Shell::from(faces))
    }
}

/// Convert patches to a vector of individual Shells (one face per shell)
impl<'a> TryFrom<PatchTableWithControlPoints<'a>> for Vec<Shell> {
    type Error = TruckIntegrationError;

    fn try_from(patches: PatchTableWithControlPoints<'a>) -> std::result::Result<Self, Self::Error> {
        let surfaces: Vec<BSplineSurface<Point3<f64>>> = patches.try_into()?;
        
        // Create one shell per surface for disconnected export
        let mut shells = Vec::new();
        
        for surface in surfaces {
            
            // Get the four corner points of the surface
            // For our knot configuration, the valid surface is in [1/3, 2/3] range
            let p00 = surface.subs(1.0/3.0, 1.0/3.0);
            let p10 = surface.subs(2.0/3.0, 1.0/3.0);
            let p11 = surface.subs(2.0/3.0, 2.0/3.0);
            let p01 = surface.subs(1.0/3.0, 2.0/3.0);
            
            // Create vertices
            let v00 = Vertex::new(p00);
            let v10 = Vertex::new(p10);
            let v11 = Vertex::new(p11);
            let v01 = Vertex::new(p01);
            
            // Create edges with linear curves
            use truck_geometry::prelude::BSplineCurve;
            let e0 = Edge::new(&v00, &v10, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p00, p10])
            ));
            let e1 = Edge::new(&v10, &v11, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p10, p11])
            ));
            let e2 = Edge::new(&v11, &v01, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p11, p01])
            ));
            let e3 = Edge::new(&v01, &v00, Curve::BSplineCurve(
                BSplineCurve::new(KnotVec::bezier_knot(1), vec![p01, p00])
            ));
            
            // Create wire and face
            let wire = Wire::from(vec![e0, e1, e2, e3]);
            let face = Face::new(vec![wire], Surface::BSplineSurface(surface));
            
            // Create a shell with just this one face
            shells.push(Shell::from(vec![face]));
        }
        
        Ok(shells)
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
    
    /// Convert patches to individual shells (one per patch) for disconnected export
    fn to_truck_shells(&self, control_points: &[[f32; 3]]) -> Result<Vec<Shell>>;
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
    
    fn to_truck_shells(&self, control_points: &[[f32; 3]]) -> Result<Vec<Shell>> {
        let wrapper = self.with_control_points(control_points);
        Vec::<Shell>::try_from(wrapper)
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