use anyhow::Result;
use opensubdiv_petite::far::{
    AdaptiveRefinementOptions, EndCapType, PatchTable, PatchTableOptions, PrimvarRefiner,
    TopologyDescriptor, TopologyRefiner, TopologyRefinerOptions,
};
use opensubdiv_petite::truck::{
    bfr_regular_surfaces, superpatch_surfaces, GregoryAccuracy, PatchTableExt, StepExportOptions,
};
use opensubdiv_petite::Index;
use truck_stepio::out::*;

fn main() -> Result<()> {
    // Creased cube: all edges sharpness 8.0
    let vertex_positions = vec![
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [-0.5, 0.5, -0.5],  // 2
        [0.5, 0.5, -0.5],   // 3
        [-0.5, 0.5, 0.5],   // 4
        [0.5, 0.5, 0.5],    // 5
        [-0.5, -0.5, 0.5],  // 6
        [0.5, -0.5, 0.5],   // 7
    ];

    let face_vertex_counts = vec![4, 4, 4, 4, 4, 4];
    let face_vertex_indices = vec![
        0, 1, 3, 2, // back
        2, 3, 5, 4, // top
        4, 5, 7, 6, // front
        6, 7, 1, 0, // bottom
        0, 2, 4, 6, // left
        1, 7, 5, 3, // right
    ];

    let mut descriptor = TopologyDescriptor::new(
        vertex_positions.len(),
        &face_vertex_counts,
        &face_vertex_indices,
    )?;

    // Crease only three edges sharing vertex 0; leave all others smooth.
    let crease_indices: [u32; 6] = [
        0, 1, // edge 0-1
        0, 2, // edge 0-2
        0, 6, // edge 0-6
    ];
    let crease_sharpness = [5.0f32; 3];
    descriptor.creases(&crease_indices, &crease_sharpness);

    let refiner_options = TopologyRefinerOptions::default();
    let mut refiner = TopologyRefiner::new(descriptor, refiner_options)?;

    // Selective adaptive refinement: refine faces touching sharp edges.
    let (selected_faces, approx_sharp, _has_creases) = {
        let base_level = refiner.level(0).expect("base level");
        let mut selected_faces = Vec::new();
        let mut face_set = std::collections::HashSet::new();

        for f in 0..base_level.face_count() {
            let edges = base_level.face_edges(Index::from(f as u32)).unwrap();
            let has_sharp = edges
                .iter()
                .any(|&e| base_level.edge_sharpness(e) > 0.0_f32);
            if has_sharp {
                let face_idx = Index::from(f as u32);
                if face_set.insert(face_idx) {
                    selected_faces.push(face_idx);
                }
                // Include neighbor faces sharing the sharp edge to avoid
                // T-junction gaps across sharp/smooth boundaries.
                for &edge in edges {
                    if let Some(neighbors) = base_level.edge_faces(edge) {
                        for &nf in neighbors {
                            if face_set.insert(nf) {
                                selected_faces.push(nf);
                            }
                        }
                    }
                }
            }
        }

        // Pick BFR sharp approximation depth based on maximum edge sharpness
        let mut max_sharpness = 0.0_f32;
        for e in 0..base_level.edge_count() {
            max_sharpness = max_sharpness.max(base_level.edge_sharpness(Index::from(e as u32)));
        }
        // BFR uses an isolation depth; push one level beyond max crease to
        // capture the full profile.
        let approx_sharp = (max_sharpness.ceil() as i32 + 1).max(0);

        (selected_faces, approx_sharp, max_sharpness > 0.0)
    };

    let mut adaptive_options = AdaptiveRefinementOptions::default();
    // Refine enough to isolate sharp edges; push one level beyond max sharpness.
    adaptive_options.isolation_level = (approx_sharp + 1).max(1) as usize;
    refiner.refine_adaptive(adaptive_options, &selected_faces);

    // Patch table with Gregory basis end cap
    let patch_options = PatchTableOptions::new()
        .end_cap_type(EndCapType::GregoryBasis)
        // Treat high sharpness as infinite to emphasize creases in export.
        .use_inf_sharp_patch(approx_sharp >= 8);
    let patch_table = PatchTable::new(&refiner, Some(patch_options))?;

    // Build vertex buffer (base + refined + local)
    let primvar_refiner = PrimvarRefiner::new(&refiner)?;
    let mut all_vertices = Vec::with_capacity(refiner.vertex_count_all_levels());
    all_vertices.extend_from_slice(&vertex_positions);

    for level in 1..refiner.refinement_levels() {
        let prev_count = refiner
            .level(level - 1)
            .map(|l| l.vertex_count())
            .unwrap_or(0);
        let start = all_vertices.len() - prev_count;
        let src_data: Vec<f32> = all_vertices[start..start + prev_count]
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();

        if let Some(refined) = primvar_refiner.interpolate(level, 3, &src_data) {
            let level_vertices: Vec<[f32; 3]> = refined
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            all_vertices.extend_from_slice(&level_vertices);
        }
    }

    // Add local points if any
    let num_local_points = patch_table.local_point_count();
    if num_local_points > 0 {
        if let Some(stencil_table) = patch_table.local_point_stencil_table() {
            let mut local_points = Vec::with_capacity(num_local_points);

            for dim in 0..3 {
                let src_dim: Vec<f32> = all_vertices.iter().map(|v| v[dim]).collect();
                let dst_dim = stencil_table.update_values(&src_dim, None, None);

                for (i, &val) in dst_dim.iter().enumerate() {
                    if dim == 0 {
                        local_points.push([val, 0.0, 0.0]);
                    } else {
                        local_points[i][dim] = val;
                    }
                }
            }

            all_vertices.extend_from_slice(&local_points);
        }
    }

    // BFR regular-only surfaces (crease-aware approx_sharp)
    println!("Creased cube: BFR regular surfaces...");
    let bfr_surfaces =
        bfr_regular_surfaces(&refiner, &all_vertices, 0, approx_sharp).unwrap_or_default();

    if bfr_surfaces.is_empty() {
        eprintln!("BFR produced no regular surfaces; falling back to PatchTable shell.");
        if let Ok(shell) = patch_table.to_truck_shell(&all_vertices) {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_bfr.step", step_string)?;
            println!("Wrote creased_cube_bfr.step (PatchTable fallback)");
        } else {
            println!(
                "Creased cube BFR conversion failed: no surfaces and PatchTable fallback failed"
            );
        }
    } else {
        let faces: Vec<truck_modeling::Face> = bfr_surfaces
            .into_iter()
            .map(|s| truck_modeling::Face::new(vec![], truck_modeling::Surface::BSplineSurface(s)))
            .collect();
        let shell = truck_modeling::Shell::from(faces);
        let compressed = shell.compress();
        let step_string =
            CompleteStepDisplay::new(StepModel::from(&compressed), Default::default()).to_string();
        std::fs::write("creased_cube_bfr.step", step_string)?;
        println!("Wrote creased_cube_bfr.step");
    }

    // Mixed export: always use BFR for regular faces and PatchTable for
    // irregular ones so every patch carries a full 4x4 control net instead of
    // the coarse, planar fallback shell.
    println!("Creased cube: BFR + PatchTable mixed surfaces...");
    match patch_table.to_truck_surfaces_bfr_mixed(&refiner, &all_vertices, 0, approx_sharp) {
        Ok(surfaces) if !surfaces.is_empty() => {
            let faces: Vec<truck_modeling::Face> = surfaces
                .into_iter()
                .map(|s| {
                    truck_modeling::Face::new(vec![], truck_modeling::Surface::BSplineSurface(s))
                })
                .collect();
            let shell = truck_modeling::Shell::from(faces);
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_bfr_mixed.step", step_string)?;
            println!("Wrote creased_cube_bfr_mixed.step");
        }
        _ => {
            eprintln!("Mixed export fell back to PatchTable shell.");
            match patch_table.to_truck_shell(&all_vertices) {
                Ok(shell) => {
                    let compressed = shell.compress();
                    let step_string =
                        CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                            .to_string();
                    std::fs::write("creased_cube_bfr_mixed.step", step_string)?;
                    println!("Wrote creased_cube_bfr_mixed.step (PatchTable fallback)");
                }
                Err(e) => println!("Creased cube BFR mixed conversion failed: {:?}", e),
            }
        }
    }

    // Superpatch export: merge adjacent regular patches into larger bicubic
    // surfaces to reduce patch count while keeping curvature.
    println!("Creased cube: superpatch export...");
    match superpatch_surfaces(&patch_table, &all_vertices, 1.0e-6) {
        Ok(surfaces) if !surfaces.is_empty() => {
            let faces: Vec<truck_modeling::Face> = surfaces
                .into_iter()
                .map(|s| {
                    truck_modeling::Face::new(vec![], truck_modeling::Surface::BSplineSurface(s))
                })
                .collect();
            let shell = truck_modeling::Shell::from(faces);
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_superpatch.step", step_string)?;
            println!("Wrote creased_cube_superpatch.step");
        }
        _ => {
            eprintln!("Superpatch export failed; falling back to PatchTable shell.");
            if let Ok(shell) = patch_table.to_truck_shell(&all_vertices) {
                let compressed = shell.compress();
                let step_string =
                    CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                        .to_string();
                std::fs::write("creased_cube_superpatch.step", step_string)?;
                println!("Wrote creased_cube_superpatch.step (PatchTable fallback)");
            }
        }
    }

    // Stitched shell from PatchTable (shared edges/vertices)
    println!("Creased cube: stitched shell export...");
    match patch_table.to_truck_shell_stitched(&all_vertices) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_stitched.step", step_string)?;
            println!("Wrote creased_cube_stitched.step");
        }
        Err(e) => println!("Creased cube stitched export failed: {:?}", e),
    }

    // -------------------------------------------------------------------------
    // NEW: Unified to_step_shell API with StepExportOptions
    // -------------------------------------------------------------------------

    // Default options: superpatch merging enabled, no stitching, BSpline end caps.
    println!("\nCreased cube: to_step_shell with default options...");
    match patch_table.to_step_shell(&all_vertices, StepExportOptions::default()) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_unified_default.step", step_string)?;
            println!("Wrote creased_cube_unified_default.step");
        }
        Err(e) => println!("Unified default export failed: {:?}", e),
    }

    // High precision Gregory fitting (8×8 sampling at extraordinary vertices).
    println!("Creased cube: to_step_shell with high precision Gregory...");
    match patch_table.to_step_shell(
        &all_vertices,
        StepExportOptions {
            gregory_accuracy: GregoryAccuracy::HighPrecision,
            use_superpatches: false, // Disable superpatches to show individual patches.
            ..Default::default()
        },
    ) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_unified_highprec.step", step_string)?;
            println!("Wrote creased_cube_unified_highprec.step");
        }
        Err(e) => println!("Unified high-precision export failed: {:?}", e),
    }

    // Stitched edges with superpatch merging.
    println!("Creased cube: to_step_shell with stitched edges...");
    match patch_table.to_step_shell(
        &all_vertices,
        StepExportOptions {
            stitch_edges: true,
            ..Default::default()
        },
    ) {
        Ok(shell) => {
            let compressed = shell.compress();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&compressed), Default::default())
                    .to_string();
            std::fs::write("creased_cube_unified_stitched.step", step_string)?;
            println!("Wrote creased_cube_unified_stitched.step");
        }
        Err(e) => println!("Unified stitched export failed: {:?}", e),
    }

    Ok(())
}
