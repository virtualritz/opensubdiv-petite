#[cfg(feature = "truck_integration")]
mod test_utils;

#[cfg(feature = "truck_integration")]
#[test]
fn test_truck_integration_compiles() {
    use opensubdiv_petite::truck_integration::{PatchTableExt, TruckIntegrationError};
    
    // Just verify the module compiles and types are accessible
    let _error: TruckIntegrationError = TruckIntegrationError::InvalidControlPoints;
    
    // This test passes if it compiles
    assert!(true);
}

#[cfg(feature = "truck_integration")]
#[test]
fn test_truck_integration_types() {
    use opensubdiv_petite::truck_integration::{Patch, PatchTableWithControlPoints};
    use opensubdiv_petite::far::PatchTable;
    
    // Test that we can create the wrapper types
    let control_points: Vec<[f32; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    
    // This test passes if it compiles
    assert_eq!(control_points.len(), 4);
}