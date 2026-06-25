use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use opensubdiv_petite::far;

// Uniformly refine the dodecahedron this many levels.
const SUBDIVISION_LEVEL: usize = 6;

// Crease sharpness applied alternately along the 30 edges of the control
// cage (every other edge gets the second value).
const CREASE_SHARPNESS_A: f32 = 4.0;
const CREASE_SHARPNESS_B: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

#[derive(Component)]
struct Rotator;

#[allow(dead_code)]
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_secs());
    }
}

fn close_on_esc(mut exit: MessageWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
) {
    ambient_light.brightness = 300.0;
    ambient_light.color = Color::WHITE;

    commands.spawn((
        Mesh3d(meshes.add(subdivided_creased_dodecahedron())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.25, 0.0).with_scale(Vec3::splat(2.0)),
        Rotator,
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, -8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Transform::from_xyz(-2.0, 2.5, 5.0),
        PanOrbitCamera::default(),
    ));
}

/// Build the subdivided dodecahedron with all 30 edges creased at
/// [`CREASE_SHARPNESS`] and emit a bevy [`Mesh`] with smooth (averaged)
/// normals.
fn subdivided_creased_dodecahedron() -> Mesh {
    let phi: f32 = (1.0 + 5_f32.sqrt()) / 2.0;
    let inv_phi: f32 = 1.0 / phi;

    // Standard golden-ratio coordinates for a regular dodecahedron, vertices
    // numbered as below.
    #[rustfmt::skip]
    let vertices: [f32; 60] = [
         1.0,      1.0,      1.0,      // 0
         1.0,      1.0,     -1.0,      // 1
         1.0,     -1.0,      1.0,      // 2
         1.0,     -1.0,     -1.0,      // 3
        -1.0,      1.0,      1.0,      // 4
        -1.0,      1.0,     -1.0,      // 5
        -1.0,     -1.0,      1.0,      // 6
        -1.0,     -1.0,     -1.0,      // 7
         0.0,      phi,      inv_phi,  // 8
         0.0,      phi,     -inv_phi,  // 9
         0.0,     -phi,      inv_phi,  // 10
         0.0,     -phi,     -inv_phi,  // 11
         inv_phi,  0.0,      phi,      // 12
         inv_phi,  0.0,     -phi,      // 13
        -inv_phi,  0.0,      phi,      // 14
        -inv_phi,  0.0,     -phi,      // 15
         phi,      inv_phi,  0.0,      // 16
         phi,     -inv_phi,  0.0,      // 17
        -phi,      inv_phi,  0.0,      // 18
        -phi,     -inv_phi,  0.0,      // 19
    ];

    // 12 pentagonal faces, CCW from outside.
    let face_arities: [u32; 12] = [5; 12];
    #[rustfmt::skip]
    let face_vertices: [u32; 60] = [
         0,  8,  9,  1, 16,
         0, 16, 17,  2, 12,
         0, 12, 14,  4,  8,
         4, 14,  6, 19, 18,
         4, 18,  5,  9,  8,
         9,  5, 15, 13,  1,
         1, 13,  3, 17, 16,
         2, 17,  3, 11, 10,
        12,  2, 10,  6, 14,
         6, 10, 11,  7, 19,
         7, 15,  5, 18, 19,
         7, 11,  3, 13, 15,
    ];

    // The 30 unique edges of the dodecahedron, each creased at the same
    // sharpness so the subdivided surface keeps faceted pentagonal facets.
    #[rustfmt::skip]
    let crease_indices: [u32; 60] = [
         0,  8,    8,  9,    1,  9,    1, 16,    0, 16,
        16, 17,    2, 17,    2, 12,    0, 12,   12, 14,
         4, 14,    4,  8,    6, 14,    6, 19,   18, 19,
         4, 18,    5, 18,    5,  9,    5, 15,   13, 15,
         1, 13,    3, 13,    3, 17,    3, 11,   10, 11,
         2, 10,    6, 10,    7, 11,    7, 19,    7, 15,
    ];
    let crease_sharpness: [f32; 30] = std::array::from_fn(|i| {
        if i % 2 == 0 {
            CREASE_SHARPNESS_A
        } else {
            CREASE_SHARPNESS_B
        }
    });

    let mut descriptor =
        far::TopologyDescriptor::new(vertices.len() / 3, &face_arities, &face_vertices)
            .expect("Could not create TopologyDescriptor");
    descriptor = descriptor
        .creases(&crease_indices, &crease_sharpness)
        .expect("Could not add creases");

    let mut refiner = far::TopologyRefiner::new(
        descriptor,
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: SUBDIVISION_LEVEL,
        ..Default::default()
    });

    let primvar_refiner =
        far::PrimvarRefiner::new(&refiner).expect("Could not create PrimvarRefiner");

    // Carry vertex positions down through every refinement level.
    let mut refined_vertices = vertices.to_vec();
    for level in 1..=SUBDIVISION_LEVEL {
        refined_vertices = primvar_refiner
            .interpolate(level, 3, &refined_vertices)
            .expect("primvar interpolation failed");
    }

    // Build an indexed (shared-vertex) triangle list with manual smooth
    // normals: accumulate the *face* normal of each incident quad at every
    // corner vertex (uniform weight per face), then normalize.
    //
    // Bevy's `compute_smooth_normals` accumulates *triangle* normals weighted
    // by the triangle's corner angle. At valence-3 corners of the
    // dodecahedron with mixed-sharpness creases, sliver microtriangles get
    // near-zero weight and the average is dominated by larger triangles
    // whose normals point off-axis — producing visible dark spots at every
    // extraordinary vertex. Per-quad-uniform face-normal averaging avoids
    // that and gives the smooth shading the user expects.
    let final_level = refiner
        .level(SUBDIVISION_LEVEL)
        .expect("missing refinement level");

    let positions: Vec<[f32; 3]> = refined_vertices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    let mut normals_acc: Vec<Vec3> = vec![Vec3::ZERO; positions.len()];
    let mut indices: Vec<u32> = Vec::with_capacity(final_level.face_count() * 6);

    for face in final_level.face_vertices_iter() {
        // After Catmull-Clark every face at this level is a quad.
        let v: [u32; 4] = [face[0].0, face[1].0, face[2].0, face[3].0];
        let p: [Vec3; 4] = [
            Vec3::from(positions[v[0] as usize]),
            Vec3::from(positions[v[1] as usize]),
            Vec3::from(positions[v[2] as usize]),
            Vec3::from(positions[v[3] as usize]),
        ];
        // Face normal from the cross product of the quad's diagonals — robust
        // for slightly non-planar quads near extraordinary vertices.
        let face_normal = (p[2] - p[0]).cross(p[3] - p[1]).normalize_or_zero();
        for &vi in &v {
            normals_acc[vi as usize] += face_normal;
        }
        indices.extend_from_slice(&[v[0], v[1], v[2], v[0], v[2], v[3]]);
    }

    let normals: Vec<[f32; 3]> = normals_acc
        .into_iter()
        .map(|n| {
            let n = n.normalize_or_zero();
            [n.x, n.y, n.z]
        })
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}
