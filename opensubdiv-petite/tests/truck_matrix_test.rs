#[cfg(feature = "truck")]
#[test]
fn test_truck_bspline_matrix_ordering() {
    use truck_geometry::prelude::*;
    
    // Create a simple B-spline surface with known control points
    // to understand truck's expected ordering
    
    // Define a 4x4 grid of control points
    let control_points = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 0.0, 0.0)],
        vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.5), Point3::new(2.0, 1.0, 0.5), Point3::new(3.0, 1.0, 0.0)],
        vec![Point3::new(0.0, 2.0, 0.0), Point3::new(1.0, 2.0, 0.5), Point3::new(2.0, 2.0, 0.5), Point3::new(3.0, 2.0, 0.0)],
        vec![Point3::new(0.0, 3.0, 0.0), Point3::new(1.0, 3.0, 0.0), Point3::new(2.0, 3.0, 0.0), Point3::new(3.0, 3.0, 0.0)],
    ];
    
    // Create uniform B-spline knot vectors
    let u_knots = KnotVec::uniform_knot(3, 4);
    let v_knots = KnotVec::uniform_knot(3, 4);
    
    let surface = BSplineSurface::new((u_knots.clone(), v_knots.clone()), control_points);
    
    // Sample the surface at various points
    println!("\n=== TRUCK B-SPLINE SURFACE TEST ===");
    println!("Testing surface evaluation at different (u,v) coordinates:");
    
    for (u, v) in &[(0.0, 0.0), (0.5, 0.0), (1.0, 0.0), (0.0, 0.5), (0.5, 0.5), (1.0, 0.5), (0.0, 1.0), (0.5, 1.0), (1.0, 1.0)] {
        let point = surface.subs(*u, *v);
        println!("  Surface at ({:.1}, {:.1}) = [{:.3}, {:.3}, {:.3}]", u, v, point.x, point.y, point.z);
    }
    
    // Now test with transposed control points
    let control_points_transposed = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 2.0, 0.0), Point3::new(0.0, 3.0, 0.0)],
        vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.5), Point3::new(1.0, 2.0, 0.5), Point3::new(1.0, 3.0, 0.0)],
        vec![Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 1.0, 0.5), Point3::new(2.0, 2.0, 0.5), Point3::new(2.0, 3.0, 0.0)],
        vec![Point3::new(3.0, 0.0, 0.0), Point3::new(3.0, 1.0, 0.0), Point3::new(3.0, 2.0, 0.0), Point3::new(3.0, 3.0, 0.0)],
    ];
    
    let surface_transposed = BSplineSurface::new((u_knots.clone(), v_knots.clone()), control_points_transposed);
    
    println!("\n=== TRANSPOSED CONTROL POINTS ===");
    for (u, v) in &[(0.0, 0.0), (0.5, 0.0), (1.0, 0.0), (0.0, 0.5), (0.5, 0.5), (1.0, 0.5), (0.0, 1.0), (0.5, 1.0), (1.0, 1.0)] {
        let point = surface_transposed.subs(*u, *v);
        println!("  Surface at ({:.1}, {:.1}) = [{:.3}, {:.3}, {:.3}]", u, v, point.x, point.y, point.z);
    }
    
    println!("\n=== END TRUCK B-SPLINE TEST ===\n");
}