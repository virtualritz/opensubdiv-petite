use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use opensubdiv_petite::{far, tri_mesh_buffers};

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
        .add_systems(Update, (rotator_system, close_on_esc))
        .run();
}

#[derive(Component)]
struct Rotator;

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
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    // Build a disconnected triangle mesh. At level 6 the per-corner normals
    // computed within each face look smooth. We don't use bevy's
    // compute_smooth_normals here because averaging triangle normals at
    // extraordinary vertices (valence-3 corners with mixed-sharpness creases)
    // produces wrong tangent planes and leaves visible dark spots.
    let (indices, positions, normals) = tri_mesh_buffers::to_triangle_mesh_buffers(
        &refined_vertices,
        refiner
            .level(SUBDIVISION_LEVEL)
            .expect("missing refinement level")
            .face_vertices_iter(),
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}
