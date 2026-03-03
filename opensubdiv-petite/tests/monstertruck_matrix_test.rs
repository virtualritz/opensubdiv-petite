#[cfg(feature = "monstertruck")]
#[test]
fn test_monstertruck_bspline_matrix_ordering() {
    use monstertruck_geometry::prelude::*;

    // Create a simple B-spline surface with known control points
    // to understand monstertruck's expected ordering

    // Define a 4x4 grid of control points
    let control_points = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.5),
            Point3::new(2.0, 1.0, 0.5),
            Point3::new(3.0, 1.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(1.0, 2.0, 0.5),
            Point3::new(2.0, 2.0, 0.5),
            Point3::new(3.0, 2.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 3.0, 0.0),
            Point3::new(1.0, 3.0, 0.0),
            Point3::new(2.0, 3.0, 0.0),
            Point3::new(3.0, 3.0, 0.0),
        ],
    ];

    // Create uniform B-spline knot vectors
    let u_knots = KnotVector::uniform_knot(3, 4);
    let v_knots = KnotVector::uniform_knot(3, 4);

    let surface = BsplineSurface::new((u_knots.clone(), v_knots.clone()), control_points);

    // Sample the surface at various points
    println!("\n=== MONSTERTRUCK B-SPLINE SURFACE TEST ===");
    println!("Testing surface evaluation at different (u,v) coordinates:");

    for (u, v) in &[
        (0.0, 0.0),
        (0.5, 0.0),
        (1.0, 0.0),
        (0.0, 0.5),
        (0.5, 0.5),
        (1.0, 0.5),
        (0.0, 1.0),
        (0.5, 1.0),
        (1.0, 1.0),
    ] {
        let point = surface.subs(*u, *v);
        println!(
            "  Surface at ({:.1}, {:.1}) = [{:.3}, {:.3}, {:.3}]",
            u, v, point.x, point.y, point.z
        );
    }

    // Now test with transposed control points
    let control_points_transposed = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(0.0, 3.0, 0.0),
        ],
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.5),
            Point3::new(1.0, 2.0, 0.5),
            Point3::new(1.0, 3.0, 0.0),
        ],
        vec![
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 1.0, 0.5),
            Point3::new(2.0, 2.0, 0.5),
            Point3::new(2.0, 3.0, 0.0),
        ],
        vec![
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(3.0, 1.0, 0.0),
            Point3::new(3.0, 2.0, 0.0),
            Point3::new(3.0, 3.0, 0.0),
        ],
    ];

    let surface_transposed = BsplineSurface::new(
        (u_knots.clone(), v_knots.clone()),
        control_points_transposed,
    );

    println!("\n=== TRANSPOSED CONTROL POINTS ===");
    for (u, v) in &[
        (0.0, 0.0),
        (0.5, 0.0),
        (1.0, 0.0),
        (0.0, 0.5),
        (0.5, 0.5),
        (1.0, 0.5),
        (0.0, 1.0),
        (0.5, 1.0),
        (1.0, 1.0),
    ] {
        let point = surface_transposed.subs(*u, *v);
        println!(
            "  Surface at ({:.1}, {:.1}) = [{:.3}, {:.3}, {:.3}]",
            u, v, point.x, point.y, point.z
        );
    }

    println!("\n=== END MONSTERTRUCK B-SPLINE TEST ===\n");
}
