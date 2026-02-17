use opensubdiv_petite::far;

fn cube_refiner() -> far::TopologyRefiner {
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices).unwrap();
    let options = far::TopologyRefinerOptions::default();
    let mut refiner = far::TopologyRefiner::new(descriptor, options).unwrap();

    refiner.refine_adaptive(far::AdaptiveRefinementOptions::default(), &[]);
    refiner
}

#[test]
fn limit_stencil_basic() {
    let refiner = cube_refiner();

    let s = [0.25_f32, 0.5, 0.75];
    let t = [0.25_f32, 0.5, 0.75];

    let locations = [far::LocationArray {
        ptex_index: 0,
        s: &s,
        t: &t,
    }];

    let table = far::LimitStencilTable::new(
        &refiner,
        &locations,
        None,
        None,
        far::LimitStencilTableOptions::default(),
    )
    .unwrap();

    assert_eq!(table.len(), 3);
    assert!(!table.is_empty());
    assert!(table.control_vertex_count() > 0);

    // 1st derivatives enabled by default.
    assert!(table.has_1st_derivatives());
    assert!(!table.du_weights().is_empty());
    assert!(!table.dv_weights().is_empty());

    // Base accessors should be consistent.
    assert_eq!(table.sizes().len(), table.len());
    assert_eq!(table.offsets().len(), table.len());
    assert!(!table.control_indices().is_empty());
    assert!(!table.weights().is_empty());
    assert_eq!(table.weights().len(), table.du_weights().len());
    assert_eq!(table.weights().len(), table.dv_weights().len());
}

#[test]
fn limit_stencil_2nd_derivatives() {
    let refiner = cube_refiner();

    let s = [0.5_f32];
    let t = [0.5_f32];

    let locations = [far::LocationArray {
        ptex_index: 0,
        s: &s,
        t: &t,
    }];

    let table = far::LimitStencilTable::new(
        &refiner,
        &locations,
        None,
        None,
        far::LimitStencilTableOptions {
            generate_2nd_derivatives: true,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(table.len(), 1);
    assert!(table.has_2nd_derivatives());
    assert!(!table.duu_weights().is_empty());
    assert!(!table.duv_weights().is_empty());
    assert!(!table.dvv_weights().is_empty());
}

#[test]
fn limit_stencil_multiple_faces() {
    let refiner = cube_refiner();

    let s0 = [0.25_f32, 0.75];
    let t0 = [0.25_f32, 0.75];
    let s1 = [0.5_f32];
    let t1 = [0.5_f32];

    let locations = [
        far::LocationArray {
            ptex_index: 0,
            s: &s0,
            t: &t0,
        },
        far::LocationArray {
            ptex_index: 1,
            s: &s1,
            t: &t1,
        },
    ];

    let table = far::LimitStencilTable::new(
        &refiner,
        &locations,
        None,
        None,
        far::LimitStencilTableOptions::default(),
    )
    .unwrap();

    assert_eq!(table.len(), 3);
}

#[test]
fn limit_stencil_mismatched_st_lengths() {
    let refiner = cube_refiner();

    let s = [0.25_f32, 0.5];
    let t = [0.25_f32]; // mismatched

    let locations = [far::LocationArray {
        ptex_index: 0,
        s: &s,
        t: &t,
    }];

    let result = far::LimitStencilTable::new(
        &refiner,
        &locations,
        None,
        None,
        far::LimitStencilTableOptions::default(),
    );

    assert!(result.is_err());
}
